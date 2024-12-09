pub use fuel_streams_core::prelude::FuelNetwork;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum ClientStatus {
    #[default]
    Pending,
    Connected,
}
