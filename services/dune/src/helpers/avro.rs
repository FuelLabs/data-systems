use std::{
    fs::File,
    io::{
        BufWriter,
        Read,
        Write,
    },
    path::{
        Path,
        PathBuf,
    },
    sync::Arc,
};

use apache_avro::{
    AvroSchema,
    Codec,
    Reader,
    Schema,
    Writer,
    from_value,
    schema::{
        Namespace,
        derive::AvroSchemaComponent,
    },
};
use serde::{
    Serialize,
    de::DeserializeOwned,
};

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

/// An Avro writer that writes directly to a file on disk.
/// This reduces memory usage by not accumulating data in memory.
pub struct AvroFileWriter<T> {
    writer: Writer<'static, BufWriter<File>>,
    file_path: PathBuf,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> AvroFileWriter<T>
where
    T: AvroSchema + AvroSchemaComponent + Serialize + Send + Sync + 'static,
{
    /// Creates a new file-based Avro writer at the specified path
    pub fn new(
        path: impl AsRef<Path>,
        schema: Schema,
        codec: Codec,
    ) -> Result<Self, AvroParserError> {
        let file_path = path.as_ref().to_path_buf();
        let file = File::create(&file_path)
            .map_err(|e| AvroParserError::Io(format!("Failed to create file: {}", e)))?;
        let buf_writer = BufWriter::new(file);

        let schema_static: &'static Schema = Box::leak(Box::new(schema));
        let writer = Writer::builder()
            .schema(schema_static)
            .codec(codec)
            .writer(buf_writer)
            .build();

        Ok(Self {
            writer,
            file_path,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Appends a value to the file
    pub fn append(&mut self, value: &T) -> Result<(), AvroParserError> {
        self.writer.append_ser(value)?;
        Ok(())
    }

    /// Flushes any buffered data to disk
    pub fn flush(&mut self) -> Result<(), AvroParserError> {
        self.writer
            .flush()
            .map_err(|e| AvroParserError::Io(format!("Failed to flush writer: {}", e)))?;
        Ok(())
    }

    /// Returns the path to the file
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Finalizes the file and returns the file contents as bytes.
    /// This flushes all data, closes the writer, and reads the file.
    pub fn finalize(self) -> Result<Vec<u8>, AvroParserError> {
        let mut inner = self.writer.into_inner().map_err(|e| {
            AvroParserError::Io(format!("Failed to finalize writer: {}", e))
        })?;
        inner.flush().map_err(|e| {
            AvroParserError::Io(format!("Failed to flush final data: {}", e))
        })?;
        drop(inner); // Close the file

        // Read the file contents
        let mut file = File::open(&self.file_path).map_err(|e| {
            AvroParserError::Io(format!("Failed to open file for reading: {}", e))
        })?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)
            .map_err(|e| AvroParserError::Io(format!("Failed to read file: {}", e)))?;

        Ok(contents)
    }

    /// Finalizes the file and returns just the path (for cleanup later).
    /// Use this when you want to handle file reading separately.
    pub fn finalize_path(self) -> Result<PathBuf, AvroParserError> {
        let mut inner = self.writer.into_inner().map_err(|e| {
            AvroParserError::Io(format!("Failed to finalize writer: {}", e))
        })?;
        inner.flush().map_err(|e| {
            AvroParserError::Io(format!("Failed to flush final data: {}", e))
        })?;
        Ok(self.file_path)
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
        let schema =
            T::get_schema_in_ctxt(&mut Default::default(), &Namespace::default());
        Ok(AvroWriter::new(
            schema,
            self.codec.unwrap_or(Codec::Deflate),
        ))
    }

    /// Creates a file-based Avro writer that writes directly to disk.
    /// Use this to avoid accumulating large amounts of data in memory.
    pub fn file_writer_with_schema<
        T: AvroSchema + AvroSchemaComponent + Serialize + Send + Sync + 'static,
    >(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<AvroFileWriter<T>, AvroParserError> {
        let schema =
            T::get_schema_in_ctxt(&mut Default::default(), &Namespace::default());
        AvroFileWriter::new(path, schema, self.codec.unwrap_or(Codec::Deflate))
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
