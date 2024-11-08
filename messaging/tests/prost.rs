use bytes::Bytes;

/// Prost Message Wrapper allowing serialization/deserialization
pub(crate) struct ProstMessageSerdelizer<T: prost::Message>(pub(crate) T);

impl<T> ProstMessageSerdelizer<T>
where
    T: prost::Message + std::default::Default,
{
    /// Method to serialize
    pub(crate) fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.0.encode(&mut buf).map_err(|e| anyhow::anyhow!("prost encoding error {:?}", e))?;
        Ok(buf)
    }

    /// Method to deserialize
    #[allow(dead_code)]
    pub(crate) fn deserialize(buf: Vec<u8>) -> anyhow::Result<T> {
        T::decode(Bytes::from(buf)).map_err(|e| anyhow::anyhow!("prost decoding error {:?}", e))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_no_schema() {}
}
