use std::{
    collections::HashMap,
    convert::TryFrom,
    future::Future,
    hash::Hash,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use derive_more::Deref;
use parking_lot::RwLock;
use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal,
};
use serde::{ser::SerializeStruct, Serialize, Serializer};
use sysinfo::{
    CpuExt,
    CpuRefreshKind,
    DiskExt,
    Pid,
    PidExt,
    ProcessExt,
    RefreshKind,
    SystemExt,
};
use thiserror::Error;
use tokio::time;

#[derive(Debug, Error)]
pub enum Error {
    #[error("The process {0} could not be found")]
    ProcessNotFound(Pid),
}

#[derive(Serialize)]
pub struct PromSystemMetrics {
    system: SystemMetrics,
}

impl From<SystemMetrics> for PromSystemMetrics {
    fn from(system: SystemMetrics) -> Self {
        Self { system }
    }
}

#[derive(Debug, Deref)]
pub struct System {
    /// System information from the `sysinfo` crate.
    #[deref]
    system: sysinfo::System,
    /// Refresh settings.
    specifics: RefreshKind,
    /// Cached physical CPU core count.
    cpu_physical_core_count: Option<usize>,
    /// Process ID.
    pid: Pid,
}

impl System {
    pub async fn new() -> Self {
        let specifics = RefreshKind::new().with_disks_list().with_memory();
        // Gathering CPU information takes about 150ms+ extra.
        let specifics = specifics.with_cpu(CpuRefreshKind::everything());

        let mut system = sysinfo::System::new_with_specifics(specifics);

        // We're only interested in the current process.
        // NOTE: This ::expect can never fail on Linux!
        let pid = sysinfo::get_current_pid().expect("Unable to get PID");
        system.refresh_process(pid);

        // We have to refresh the CPU statistics once on startup.
        time::sleep(Duration::from_millis(100)).await;
        system.refresh_process(pid);

        // Only retrieve the physical CPU core count once (while
        // hotplug CPUs exist on virtual and physical platforms, we
        // just assume that it is usually not changing on runtime).
        let cpu_physical_core_count = system.physical_core_count();

        Self {
            system,
            specifics,
            cpu_physical_core_count,
            pid,
        }
    }

    pub fn monitor(
        system: &Arc<RwLock<Self>>,
        interval: Duration,
    ) -> impl Future<Output = ()> {
        let system = system.clone();

        async move {
            let mut interval = time::interval(interval);

            // The first ticket is returning immediately.
            interval.tick().await;

            loop {
                interval.tick().await;

                // sysinfo is sync and taking ~8 to ~160ms on a 4-core
                // machine (with cpu_usage enabled) so there's the
                // risk that it takes >1s on an 32 core machine.  On
                // Linux, sysinfo accesses the /proc filesystem which
                // is supposed to be always ready so async I/O
                // wouldn't help much unless the processing itself is
                // also async.  We could eventually switch the backend
                // crate once there is an async version that is better
                // than the heim crate.
                let _ = tokio::task::spawn_blocking({
                    let system = system.clone();
                    move || {
                        system.write().refresh();
                    }
                })
                .await;
            }
        }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_process(self.pid);
        self.system.refresh_specifics(self.specifics);
    }

    pub fn metrics(&self) -> Result<SystemMetrics, Error> {
        SystemMetrics::try_from(self)
    }

    fn pid(&self) -> Pid {
        self.pid
    }
}

/// Accumulated system status information.
#[derive(Debug, Default, Serialize)]
pub struct SystemMetrics {
    /// Parent process of the application.
    pub application: Process,
    /// System memory information.
    pub memory: SystemMemory,
    /// Load averages
    pub load_average: LoadAverage,
    /// Host and operation system information.
    pub host: Host,
    /// Disk information and usage.
    pub disk: HashMap<PathBuf, Disk>,
    /// CPU physical core count.
    #[serde(serialize_with = "format_value")]
    pub cpu_physical_core_count: usize,
    /// CPU count.
    #[serde(serialize_with = "format_value")]
    pub cpu_count: usize,
    /// CPU information.
    pub cpu: HashMap<usize, Cpu>,
}

impl TryFrom<&System> for SystemMetrics {
    type Error = Error;

