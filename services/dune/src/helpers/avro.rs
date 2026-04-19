use std::{
    any::TypeId,
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    sync::RwLock,
};

use crate::alloc_counter;

use apache_avro::{
    AvroSchema, Codec, Reader, Schema, Writer, from_value,
    schema::{Namespace, derive::AvroSchemaComponent},
};
use serde::{Serialize, de::DeserializeOwned};

/// Global cache for Avro schemas, keyed by TypeId.
/// Schemas are immutable and type-specific, so we cache them to avoid
/// repeated allocations and memory leaks from Box::leak.
static SCHEMA_CACHE: RwLock<Option<HashMap<TypeId, &'static Schema>>> = RwLock::new(None);

/// Gets or creates a cached static schema for type T.
/// The schema is created once per type and cached globally.
fn get_cached_schema<T: AvroSchema + AvroSchemaComponent + 'static>() -> &'static Schema {
    let type_id = TypeId::of::<T>();

    // Fast path: try to read from cache
    {
        let cache = SCHEMA_CACHE.read().unwrap();
        if let Some(ref map) = *cache
            && let Some(schema) = map.get(&type_id)
        {
            return schema;
        }
    }

    // Slow path: create schema and cache it
    let mut cache = SCHEMA_CACHE.write().unwrap();
    let map = cache.get_or_insert_with(HashMap::new);

    // Double-check after acquiring write lock
    if let Some(schema) = map.get(&type_id) {
        return schema;
    }

    // Create and cache the schema
    let schema = T::get_schema_in_ctxt(&mut Default::default(), &Namespace::default());
    let schema_static: &'static Schema = Box::leak(Box::new(schema));
    map.insert(type_id, schema_static);
    schema_static
}

/// Data parser error types.
#[derive(Debug, thiserror::Error)]
pub enum AvroParserError {
    #[error(transparent)]
    Avro(Box<apache_avro::Error>),
    #[error("Schema not found {0}")]
    SchemaNotFound(String),
    #[error("IO error: {0}")]
    Io(String),
}

impl From<apache_avro::Error> for AvroParserError {
    fn from(err: apache_avro::Error) -> Self {
        AvroParserError::Avro(Box::new(err))
    }
}

pub struct AvroWriter<T> {
    writer: Writer<'static, Vec<u8>>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> AvroWriter<T>
where
    T: AvroSchema + AvroSchemaComponent + Serialize + Send + Sync + 'static,
{
    pub fn new(codec: Codec) -> Self {
        let schema = get_cached_schema::<T>();
        let writer = Writer::builder()
            .schema(schema)
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

/// An Avro writer that writes directly to a file on disk.
/// This reduces memory usage by not accumulating data in memory.
pub struct AvroFileWriter<T> {
    writer: Option<Writer<'static, BufWriter<File>>>,
    file_path: PathBuf,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> AvroFileWriter<T>
where
    T: AvroSchema + AvroSchemaComponent + Serialize + Send + Sync + 'static,
{
    /// Creates a new file-based Avro writer at the specified path
    pub fn new(path: impl AsRef<Path>, codec: Codec) -> Result<Self, AvroParserError> {
        let file_path = path.as_ref().to_path_buf();
        let file = File::create(&file_path)
            .map_err(|e| AvroParserError::Io(format!("Failed to create file: {}", e)))?;
        let buf_writer = BufWriter::new(file);

        let schema = get_cached_schema::<T>();
        let writer = Writer::builder()
            .schema(schema)
            .codec(codec)
            .writer(buf_writer)
            .build();

        alloc_counter::inc(&alloc_counter::AVRO_FILE_WRITER);
        Ok(Self {
            writer: Some(writer),
            file_path,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Appends a value to the file.
    ///
    /// Note: Data is buffered internally by the Avro Writer. Call `flush()`
    /// periodically to write buffered data to disk and prevent memory accumulation.
    pub fn append(&mut self, value: &T) -> Result<(), AvroParserError> {
        self.writer
            .as_mut()
            .ok_or_else(|| AvroParserError::Io("Writer already finalized".into()))?
            .append_ser(value)?;
        Ok(())
    }

    /// Flushes buffered data to disk.
    ///
    /// The Avro Writer buffers data internally for performance. Without
    /// periodic flushing, all data accumulates in memory until finalize_path().
    /// Call this after processing each block to bound memory usage.
    pub fn flush(&mut self) -> Result<(), AvroParserError> {
        self.writer
            .as_mut()
            .ok_or_else(|| AvroParserError::Io("Writer already finalized".into()))?
            .flush()?;
        Ok(())
    }

    /// Finalizes the file and returns just the path.
    /// The file is flushed and closed, ready for streaming to its destination.
    /// The inner Writer is taken via `.take()`, consumed by `into_inner()`,
    /// and deallocated. `self` then drops with `writer: None`, firing `Drop`
    /// which decrements the counter.
    pub fn finalize_path(mut self) -> Result<PathBuf, AvroParserError> {
        let writer = self
            .writer
            .take()
            .ok_or_else(|| AvroParserError::Io("Writer already finalized".into()))?;
        let mut inner = writer.into_inner().map_err(|e| {
            AvroParserError::Io(format!("Failed to finalize writer: {}", e))
        })?;
        inner.flush().map_err(|e| {
            AvroParserError::Io(format!("Failed to flush final data: {}", e))
        })?;
        Ok(self.file_path.clone())
    }
}

impl<T> Drop for AvroFileWriter<T> {
    fn drop(&mut self) {
        alloc_counter::dec(&alloc_counter::AVRO_FILE_WRITER);
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
    pub fn writer_with_schema<
        T: AvroSchema + AvroSchemaComponent + Serialize + Send + Sync + 'static,
    >(
        &self,
    ) -> Result<AvroWriter<T>, AvroParserError> {
        Ok(AvroWriter::new(self.codec.unwrap_or(Codec::Deflate)))
    }

    /// Creates a file-based Avro writer that writes directly to disk.
    /// Use this to avoid accumulating large amounts of data in memory.
    pub fn file_writer_with_schema<
        T: AvroSchema + AvroSchemaComponent + Serialize + Send + Sync + 'static,
    >(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<AvroFileWriter<T>, AvroParserError> {
        AvroFileWriter::new(path, self.codec.unwrap_or(Codec::Deflate))
    }

    pub fn reader_with_schema<
        T: AvroSchema + AvroSchemaComponent + DeserializeOwned + Send + Sync + 'static,
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
    T: AvroSchema + AvroSchemaComponent + DeserializeOwned + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> AvroReader<T>
where
    T: AvroSchema + AvroSchemaComponent + DeserializeOwned + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn deserialize(self, data: &[u8]) -> Result<Vec<T>, AvroParserError> {
        let schema =
            T::get_schema_in_ctxt(&mut Default::default(), &Namespace::default());
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

    #[derive(Debug, Default, Deserialize, Serialize, Eq, PartialEq, AvroSchema)]
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
