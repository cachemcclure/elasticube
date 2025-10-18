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

// ==============================================================================
// Database Sources (via ODBC)
// ==============================================================================

#[cfg(feature = "database")]
pub mod database {
    use super::*;
    use arrow_odbc::OdbcReaderBuilder;
    use arrow_odbc::odbc_api::{Environment, ConnectionOptions};

    /// Configuration for connecting to databases via ODBC
    ///
    /// Supports PostgreSQL, MySQL, SQL Server, SQLite, and any ODBC-compatible database.
    ///
    /// # Example Connection Strings
    ///
    /// **PostgreSQL**:
    /// ```text
    /// Driver={PostgreSQL Unicode};Server=localhost;Port=5432;Database=mydb;Uid=user;Pwd=pass;
    /// ```
    ///
    /// **MySQL**:
    /// ```text
    /// Driver={MySQL ODBC 8.0 Unicode Driver};Server=localhost;Port=3306;Database=mydb;Uid=user;Pwd=pass;
    /// ```
    ///
    /// **SQL Server**:
    /// ```text
    /// Driver={ODBC Driver 17 for SQL Server};Server=localhost;Database=mydb;Uid=user;Pwd=pass;
    /// ```
    #[derive(Debug, Clone)]
    pub struct OdbcSource {
        /// ODBC connection string
        connection_string: String,

        /// SQL query to execute
        query: String,

        /// Maximum bytes per batch (default: 8MB)
        max_bytes_per_batch: usize,

        /// Maximum number of rows to fetch (None = unlimited)
        max_rows: Option<usize>,
    }

    impl OdbcSource {
        /// Create a new ODBC data source
        ///
        /// # Arguments
        /// * `connection_string` - ODBC connection string
        /// * `query` - SQL query to execute
        ///
        /// # Example
        /// ```rust,ignore
        /// let source = OdbcSource::new(
        ///     "Driver={PostgreSQL Unicode};Server=localhost;Database=sales;Uid=user;Pwd=pass",
        ///     "SELECT * FROM transactions WHERE date >= '2025-01-01'"
        /// );
        /// ```
        pub fn new(connection_string: impl Into<String>, query: impl Into<String>) -> Self {
            Self {
                connection_string: connection_string.into(),
                query: query.into(),
                max_bytes_per_batch: 8 * 1024 * 1024, // 8MB default
                max_rows: None,
            }
        }

        /// Set the maximum bytes per batch
        pub fn with_max_bytes_per_batch(mut self, max_bytes: usize) -> Self {
            self.max_bytes_per_batch = max_bytes;
            self
        }

        /// Set maximum number of rows to fetch
        pub fn with_max_rows(mut self, max_rows: usize) -> Self {
            self.max_rows = Some(max_rows);
            self
        }
    }

    impl DataSource for OdbcSource {
        fn load(&self) -> Result<(Arc<ArrowSchema>, Vec<RecordBatch>)> {
            // Create ODBC environment
            let env = Environment::new().map_err(|e| {
                Error::data(format!("Failed to create ODBC environment: {}", e))
            })?;

            // Connect to database
            let conn = env.connect_with_connection_string(
                &self.connection_string,
                ConnectionOptions::default()
            ).map_err(|e| {
                Error::data(format!("Failed to connect to database: {}", e))
            })?;

            // Execute query to get cursor
            // Third parameter is max_rows (None = unlimited)
            let cursor = match conn.execute(&self.query, (), self.max_rows).map_err(|e| {
                Error::data(format!("Failed to execute SQL query: {}", e))
            })? {
                Some(cursor) => cursor,
                None => {
                    return Err(Error::data("SQL query did not return a result set (cursor). Use SELECT statements for data loading."));
                }
            };

            // Build the ODBC reader
            let reader = OdbcReaderBuilder::new()
                .with_max_bytes_per_batch(self.max_bytes_per_batch)
                .build(cursor)
                .map_err(|e| {
                    Error::data(format!("Failed to create ODBC reader: {}", e))
                })?;

            let schema = reader.schema();

            // Read all batches
            // Note: max_rows is already handled by the execute() method above
            let mut batches = Vec::new();

            for batch_result in reader {
                let batch = batch_result.map_err(|e| {
                    Error::arrow(format!("Failed to read ODBC batch: {}", e))
                })?;
                batches.push(batch);
            }

            if batches.is_empty() {
                return Err(Error::data("ODBC query returned no results"));
            }

            Ok((schema, batches))
        }
    }

