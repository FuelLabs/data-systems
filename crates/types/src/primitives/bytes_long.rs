#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    Default,
)]
pub struct LongBytes(pub Vec<u8>);

impl LongBytes {
    pub fn zeroed() -> Self {
        Self(vec![0; 32])
    }
}
impl AsRef<[u8]> for LongBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
impl AsMut<[u8]> for LongBytes {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
impl From<Vec<u8>> for LongBytes {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}
impl std::fmt::Display for LongBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}
impl From<&[u8]> for LongBytes {
    fn from(value: &[u8]) -> Self {
        Self(value.to_vec())
    }
}