    fn try_from(system: &System) -> Result<Self, Self::Error> {
        // Get current pid.
        let pid = system.pid();

        let disk = system
            .disks()
            .iter()
            .map(|v| {
                let path = v.mount_point().to_path_buf();
                let disk = Disk::from(v);
                (path, disk)
            })
            .collect();

        let cpu = system
            .cpus()
            .iter()
            .enumerate()
            .map(|(i, v)| (i, v.into()))
            .collect::<HashMap<_, _>>();
        // Total number of CPUs (including CPU threads).
        let cpu_count = cpu.len();

        // Use cached number of CPU physical cores, if set.
        let cpu_physical_core_count = system
            .cpu_physical_core_count
            .unwrap_or_else(|| system.physical_core_count().unwrap_or(1));

        Ok(Self {
            application: TryFrom::try_from((system.deref(), pid))?,
            memory: system.deref().into(),
            load_average: system.deref().into(),
            host: system.deref().into(),
            disk,
            cpu_count,
            cpu_physical_core_count,
            cpu,
        })
    }
}

/// System memory usage information.
#[derive(Debug, Clone, Default)]
pub struct Memory {
    /// Total memory.
    size: u64,
    /// Used memory.
    free: Option<u64>,
    /// Memory usage in percent.
    usage: Decimal,
}

impl serde::Serialize for Memory {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        if let Some(free) = self.free {
            let mut s = serializer.serialize_struct("Memory", 3)?;
            s.serialize_field("size", &Format::Memory(self.size))?;
            s.serialize_field("free", &Format::Memory(free))?;
            s.serialize_field("usage", &Format::Memory(AsF64(self.usage)))?;
            s.end()
        } else {
            let mut s = serializer.serialize_struct("Memory", 2)?;
            s.serialize_field("size", &Format::Memory2(self.size))?;
            s.serialize_field("usage", &Format::Memory2(AsF64(self.usage)))?;
            s.end()
        }
    }
}

/// System memory usage information.
#[derive(Debug, Default, Serialize)]
pub struct SystemMemory {
    /// System memory.
    system: Memory,
    /// Swap memory.
    swap: Memory,
}

impl From<&sysinfo::System> for SystemMemory {
    fn from(system: &sysinfo::System) -> Self {
        let size = system.total_memory();
        let used = system.used_memory();
        let free = Some(size.saturating_sub(used));
        let usage = percent_usage(used, size);

        let swap_size = system.total_swap();
        let swap_used = system.used_swap();
        let swap_free = Some(swap_size.saturating_sub(swap_used));
        let swap_usage = percent_usage(swap_used, swap_size);

        Self {
            system: Memory { size, free, usage },
            swap: Memory {
                size: swap_size,
                free: swap_free,
                usage: swap_usage,
            },
        }
    }
}

/// Process information and metrics.
#[derive(Debug)]
pub struct Process {
    pid: Pid,
    name: String,
    cpu_usage: Decimal,
    memory: Memory,
}

impl Default for Process {
    fn default() -> Self {
        Self {
            pid: Pid::from(0),
            name: Default::default(),
            cpu_usage: Default::default(),
            memory: Default::default(),
        }
    }
}

impl serde::Serialize for Process {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("Process", 4)?;
        s.serialize_field(
            "pid",
            &Format::<i32>::Process(self.pid.as_u32() as i32),
        )?;
        s.serialize_field("name", &FormatKey(&self.name))?;
        s.serialize_field(
            "cpu_usage",
            &Format::Process(AsF64(self.cpu_usage)),
        )?;
        s.serialize_field("memory", &self.memory)?;
        s.end()
    }
}

impl TryFrom<(&sysinfo::System, Pid)> for Process {
    type Error = Error;

    fn try_from(
        (system, pid): (&sysinfo::System, Pid),
    ) -> Result<Self, Self::Error> {
        let process = system.process(pid).ok_or(Error::ProcessNotFound(pid))?;

        let total = system.total_memory();
        let size = process.memory();
        let usage = percent_usage(size, total);

        Ok(Self {
            memory: Memory {
                size,
                free: None,
                usage,
            },
            ..Self::from(process)
        })
    }
}

impl From<&sysinfo::Process> for Process {
    fn from(process: &sysinfo::Process) -> Self {
        Self {
            name: process.name().to_string(),
            pid: process.pid(),
            cpu_usage: decimal(process.cpu_usage()),
            memory: Default::default(),
        }
    }
}

/// Disk information and usage.
#[derive(Debug)]
pub struct Disk {
    size: u64,
    free: u64,
    usage: Decimal,
}

