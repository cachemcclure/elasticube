//! Data source connectors for ElastiCube

use crate::error::{Error, Result};
use arrow::datatypes::Schema as ArrowSchema;
use arrow::record_batch::{RecordBatch, RecordBatchReader};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

/// Trait for data sources that can load data into a cube
///
/// Data sources must be Send + Sync to allow use in multi-threaded contexts,
/// particularly for Python bindings via PyO3.
pub trait DataSource: std::fmt::Debug + Send + Sync {
    /// Load data from the source
    ///
    /// Returns a tuple of (Arrow schema, vector of RecordBatches)
    fn load(&self) -> Result<(Arc<ArrowSchema>, Vec<RecordBatch>)>;
}

/// CSV data source configuration
#[derive(Debug, Clone)]
pub struct CsvSource {
    /// Path to the CSV file
    path: String,

    /// Whether the CSV has a header row
    has_header: bool,

    /// Batch size for reading (number of rows per batch)
    batch_size: usize,

    /// Optional schema (if None, will be inferred)
    schema: Option<Arc<ArrowSchema>>,

    /// Delimiter character (default: ',')
    delimiter: u8,
}

impl CsvSource {
    /// Create a new CSV source
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            has_header: true,
            batch_size: 8192,
            schema: None,
            delimiter: b',',
        }
    }

    /// Set whether the CSV has a header row
    pub fn with_header(mut self, has_header: bool) -> Self {
        self.has_header = has_header;
        self
    }

    /// Set the batch size for reading
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    /// Set the expected schema
    pub fn with_schema(mut self, schema: Arc<ArrowSchema>) -> Self {
        self.schema = Some(schema);
        self
    }

    /// Set the delimiter character
    pub fn with_delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }
}

impl DataSource for CsvSource {
    fn load(&self) -> Result<(Arc<ArrowSchema>, Vec<RecordBatch>)> {
        use arrow_csv::ReaderBuilder;

        // Open the file
        let file = File::open(&self.path).map_err(|e| {
            Error::io(format!("Failed to open CSV file '{}': {}", self.path, e))
        })?;

        // Create format with delimiter
        let format = arrow_csv::reader::Format::default()
            .with_header(self.has_header)
            .with_delimiter(self.delimiter);

        // Build the CSV reader with or without schema
        let reader = if let Some(schema) = &self.schema {
            ReaderBuilder::new(schema.clone())
                .with_format(format)
                .with_batch_size(self.batch_size)
                .build(file)
                .map_err(|e| {
                    Error::arrow(format!("Failed to create CSV reader: {}", e))
                })?
        } else {
            // For schema inference, create a buffered reader first
            let buf_reader = BufReader::new(file);
            let (inferred_schema, _) = format.infer_schema(buf_reader, Some(100))
                .map_err(|e| {
                    Error::arrow(format!("Failed to infer CSV schema: {}", e))
                })?;

            // Re-open the file for reading
            let file = File::open(&self.path).map_err(|e| {
                Error::io(format!("Failed to re-open CSV file '{}': {}", self.path, e))
            })?;

            ReaderBuilder::new(Arc::new(inferred_schema))
                .with_format(format)
                .with_batch_size(self.batch_size)
                .build(file)
                .map_err(|e| {
                    Error::arrow(format!("Failed to create CSV reader: {}", e))
                })?
        };

        // Get the schema from the reader
        let schema = reader.schema();

        // Read all batches
        let mut batches = Vec::new();
        for batch_result in reader {
            let batch = batch_result.map_err(|e| {
                Error::arrow(format!("Failed to read CSV batch: {}", e))
            })?;
            batches.push(batch);
        }

        if batches.is_empty() {
            return Err(Error::data(format!("CSV file '{}' is empty", self.path)));
        }

        Ok((schema, batches))
    }
}

/// Parquet data source configuration
#[derive(Debug, Clone)]
pub struct ParquetSource {
    /// Path to the Parquet file
    path: String,

    /// Batch size for reading
    batch_size: usize,
}

impl ParquetSource {
    /// Create a new Parquet source
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            batch_size: 8192,
        }
    }

    /// Set the batch size for reading
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }
}

impl DataSource for ParquetSource {
    fn load(&self) -> Result<(Arc<ArrowSchema>, Vec<RecordBatch>)> {
        use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

        // Open the file
        let file = File::open(&self.path).map_err(|e| {
            Error::io(format!("Failed to open Parquet file '{}': {}", self.path, e))
        })?;

        // Create the Parquet reader
        let builder = ParquetRecordBatchReaderBuilder::try_new(file).map_err(|e| {
            Error::arrow(format!("Failed to create Parquet reader: {}", e))
        })?;

        let schema = builder.schema().clone();

        let reader = builder
            .with_batch_size(self.batch_size)
            .build()
            .map_err(|e| {
                Error::arrow(format!("Failed to build Parquet reader: {}", e))
            })?;

        // Read all batches
        let mut batches = Vec::new();
        for batch_result in reader {
            let batch = batch_result.map_err(|e| {
                Error::arrow(format!("Failed to read Parquet batch: {}", e))
            })?;
            batches.push(batch);
        }

        if batches.is_empty() {
            return Err(Error::data(format!("Parquet file '{}' is empty", self.path)));
        }

        Ok((schema, batches))
    }
}

