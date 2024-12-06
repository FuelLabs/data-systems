use clap::Parser;

/// CLI structure for parsing command-line arguments.
///
/// - `conig_path`: Path to the toml config file.
#[derive(Clone, Parser)]
pub struct Cli {
    /// Config path
    #[arg(
        long,
        value_name = "CONFIG",
        env = "CONFIG_PATH",
        default_value = "config.toml",
        help = "Path to toml config file"
    )]
    pub config_path: Option<String>,
}