impl serde::Serialize for Disk {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("Disk", 2)?;
        s.serialize_field("size", &Format::Disk(self.size))?;
        s.serialize_field("free", &Format::Disk(self.free))?;
        s.serialize_field("usage", &Format::Disk(AsF64(self.usage)))?;
        s.end()
    }
}

impl From<&sysinfo::Disk> for Disk {
    fn from(disk: &sysinfo::Disk) -> Self {
        let size = disk.total_space();
        let free = disk.available_space();
        let used = size.saturating_sub(free);

        // Calculate the disk usage in percent.
        let usage = percent_usage(used, size);

        Self { size, free, usage }
    }
}

/// System memory usage information.
#[derive(Debug, Default)]
pub struct LoadAverage(f64, f64, f64);

impl serde::Serialize for LoadAverage {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("LoadAverage", 3)?;
        s.serialize_field("1", &Format::LoadAverage(self.0))?;
        s.serialize_field("5", &Format::LoadAverage(self.1))?;
        s.serialize_field("15", &Format::LoadAverage(self.2))?;
        s.end()
    }
}

impl From<&sysinfo::System> for LoadAverage {
    fn from(system: &sysinfo::System) -> Self {
        let load_avg = system.load_average();
        Self(load_avg.one, load_avg.five, load_avg.fifteen)
    }
}

/// System memory usage information.
#[derive(Debug, Default)]
pub struct Cpu {
    #[allow(dead_code)]
    name: String,
    frequency: u64,
    usage: Decimal,
}

impl serde::Serialize for Cpu {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("Cpu", 2)?;
        s.serialize_field("frequency", &Format::Cpu(self.frequency))?;
        s.serialize_field("usage", &Format::Cpu(AsF64(self.usage)))?;
        s.end()
    }
}

impl From<&sysinfo::Cpu> for Cpu {
    fn from(cpu: &sysinfo::Cpu) -> Self {
        Self {
            name: cpu.brand().to_string(),
            frequency: cpu.frequency(),
            usage: decimal(cpu.cpu_usage()),
        }
    }
}

/// System memory usage information.
#[derive(Debug, Default)]
pub struct Host {
    os_version: String,
    kernel_version: String,
    uptime: u64,
}

impl serde::Serialize for Host {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("Host", 3)?;
        s.serialize_field("os_version", &FormatKey(&self.os_version))?;
        s.serialize_field("kernel_version", &FormatKey(&self.kernel_version))?;
        s.serialize_field("uptime", &Format::Host(self.uptime))?;
        s.end()
    }
}

impl From<&sysinfo::System> for Host {
    fn from(system: &sysinfo::System) -> Self {
        Self {
            os_version: system.long_os_version().unwrap_or_default(),
            kernel_version: system.kernel_version().unwrap_or_default(),
            uptime: system.uptime(),
        }
    }
}

struct AsF64(Decimal);

impl Serialize for AsF64 {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        use serde::ser::Error;
        let value = self.0.to_f64().ok_or_else(|| {
            S::Error::custom(format!(
                "Failed to convert a Decimal value into a f64: {:?}",
                self.0
            ))
        })?;
        value.serialize(serializer)
    }
}

pub enum Format<T: serde::Serialize> {
    Cpu(T),
    Disk(T),
    Host(T),
    LoadAverage(T),
    Memory(T),
    Memory2(T),
    Process(T),
}

impl<T: serde::Serialize> serde::Serialize for Format<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // https://en.wikipedia.org/wiki/Brainfuck light
        let (code, v) = match &self {
            Self::Host(v) => ("<<-|", v),
            Self::Cpu(v) => ("<!<|id==-<", v),
            Self::Disk(v) => ("<!<|path==-<", v),
            Self::LoadAverage(v) => ("<<-|", v),
            Self::Memory(v) => ("<!<<|type==-<", v),
            Self::Memory2(v) => ("<<!<|type==--<", v),
            Self::Process(v) => ("<<<|", v),
        };

        serializer.serialize_newtype_struct(code, v)
    }
}

pub struct FormatKey<T: serde::Serialize + Eq + Hash>(T);

impl<T: serde::Serialize + Eq + Hash> serde::Serialize for FormatKey<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut hashmap = HashMap::new();
        hashmap.insert(&self.0, 1);
        serializer.serialize_newtype_struct(".<<<|", &hashmap)
    }
}

fn format_value<S>(value: &usize, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_newtype_struct("<<-|", value)
}

const DECIMAL_PRECISION: u32 = 4;