    /// Convenience wrapper for PostgreSQL connections
    ///
    /// # Example
    /// ```rust,ignore
    /// let source = PostgresSource::new("localhost", "mydb", "user", "pass")
    ///     .with_port(5432)
    ///     .with_query("SELECT * FROM sales");
    /// ```
    #[derive(Debug, Clone)]
    pub struct PostgresSource {
        host: String,
        database: String,
        username: String,
        password: String,
        port: u16,
        query: String,
        max_bytes_per_batch: usize,
    }

    impl PostgresSource {
        /// Create a new PostgreSQL data source
        pub fn new(
            host: impl Into<String>,
            database: impl Into<String>,
            username: impl Into<String>,
            password: impl Into<String>,
        ) -> Self {
            Self {
                host: host.into(),
                database: database.into(),
                username: username.into(),
                password: password.into(),
                port: 5432,
                query: String::new(),
                max_bytes_per_batch: 8 * 1024 * 1024, // 8MB default
            }
        }

        /// Set the port (default: 5432)
        pub fn with_port(mut self, port: u16) -> Self {
            self.port = port;
            self
        }

        /// Set the SQL query to execute
        pub fn with_query(mut self, query: impl Into<String>) -> Self {
            self.query = query.into();
            self
        }

        /// Set the maximum bytes per batch
        pub fn with_max_bytes_per_batch(mut self, max_bytes: usize) -> Self {
            self.max_bytes_per_batch = max_bytes;
            self
        }

        /// Build the ODBC connection string
        pub(crate) fn connection_string(&self) -> String {
            format!(
                "Driver={{PostgreSQL Unicode}};Server={};Port={};Database={};Uid={};Pwd={};",
                self.host, self.port, self.database, self.username, self.password
            )
        }
    }

    impl DataSource for PostgresSource {
        fn load(&self) -> Result<(Arc<ArrowSchema>, Vec<RecordBatch>)> {
            if self.query.is_empty() {
                return Err(Error::data("PostgreSQL query cannot be empty. Use with_query() to set it."));
            }

            let odbc_source = OdbcSource::new(self.connection_string(), &self.query)
                .with_max_bytes_per_batch(self.max_bytes_per_batch);

            odbc_source.load()
        }
    }

    /// Convenience wrapper for MySQL connections
    ///
    /// # Example
    /// ```rust,ignore
    /// let source = MySqlSource::new("localhost", "mydb", "user", "pass")
    ///     .with_port(3306)
    ///     .with_query("SELECT * FROM orders");
    /// ```
    #[derive(Debug, Clone)]
    pub struct MySqlSource {
        host: String,
        database: String,
        username: String,
        password: String,
        port: u16,
        query: String,
        max_bytes_per_batch: usize,
    }

    impl MySqlSource {
        /// Create a new MySQL data source
        pub fn new(
            host: impl Into<String>,
            database: impl Into<String>,
            username: impl Into<String>,
            password: impl Into<String>,
        ) -> Self {
            Self {
                host: host.into(),
                database: database.into(),
                username: username.into(),
                password: password.into(),
                port: 3306,
                query: String::new(),
                max_bytes_per_batch: 8 * 1024 * 1024, // 8MB default
            }
        }

        /// Set the port (default: 3306)
        pub fn with_port(mut self, port: u16) -> Self {
            self.port = port;
            self
        }

        /// Set the SQL query to execute
        pub fn with_query(mut self, query: impl Into<String>) -> Self {
            self.query = query.into();
            self
        }

        /// Set the maximum bytes per batch
        pub fn with_max_bytes_per_batch(mut self, max_bytes: usize) -> Self {
            self.max_bytes_per_batch = max_bytes;
            self
        }

        /// Build the ODBC connection string
        pub(crate) fn connection_string(&self) -> String {
            format!(
                "Driver={{MySQL ODBC 8.0 Unicode Driver}};Server={};Port={};Database={};Uid={};Pwd={};",
                self.host, self.port, self.database, self.username, self.password
            )
        }
    }

    impl DataSource for MySqlSource {
        fn load(&self) -> Result<(Arc<ArrowSchema>, Vec<RecordBatch>)> {
            if self.query.is_empty() {
                return Err(Error::data("MySQL query cannot be empty. Use with_query() to set it."));
            }

            let odbc_source = OdbcSource::new(self.connection_string(), &self.query)
                .with_max_bytes_per_batch(self.max_bytes_per_batch);

            odbc_source.load()
        }
    }
}

