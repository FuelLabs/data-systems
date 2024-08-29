#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum ClientStatus {
    #[default]
    Pending,
    Connected,
}