#[inline]
fn percent_usage(current: u64, max: u64) -> Decimal {
    Decimal::from(current)
        .checked_div(Decimal::from(max))
        .unwrap_or_default()
        .checked_mul(100.into())
        .unwrap_or_default()
        .round_dp(DECIMAL_PRECISION)
}

#[inline]
fn decimal(current: f32) -> Decimal {
    Decimal::from_f32(current)
        .unwrap_or_default()
        .round_dp(DECIMAL_PRECISION)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rust_decimal::Decimal;
    use serde::Serialize;

    use super::{
        Cpu,
        Disk,
        Host,
        LoadAverage,
        Memory,
        Pid,
        Process,
        System,
        SystemMemory,
    };

    #[derive(Serialize)]
    pub struct Metrics {
        system: super::SystemMetrics,
    }

    impl From<&System> for Metrics {
        fn from(system: &System) -> Self {
            Self {
                system: system.metrics().expect("metrics"),
            }
        }
    }

    #[tokio::test]
    async fn test_metrics_system_values() {
        let system = System::new().await;
        let metrics = Metrics::from(&system);

        // NOTE: This ::expect can never fail on Linux!
        let pid = sysinfo::get_current_pid().expect("Unable to get PID");
        assert_eq!(metrics.system.application.pid, pid);
        assert!(metrics.system.host.uptime > 0);
        assert!(!metrics.system.cpu.is_empty());
        assert!(!metrics.system.disk.is_empty());
    }

    #[tokio::test]
    async fn test_metrics_system_prometheus_full() {
        let memory = Memory {
            size: 1000,
            free: Some(877),
            usage: Decimal::new(1234, 2),
        };

        let metrics = Metrics {
            system: super::SystemMetrics {
                application: Process {
                    pid: Pid::from(0),
                    name: "process".to_string(),
                    cpu_usage: Decimal::new(1234, 2),
                    memory: memory.clone(),
                },
                memory: SystemMemory {
                    system: memory.clone(),
                    swap: memory,
                },
                load_average: LoadAverage(1.2, 2.3, 3.4),
                host: Host {
                    os_version: "os-version".to_string(),
                    kernel_version: "kernel-version".to_string(),
                    uptime: 123456,
                },
                disk: vec![(
                    PathBuf::from("disk1"),
                    Disk {
                        size: 1000,
                        free: 877,
                        usage: Decimal::new(1234, 2),
                    },
                )]
                .into_iter()
                .collect(),
                cpu_physical_core_count: 1,
                cpu_count: 1,
                cpu: vec![(
                    1,
                    Cpu {
                        name: "cpu1".to_string(),
                        frequency: 12345,
                        usage: Decimal::new(1234, 2),
                    },
                )]
                .into_iter()
                .collect(),
            },
        };
        let output = serde_prometheus::to_string(&metrics, None, &[])
            .expect("prometheus");

        assert_eq!(
            output.trim_end().split('\n').collect::<Vec<&str>>(),
            vec![
                r#"system_application_pid 0"#,
                r#"system_application_name{path = "process"} 1"#,
                r#"system_application_cpu_usage 12.34"#,
                r#"system_application_size{type = "memory"} 1000"#,
                r#"system_application_free{type = "memory"} 877"#,
                r#"system_application_usage{type = "memory"} 12.34"#,
                r#"system_memory_size{type = "system"} 1000"#,
                r#"system_memory_free{type = "system"} 877"#,
                r#"system_memory_usage{type = "system"} 12.34"#,
                r#"system_memory_size{type = "swap"} 1000"#,
                r#"system_memory_free{type = "swap"} 877"#,
                r#"system_memory_usage{type = "swap"} 12.34"#,
                r#"system_load_average_1 1.2"#,
                r#"system_load_average_5 2.3"#,
                r#"system_load_average_15 3.4"#,
                r#"system_host_os_version{path = "os-version"} 1"#,
                r#"system_host_kernel_version{path = "kernel-version"} 1"#,
                r#"system_host_uptime 123456"#,
                r#"system_disk_size{path = "disk1"} 1000"#,
                r#"system_disk_free{path = "disk1"} 877"#,
                r#"system_disk_usage{path = "disk1"} 12.34"#,
                r#"system_cpu_physical_core_count 1"#,
                r#"system_cpu_count 1"#,
                r#"system_cpu_frequency{id = "1"} 12345"#,
                r#"system_cpu_usage{id = "1"} 12.34"#,
            ]
        )
    }
}