// ==============================================================================
// REST API Sources
// ==============================================================================

#[cfg(feature = "rest-api")]
pub mod rest {
    use super::*;
    use reqwest::blocking::Client;
    use std::collections::HashMap;
    use std::io::Cursor;

    /// HTTP method for REST API requests
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum HttpMethod {
        Get,
        Post,
    }

    /// REST API data source that fetches JSON data
    ///
    /// Supports GET and POST requests with optional headers and query parameters.
    /// The response must be JSON (either array of objects or single object).
    ///
    /// # Example
    /// ```rust,ignore
    /// let source = RestApiSource::new("https://api.example.com/sales")
    ///     .with_method(HttpMethod::Get)
    ///     .with_header("Authorization", "Bearer token123")
    ///     .with_query_param("date_from", "2024-01-01");
    /// ```
    #[derive(Debug, Clone)]
    pub struct RestApiSource {
        /// Base URL for the API endpoint
        url: String,

        /// HTTP method (GET or POST)
        method: HttpMethod,

        /// HTTP headers
        headers: HashMap<String, String>,

        /// Query parameters (for GET requests)
        query_params: HashMap<String, String>,

        /// Request body (for POST requests)
        body: Option<String>,

        /// Batch size for reading
        batch_size: usize,

        /// Optional schema (if None, will be inferred from JSON)
        schema: Option<Arc<ArrowSchema>>,

        /// Timeout in seconds (default: 30)
        timeout_secs: u64,
    }

    impl RestApiSource {
        /// Create a new REST API data source
        pub fn new(url: impl Into<String>) -> Self {
            Self {
                url: url.into(),
                method: HttpMethod::Get,
                headers: HashMap::new(),
                query_params: HashMap::new(),
                body: None,
                batch_size: 8192,
                schema: None,
                timeout_secs: 30,
            }
        }

        /// Set the HTTP method
        pub fn with_method(mut self, method: HttpMethod) -> Self {
            self.method = method;
            self
        }

        /// Add an HTTP header
        pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
            self.headers.insert(key.into(), value.into());
            self
        }

