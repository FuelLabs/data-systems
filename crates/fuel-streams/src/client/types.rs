pub use fuel_streams_core::nats::FuelNetwork;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum ClientStatus {
    #[default]
    Pending,
    Connected,
}
