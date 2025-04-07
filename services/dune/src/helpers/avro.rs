use std::sync::Arc;

use apache_avro::{
    from_value,
    schema::{derive::AvroSchemaComponent, Namespace},
    AvroSchema,
    Codec,
    Reader,
    Schema,
    Writer,
};
use serde::{de::DeserializeOwned, Serialize};

/// Data parser error types.
#[derive(Debug, thiserror::Error)]
pub enum AvroParserError {
    #[error(transparent)]
    Avro(#[from] apache_avro::Error),
    #[error("Schema not found {0}")]
    SchemaNotFound(String),
}

pub struct AvroWriter<T> {
    writer: Writer<'static, Vec<u8>>, // We'll adjust this
    _phantom: std::marker::PhantomData<T>,
}

impl<T> AvroWriter<T>
where
    T: AvroSchema + AvroSchemaComponent + Serialize + Send + Sync + 'static,
{
    pub fn new(schema: Schema, codec: Codec) -> Self {
        let schema_static: &'static Schema = Box::leak(Box::new(schema));
        let writer = Writer::builder()
            .schema(schema_static)
            .codec(codec)
            .writer(Vec::new())
            .build();
        Self {
            writer,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn append(&mut self, value: &T) -> Result<(), AvroParserError> {
        self.writer.append_ser(value)?;
        Ok(())
    }

    pub fn into_inner(self) -> Result<Vec<u8>, AvroParserError> {
        Ok(self.writer.into_inner()?.to_vec())
    }
}

#[derive(Clone)]
pub struct AvroParser {
    codec: Option<Codec>,
}

impl Default for AvroParser {
    fn default() -> Self {
        Self {
            codec: Some(Codec::Deflate),
        }
    }
}

impl AvroParser {
    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    pub fn with_codec(&mut self, codec: Codec) -> &mut Self {
        self.codec = Some(codec);
        self
    }

    pub fn writer_with_schema<
        T: AvroSchema + AvroSchemaComponent + Serialize + Send + Sync + 'static,
    >(
        &self,
    ) -> Result<AvroWriter<T>, AvroParserError> {
        let schema = T::get_schema_in_ctxt(
            &mut Default::default(),
            &Namespace::default(),
        );
        Ok(AvroWriter::new(
            schema,
            self.codec.unwrap_or(Codec::Deflate),
        ))
    }

    pub fn reader_with_schema<
        T: AvroSchema
            + AvroSchemaComponent
            + DeserializeOwned
            + Send
            + Sync
            + 'static,
    >(
        &self,
    ) -> Result<AvroReader<T>, AvroParserError> {
        Ok(AvroReader::new())
    }
}

pub struct AvroReader<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Default for AvroReader<T>
where
    T: AvroSchema
        + AvroSchemaComponent
        + DeserializeOwned
        + Send
        + Sync
        + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> AvroReader<T>
where
    T: AvroSchema
        + AvroSchemaComponent
        + DeserializeOwned
        + Send
        + Sync
        + 'static,
{
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn deserialize(self, data: &[u8]) -> Result<Vec<T>, AvroParserError> {
        let schema = T::get_schema_in_ctxt(
            &mut Default::default(),
            &Namespace::default(),
        );
        let cursor = std::io::Cursor::new(data);
        let reader = Reader::with_schema(&schema, cursor)?;

        let mut list = vec![];
        for record in reader {
            let record = record?;
            let decoded = from_value::<T>(&record)?;
            list.push(decoded);
        }
        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(
        Debug, Default, Deserialize, Serialize, Eq, PartialEq, AvroSchema,
    )]
    struct Test {
        a: i64,
        b: String,
    }

    #[test]
    fn test_avro_parser() {
        let parser = AvroParser::default();
        let test = Test {
            a: 27,
            b: "foo".to_owned(),
        };

        // serialize
        let mut avro_writer = parser.writer_with_schema::<Test>().unwrap();
        avro_writer.append(&test).unwrap();
        let serialized = avro_writer.into_inner().unwrap();

        // deserialize
        let deserialized = parser
            .reader_with_schema::<Test>()
            .unwrap()
            .deserialize(&serialized)
            .unwrap();

        // assert
        assert!(deserialized.len() == 1);
        assert!(test == deserialized[0]);
    }
}