        /// Add a query parameter
        pub fn with_query_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
            self.query_params.insert(key.into(), value.into());
            self
        }

        /// Set the request body (for POST requests)
        pub fn with_body(mut self, body: impl Into<String>) -> Self {
            self.body = Some(body.into());
            self
        }

        /// Set the batch size
        pub fn with_batch_size(mut self, batch_size: usize) -> Self {
            self.batch_size = batch_size;
            self
        }

        /// Set the expected schema
        pub fn with_schema(mut self, schema: Arc<ArrowSchema>) -> Self {
            self.schema = Some(schema);
            self
        }

        /// Set the request timeout in seconds
        pub fn with_timeout_secs(mut self, timeout_secs: u64) -> Self {
            self.timeout_secs = timeout_secs;
            self
        }
    }

    impl DataSource for RestApiSource {
        fn load(&self) -> Result<(Arc<ArrowSchema>, Vec<RecordBatch>)> {
            use arrow_json::ReaderBuilder;

            // Build the HTTP client
            let client = Client::builder()
                .timeout(std::time::Duration::from_secs(self.timeout_secs))
                .build()
                .map_err(|e| Error::io(format!("Failed to create HTTP client: {}", e)))?;

            // Build the URL with query parameters
            let mut url = url::Url::parse(&self.url)
                .map_err(|e| Error::data(format!("Invalid URL '{}': {}", self.url, e)))?;

            for (key, value) in &self.query_params {
                url.query_pairs_mut().append_pair(key, value);
            }

            // Build the request
            let mut request = match self.method {
                HttpMethod::Get => client.get(url.as_str()),
                HttpMethod::Post => {
                    let mut req = client.post(url.as_str());
                    if let Some(body) = &self.body {
                        req = req.body(body.clone());
                    }
                    req
                }
            };

            // Add headers
            for (key, value) in &self.headers {
                request = request.header(key, value);
            }

            // Execute the request
            let response = request
                .send()
                .map_err(|e| Error::io(format!("HTTP request failed: {}", e)))?;

            // Check status
            if !response.status().is_success() {
                return Err(Error::data(format!(
                    "HTTP request failed with status {}: {}",
                    response.status(),
                    response.text().unwrap_or_default()
                )));
            }

            // Get the response body as bytes
            let response_bytes = response
                .bytes()
                .map_err(|e| Error::io(format!("Failed to read HTTP response: {}", e)))?;

            // Parse JSON and convert to Arrow RecordBatch
            let cursor = Cursor::new(response_bytes.as_ref());

            // Build the JSON reader
            let reader = if let Some(schema) = &self.schema {
                ReaderBuilder::new(schema.clone())
                    .with_batch_size(self.batch_size)
                    .build(cursor)
                    .map_err(|e| Error::arrow(format!("Failed to create JSON reader: {}", e)))?
            } else {
                // Infer schema from JSON
                let cursor_for_infer = Cursor::new(response_bytes.as_ref());
                let inferred_result = arrow_json::reader::infer_json_schema(cursor_for_infer, None)
                    .map_err(|e| Error::arrow(format!("Failed to infer JSON schema from API response: {}", e)))?;

                let inferred_schema = inferred_result.0;
                let cursor = Cursor::new(response_bytes.as_ref());

                ReaderBuilder::new(Arc::new(inferred_schema))
                    .with_batch_size(self.batch_size)
                    .build(cursor)
                    .map_err(|e| Error::arrow(format!("Failed to create JSON reader: {}", e)))?
            };

            let schema = reader.schema();

            // Read all batches
            let mut batches = Vec::new();
            for batch_result in reader {
                let batch = batch_result.map_err(|e| {
                    Error::arrow(format!("Failed to read JSON batch from API response: {}", e))
                })?;
                batches.push(batch);
            }

            if batches.is_empty() {
                return Err(Error::data(format!("API response from '{}' is empty", self.url)));
            }

            Ok((schema, batches))
        }
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

    #[cfg(feature = "database")]
    #[test]
    fn test_postgres_source_builder() {
        let source = database::PostgresSource::new("localhost", "testdb", "user", "pass")
            .with_port(5432)
            .with_query("SELECT * FROM test");

        // Test connection string generation
        assert_eq!(source.connection_string(),
            "Driver={PostgreSQL Unicode};Server=localhost;Port=5432;Database=testdb;Uid=user;Pwd=pass;");
    }

    #[cfg(feature = "database")]
    #[test]
    fn test_mysql_source_builder() {
        let source = database::MySqlSource::new("localhost", "testdb", "user", "pass")
            .with_port(3306)
            .with_query("SELECT * FROM test");

        // Test connection string generation
        assert_eq!(source.connection_string(),
            "Driver={MySQL ODBC 8.0 Unicode Driver};Server=localhost;Port=3306;Database=testdb;Uid=user;Pwd=pass;");
    }

    #[cfg(feature = "rest-api")]
    #[test]
    fn test_rest_api_source_builder() {
        // Just test that the builder pattern works without errors
        let _source = rest::RestApiSource::new("https://api.example.com/data")
            .with_method(rest::HttpMethod::Get)
            .with_header("Authorization", "Bearer token")
            .with_query_param("limit", "100")
            .with_batch_size(512)
            .with_timeout_secs(60);

        // Builder pattern works - source created successfully
        assert!(true);
    }

    #[cfg(feature = "object-storage")]
    #[test]
    fn test_s3_source_builder() {
        use object_storage::{S3Source, StorageFileFormat};

        let source = S3Source::new("my-bucket", "data/sales.parquet")
            .with_region("us-west-2")
            .with_format(StorageFileFormat::Parquet)
            .with_batch_size(4096);

        // Builder pattern works - source created successfully
        assert!(true);
    }

    #[cfg(feature = "object-storage")]
    #[test]
    fn test_gcs_source_builder() {
        use object_storage::{GcsSource, StorageFileFormat};

        let source = GcsSource::new("my-bucket", "data/analytics.json")
            .with_format(StorageFileFormat::Json)
            .with_batch_size(8192);

        // Builder pattern works - source created successfully
        assert!(true);
    }

    #[cfg(feature = "object-storage")]
    #[test]
    fn test_azure_source_builder() {
        use object_storage::{AzureSource, StorageFileFormat};

        let source = AzureSource::new("myaccount", "mycontainer", "data/logs.csv")
            .with_format(StorageFileFormat::Csv)
            .with_batch_size(2048);

        // Builder pattern works - source created successfully
        assert!(true);
    }
}