/// JSON data source configuration
#[derive(Debug, Clone)]
pub struct JsonSource {
    /// Path to the JSON file
    path: String,

    /// Batch size for reading
    batch_size: usize,

    /// Optional schema (if None, will be inferred)
    schema: Option<Arc<ArrowSchema>>,
}

impl JsonSource {
    /// Create a new JSON source
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            batch_size: 8192,
            schema: None,
        }
    }

    /// Set the batch size for reading
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }

    /// Set the expected schema
    pub fn with_schema(mut self, schema: Arc<ArrowSchema>) -> Self {
        self.schema = Some(schema);
        self
    }
}

impl DataSource for JsonSource {
    fn load(&self) -> Result<(Arc<ArrowSchema>, Vec<RecordBatch>)> {
        use arrow_json::ReaderBuilder;

        // Open the file with buffered reader
        let file = File::open(&self.path).map_err(|e| {
            Error::io(format!("Failed to open JSON file '{}': {}", self.path, e))
        })?;
        let buf_reader = BufReader::new(file);

        // Build the JSON reader
        let reader = if let Some(schema) = &self.schema {
            ReaderBuilder::new(schema.clone())
                .with_batch_size(self.batch_size)
                .build(buf_reader)
                .map_err(|e| {
                    Error::arrow(format!("Failed to create JSON reader: {}", e))
                })?
        } else {
            // For schema inference, read and infer first
            let file_for_infer = File::open(&self.path).map_err(|e| {
                Error::io(format!("Failed to open JSON file for schema inference '{}': {}", self.path, e))
            })?;
            let buf_reader_infer = BufReader::new(file_for_infer);

            let inferred_result = arrow_json::reader::infer_json_schema(buf_reader_infer, Some(100))
                .map_err(|e| {
                    Error::arrow(format!("Failed to infer JSON schema: {}", e))
                })?;

            // Extract schema from tuple (schema, inferred_rows)
            let inferred_schema = inferred_result.0;

            // Re-open the file for reading data
            let file = File::open(&self.path).map_err(|e| {
                Error::io(format!("Failed to re-open JSON file '{}': {}", self.path, e))
            })?;
            let buf_reader = BufReader::new(file);

            ReaderBuilder::new(Arc::new(inferred_schema))
                .with_batch_size(self.batch_size)
                .build(buf_reader)
                .map_err(|e| {
                    Error::arrow(format!("Failed to create JSON reader: {}", e))
                })?
        };

        let schema = reader.schema();

        // Read all batches
        let mut batches = Vec::new();
        for batch_result in reader {
            let batch = batch_result.map_err(|e| {
                Error::arrow(format!("Failed to read JSON batch: {}", e))
            })?;
            batches.push(batch);
        }

        if batches.is_empty() {
            return Err(Error::data(format!("JSON file '{}' is empty", self.path)));
        }

        Ok((schema, batches))
    }
}

/// In-memory data source from Arrow RecordBatches
#[derive(Debug)]
pub struct RecordBatchSource {
    schema: Arc<ArrowSchema>,
    batches: Vec<RecordBatch>,
}

impl RecordBatchSource {
    /// Create a new in-memory source from RecordBatches
    pub fn new(schema: Arc<ArrowSchema>, batches: Vec<RecordBatch>) -> Result<Self> {
        if batches.is_empty() {
            return Err(Error::data("RecordBatchSource requires at least one batch"));
        }

        // Validate that all batches have the same schema
        for batch in &batches {
            if batch.schema().as_ref() != schema.as_ref() {
                return Err(Error::schema(
                    "All RecordBatches must have the same schema as the provided schema"
                ));
            }
        }

        Ok(Self { schema, batches })
    }
}

impl DataSource for RecordBatchSource {
    fn load(&self) -> Result<(Arc<ArrowSchema>, Vec<RecordBatch>)> {
        Ok((self.schema.clone(), self.batches.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_source_builder() {
        let source = CsvSource::new("test.csv")
            .with_header(true)
            .with_batch_size(1024)
            .with_delimiter(b';');

        assert_eq!(source.path, "test.csv");
        assert_eq!(source.has_header, true);
        assert_eq!(source.batch_size, 1024);
        assert_eq!(source.delimiter, b';');
    }

    #[test]
    fn test_parquet_source_builder() {
        let source = ParquetSource::new("test.parquet")
            .with_batch_size(2048);

        assert_eq!(source.path, "test.parquet");
        assert_eq!(source.batch_size, 2048);
    }

    #[test]
    fn test_json_source_builder() {
        let source = JsonSource::new("test.json")
            .with_batch_size(512);

        assert_eq!(source.path, "test.json");
        assert_eq!(source.batch_size, 512);
    }
}