// ==============================================================================
// Object Storage Sources (S3, GCS, Azure)
// ==============================================================================

#[cfg(feature = "object-storage")]
pub mod object_storage {
    use super::*;
    use bytes::Bytes;
    use object_store::{ObjectStore, path::Path as ObjectPath};
    use std::sync::Arc as StdArc;

    /// File format for object storage files
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum StorageFileFormat {
        /// Parquet format
        Parquet,
        /// CSV format
        Csv,
        /// JSON format (newline-delimited JSON)
        Json,
    }

    /// Generic object storage source that works with any ObjectStore backend
    ///
    /// This can be used with S3, GCS, Azure, or local file system via object_store.
    ///
    /// # Example
    /// ```rust,ignore
    /// use object_store::aws::AmazonS3Builder;
    ///
    /// let store = AmazonS3Builder::new()
    ///     .with_bucket_name("my-bucket")
    ///     .with_region("us-west-2")
    ///     .build()?;
    ///
    /// let source = ObjectStorageSource::new(store, "data/sales.parquet")
    ///     .with_format(StorageFileFormat::Parquet);
    /// ```
    #[derive(Debug)]
    pub struct ObjectStorageSource {
        /// Object store instance
        store: StdArc<dyn ObjectStore>,

        /// Path to the file in the object store
        path: String,

        /// File format
        format: StorageFileFormat,

        /// Batch size for reading
        batch_size: usize,

        /// Optional schema for CSV/JSON
        schema: Option<Arc<ArrowSchema>>,

        /// CSV-specific: has header row
        csv_has_header: bool,

        /// CSV-specific: delimiter
        csv_delimiter: u8,
    }

    impl ObjectStorageSource {
        /// Create a new object storage source
        ///
        /// # Arguments
        /// * `store` - ObjectStore instance (S3, GCS, Azure, etc.)
        /// * `path` - Path to the file in the object store
        pub fn new(store: StdArc<dyn ObjectStore>, path: impl Into<String>) -> Self {
            Self {
                store,
                path: path.into(),
                format: StorageFileFormat::Parquet,
                batch_size: 8192,
                schema: None,
                csv_has_header: true,
                csv_delimiter: b',',
            }
        }

        /// Set the file format
        pub fn with_format(mut self, format: StorageFileFormat) -> Self {
            self.format = format;
            self
        }

        /// Set the batch size
        pub fn with_batch_size(mut self, batch_size: usize) -> Self {
            self.batch_size = batch_size;
            self
        }

        /// Set the schema (for CSV/JSON)
        pub fn with_schema(mut self, schema: Arc<ArrowSchema>) -> Self {
            self.schema = Some(schema);
            self
        }

        /// Set CSV header flag
        pub fn with_csv_header(mut self, has_header: bool) -> Self {
            self.csv_has_header = has_header;
            self
        }

        /// Set CSV delimiter
        pub fn with_csv_delimiter(mut self, delimiter: u8) -> Self {
            self.csv_delimiter = delimiter;
            self
        }

        /// Download the file from object storage
        async fn download_file(&self) -> Result<Bytes> {
            let path = ObjectPath::from(self.path.as_str());

            // Use get() to fetch the entire object
            let result = self.store.get(&path).await.map_err(|e| {
                Error::io(format!("Failed to download file '{}' from object storage: {}", self.path, e))
            })?;

            // Read all bytes
            let bytes = result.bytes().await.map_err(|e| {
                Error::io(format!("Failed to read bytes from object storage: {}", e))
            })?;

            Ok(bytes)
        }
    }

    impl DataSource for ObjectStorageSource {
        fn load(&self) -> Result<(Arc<ArrowSchema>, Vec<RecordBatch>)> {
            // Use tokio runtime to run async code
            let runtime = tokio::runtime::Runtime::new().map_err(|e| {
                Error::io(format!("Failed to create tokio runtime: {}", e))
            })?;

            runtime.block_on(async {
                // Download the file
                let bytes = self.download_file().await?;

                // Parse based on format
                match self.format {
                    StorageFileFormat::Parquet => {
                        use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

                        // ParquetRecordBatchReaderBuilder requires a type that implements ChunkReader
                        // Bytes implements ChunkReader directly, so we don't need Cursor
                        let builder = ParquetRecordBatchReaderBuilder::try_new(bytes.clone()).map_err(|e| {
                            Error::arrow(format!("Failed to create Parquet reader: {}", e))
                        })?;

                        let schema = builder.schema().clone();
                        let reader = builder.with_batch_size(self.batch_size).build().map_err(|e| {
                            Error::arrow(format!("Failed to build Parquet reader: {}", e))
                        })?;

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

                    StorageFileFormat::Csv => {
                        use arrow_csv::ReaderBuilder;
                        use std::io::Cursor;

                        let format = arrow_csv::reader::Format::default()
                            .with_header(self.csv_has_header)
                            .with_delimiter(self.csv_delimiter);

                        let reader = if let Some(schema) = &self.schema {
                            let cursor = Cursor::new(bytes);
                            ReaderBuilder::new(schema.clone())
                                .with_format(format)
                                .with_batch_size(self.batch_size)
                                .build(cursor)
                                .map_err(|e| Error::arrow(format!("Failed to create CSV reader: {}", e)))?
                        } else {
                            // Infer schema
                            let cursor_for_infer = Cursor::new(bytes.clone());
                            let buf_reader = BufReader::new(cursor_for_infer);
                            let (inferred_schema, _) = format.infer_schema(buf_reader, Some(100))
                                .map_err(|e| Error::arrow(format!("Failed to infer CSV schema: {}", e)))?;

                            let cursor = Cursor::new(bytes);
                            ReaderBuilder::new(Arc::new(inferred_schema))
                                .with_format(format)
                                .with_batch_size(self.batch_size)
                                .build(cursor)
                                .map_err(|e| Error::arrow(format!("Failed to create CSV reader: {}", e)))?
                        };

                        let schema = reader.schema();
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

                    StorageFileFormat::Json => {
                        use arrow_json::ReaderBuilder;
                        use std::io::Cursor;

                        let cursor = Cursor::new(bytes.clone());

                        let reader = if let Some(schema) = &self.schema {
                            ReaderBuilder::new(schema.clone())
                                .with_batch_size(self.batch_size)
                                .build(cursor)
                                .map_err(|e| Error::arrow(format!("Failed to create JSON reader: {}", e)))?
                        } else {
                            // Infer schema
                            let cursor_for_infer = Cursor::new(bytes.clone());
                            let buf_reader = BufReader::new(cursor_for_infer);
                            let inferred_result = arrow_json::reader::infer_json_schema(buf_reader, Some(100))
                                .map_err(|e| Error::arrow(format!("Failed to infer JSON schema: {}", e)))?;

                            let inferred_schema = inferred_result.0;
                            let cursor = Cursor::new(bytes);
                            ReaderBuilder::new(Arc::new(inferred_schema))
                                .with_batch_size(self.batch_size)
                                .build(cursor)
                                .map_err(|e| Error::arrow(format!("Failed to create JSON reader: {}", e)))?
                        };

                        let schema = reader.schema();
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
            })
        }
    }

    /// AWS S3 data source
    ///
    /// # Example
    /// ```rust,ignore
    /// let source = S3Source::new("my-bucket", "data/sales.parquet")
    ///     .with_region("us-west-2")
    ///     .with_format(StorageFileFormat::Parquet);
    /// ```
    #[derive(Debug, Clone)]
    pub struct S3Source {
        bucket: String,
        path: String,
        region: Option<String>,
        access_key_id: Option<String>,
        secret_access_key: Option<String>,
        endpoint: Option<String>,
        format: StorageFileFormat,
        batch_size: usize,
        schema: Option<Arc<ArrowSchema>>,
    }

    impl S3Source {
        /// Create a new S3 data source
        ///
        /// # Arguments
        /// * `bucket` - S3 bucket name
        /// * `path` - Path to the file in the bucket (e.g., "data/sales.parquet")
        ///
        /// # Authentication
        /// By default, uses AWS credentials from environment variables or ~/.aws/credentials.
        /// Use `with_access_key()` to provide explicit credentials.
        pub fn new(bucket: impl Into<String>, path: impl Into<String>) -> Self {
            Self {
                bucket: bucket.into(),
                path: path.into(),
                region: None,
                access_key_id: None,
                secret_access_key: None,
                endpoint: None,
                format: StorageFileFormat::Parquet,
                batch_size: 8192,
                schema: None,
            }
        }

        /// Set the AWS region (e.g., "us-west-2")
        pub fn with_region(mut self, region: impl Into<String>) -> Self {
            self.region = Some(region.into());
            self
        }

        /// Set explicit AWS credentials
        pub fn with_access_key(
            mut self,
            access_key_id: impl Into<String>,
            secret_access_key: impl Into<String>,
        ) -> Self {
            self.access_key_id = Some(access_key_id.into());
            self.secret_access_key = Some(secret_access_key.into());
            self
        }

        /// Set custom S3 endpoint (for S3-compatible services like MinIO)
        pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
            self.endpoint = Some(endpoint.into());
            self
        }

        /// Set the file format
        pub fn with_format(mut self, format: StorageFileFormat) -> Self {
            self.format = format;
            self
        }

        /// Set the batch size
        pub fn with_batch_size(mut self, batch_size: usize) -> Self {
            self.batch_size = batch_size;
            self
        }

        /// Set the schema (for CSV/JSON)
        pub fn with_schema(mut self, schema: Arc<ArrowSchema>) -> Self {
            self.schema = Some(schema);
            self
        }

        /// Build the ObjectStore instance
        fn build_store(&self) -> Result<StdArc<dyn ObjectStore>> {
            use object_store::aws::AmazonS3Builder;

            let mut builder = AmazonS3Builder::new()
                .with_bucket_name(&self.bucket);

            if let Some(region) = &self.region {
                builder = builder.with_region(region);
            }

            if let Some(access_key_id) = &self.access_key_id {
                builder = builder.with_access_key_id(access_key_id);
            }

            if let Some(secret_access_key) = &self.secret_access_key {
                builder = builder.with_secret_access_key(secret_access_key);
            }

            if let Some(endpoint) = &self.endpoint {
                builder = builder.with_endpoint(endpoint);
            }

            let store = builder.build().map_err(|e| {
                Error::data(format!("Failed to build S3 store: {}", e))
            })?;

            Ok(StdArc::new(store))
        }
    }

    impl DataSource for S3Source {
        fn load(&self) -> Result<(Arc<ArrowSchema>, Vec<RecordBatch>)> {
            let store = self.build_store()?;

            let mut obj_source = ObjectStorageSource::new(store, &self.path)
                .with_format(self.format)
                .with_batch_size(self.batch_size);

            if let Some(schema) = &self.schema {
                obj_source = obj_source.with_schema(schema.clone());
            }

            obj_source.load()
        }
    }

    /// Google Cloud Storage (GCS) data source
    ///
    /// # Example
    /// ```rust,ignore
    /// let source = GcsSource::new("my-bucket", "data/sales.parquet")
    ///     .with_service_account_key("path/to/key.json")
    ///     .with_format(StorageFileFormat::Parquet);
    /// ```
    #[derive(Debug, Clone)]
    pub struct GcsSource {
        bucket: String,
        path: String,
        service_account_key: Option<String>,
        format: StorageFileFormat,
        batch_size: usize,
        schema: Option<Arc<ArrowSchema>>,
    }

    impl GcsSource {
        /// Create a new GCS data source
        ///
        /// # Arguments
        /// * `bucket` - GCS bucket name
        /// * `path` - Path to the file in the bucket
        ///
        /// # Authentication
        /// By default, uses Google Cloud credentials from GOOGLE_APPLICATION_CREDENTIALS env var.
        /// Use `with_service_account_key()` to provide explicit credentials.
        pub fn new(bucket: impl Into<String>, path: impl Into<String>) -> Self {
            Self {
                bucket: bucket.into(),
                path: path.into(),
                service_account_key: None,
                format: StorageFileFormat::Parquet,
                batch_size: 8192,
                schema: None,
            }
        }

        /// Set the service account key path or JSON content
        pub fn with_service_account_key(mut self, key: impl Into<String>) -> Self {
            self.service_account_key = Some(key.into());
            self
        }

        /// Set the file format
        pub fn with_format(mut self, format: StorageFileFormat) -> Self {
            self.format = format;
            self
        }

        /// Set the batch size
        pub fn with_batch_size(mut self, batch_size: usize) -> Self {
            self.batch_size = batch_size;
            self
        }

        /// Set the schema (for CSV/JSON)
        pub fn with_schema(mut self, schema: Arc<ArrowSchema>) -> Self {
            self.schema = Some(schema);
            self
        }

        /// Build the ObjectStore instance
        fn build_store(&self) -> Result<StdArc<dyn ObjectStore>> {
            use object_store::gcp::GoogleCloudStorageBuilder;

            let mut builder = GoogleCloudStorageBuilder::new()
                .with_bucket_name(&self.bucket);

            if let Some(key) = &self.service_account_key {
                builder = builder.with_service_account_key(key);
            }

            let store = builder.build().map_err(|e| {
                Error::data(format!("Failed to build GCS store: {}", e))
            })?;

            Ok(StdArc::new(store))
        }
    }

    impl DataSource for GcsSource {
        fn load(&self) -> Result<(Arc<ArrowSchema>, Vec<RecordBatch>)> {
            let store = self.build_store()?;

            let mut obj_source = ObjectStorageSource::new(store, &self.path)
                .with_format(self.format)
                .with_batch_size(self.batch_size);

            if let Some(schema) = &self.schema {
                obj_source = obj_source.with_schema(schema.clone());
            }

            obj_source.load()
        }
    }

    /// Azure Blob Storage data source
    ///
    /// # Example
    /// ```rust,ignore
    /// let source = AzureSource::new("myaccount", "mycontainer", "data/sales.parquet")
    ///     .with_access_key("access_key")
    ///     .with_format(StorageFileFormat::Parquet);
    /// ```
    #[derive(Debug, Clone)]
    pub struct AzureSource {
        account: String,
        container: String,
        path: String,
        access_key: Option<String>,
        sas_token: Option<String>,
        format: StorageFileFormat,
        batch_size: usize,
        schema: Option<Arc<ArrowSchema>>,
    }

    impl AzureSource {
        /// Create a new Azure Blob Storage data source
        ///
        /// # Arguments
        /// * `account` - Azure storage account name
        /// * `container` - Container name
        /// * `path` - Path to the file in the container
        ///
        /// # Authentication
        /// Use `with_access_key()` or `with_sas_token()` to provide credentials.
        pub fn new(
            account: impl Into<String>,
            container: impl Into<String>,
            path: impl Into<String>,
        ) -> Self {
            Self {
                account: account.into(),
                container: container.into(),
                path: path.into(),
                access_key: None,
                sas_token: None,
                format: StorageFileFormat::Parquet,
                batch_size: 8192,
                schema: None,
            }
        }

        /// Set the access key for authentication
        pub fn with_access_key(mut self, access_key: impl Into<String>) -> Self {
            self.access_key = Some(access_key.into());
            self
        }

        /// Set the SAS token for authentication
        pub fn with_sas_token(mut self, sas_token: impl Into<String>) -> Self {
            self.sas_token = Some(sas_token.into());
            self
        }

        /// Set the file format
        pub fn with_format(mut self, format: StorageFileFormat) -> Self {
            self.format = format;
            self
        }

        /// Set the batch size
        pub fn with_batch_size(mut self, batch_size: usize) -> Self {
            self.batch_size = batch_size;
            self
        }

        /// Set the schema (for CSV/JSON)
        pub fn with_schema(mut self, schema: Arc<ArrowSchema>) -> Self {
            self.schema = Some(schema);
            self
        }

        /// Build the ObjectStore instance
        fn build_store(&self) -> Result<StdArc<dyn ObjectStore>> {
            use object_store::azure::{MicrosoftAzureBuilder, AzureConfigKey};

            let mut builder = MicrosoftAzureBuilder::new()
                .with_account(&self.account)
                .with_container_name(&self.container);

            if let Some(access_key) = &self.access_key {
                builder = builder.with_access_key(access_key);
            }

            if let Some(sas_token) = &self.sas_token {
                // SAS token is set using with_config method
                builder = builder.with_config(AzureConfigKey::SasKey, sas_token);
            }

            let store = builder.build().map_err(|e| {
                Error::data(format!("Failed to build Azure store: {}", e))
            })?;

            Ok(StdArc::new(store))
        }
    }

    impl DataSource for AzureSource {
        fn load(&self) -> Result<(Arc<ArrowSchema>, Vec<RecordBatch>)> {
            let store = self.build_store()?;

            let mut obj_source = ObjectStorageSource::new(store, &self.path)
                .with_format(self.format)
                .with_batch_size(self.batch_size);

            if let Some(schema) = &self.schema {
                obj_source = obj_source.with_schema(schema.clone());
            }

            obj_source.load()
        }
    }
}
