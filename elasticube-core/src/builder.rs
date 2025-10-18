//! ElastiCube builder for constructing cubes

use crate::cube::{
    AggFunc, CalculatedMeasure, CubeSchema, Dimension, ElastiCube, Hierarchy, Measure,
    VirtualDimension,
};
use crate::error::{Error, Result};
use crate::sources::{CsvSource, DataSource, JsonSource, ParquetSource, RecordBatchSource};
use arrow::datatypes::{DataType, Schema as ArrowSchema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

/// Builder for constructing an ElastiCube
///
/// Provides a fluent API for defining dimensions, measures, hierarchies,
/// and loading data from various sources.
#[derive(Debug)]
pub struct ElastiCubeBuilder {
    schema: CubeSchema,
    data_source: Option<Box<dyn DataSource>>,
}

impl ElastiCubeBuilder {
    /// Create a new builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            schema: CubeSchema::new(name),
            data_source: None,
        }
    }

    /// Add a dimension
    pub fn add_dimension(
        mut self,
        name: impl Into<String>,
        data_type: DataType,
    ) -> Result<Self> {
        let dimension = Dimension::new(name, data_type);
        self.schema.add_dimension(dimension)?;
        Ok(self)
    }

    /// Add a measure
    pub fn add_measure(
        mut self,
        name: impl Into<String>,
        data_type: DataType,
        agg_func: AggFunc,
    ) -> Result<Self> {
        let measure = Measure::new(name, data_type, agg_func);
        self.schema.add_measure(measure)?;
        Ok(self)
    }

    /// Add a hierarchy
    pub fn add_hierarchy(
        mut self,
        name: impl Into<String>,
        levels: Vec<String>,
    ) -> Result<Self> {
        let hierarchy = Hierarchy::new(name, levels);
        self.schema.add_hierarchy(hierarchy)?;
        Ok(self)
    }

    /// Add a calculated measure (derived from an expression)
    ///
    /// # Arguments
    /// * `name` - Name for the calculated measure
    /// * `expression` - SQL expression (e.g., "revenue - cost")
    /// * `data_type` - Expected result data type
    /// * `agg_func` - Default aggregation function
    ///
    /// # Example
    /// ```rust,ignore
    /// let cube = ElastiCubeBuilder::new("sales")
    ///     .add_measure("revenue", DataType::Float64, AggFunc::Sum)?
    ///     .add_measure("cost", DataType::Float64, AggFunc::Sum)?
    ///     .add_calculated_measure(
    ///         "profit",
    ///         "revenue - cost",
    ///         DataType::Float64,
    ///         AggFunc::Sum
    ///     )?
    ///     .build()?;
    /// ```
    pub fn add_calculated_measure(
        mut self,
        name: impl Into<String>,
        expression: impl Into<String>,
        data_type: DataType,
        agg_func: AggFunc,
    ) -> Result<Self> {
        let calc_measure = CalculatedMeasure::new(name, expression, data_type, agg_func)?;
        self.schema.add_calculated_measure(calc_measure)?;
        Ok(self)
    }

    /// Add a virtual dimension (computed dimension)
    ///
    /// # Arguments
    /// * `name` - Name for the virtual dimension
    /// * `expression` - SQL expression (e.g., "EXTRACT(YEAR FROM date)")
    /// * `data_type` - Expected result data type
    ///
    /// # Example
    /// ```rust,ignore
    /// let cube = ElastiCubeBuilder::new("sales")
    ///     .add_dimension("sale_date", DataType::Date32)?
    ///     .add_virtual_dimension(
    ///         "year",
    ///         "EXTRACT(YEAR FROM sale_date)",
    ///         DataType::Int32
    ///     )?
    ///     .build()?;
    /// ```
    pub fn add_virtual_dimension(
        mut self,
        name: impl Into<String>,
        expression: impl Into<String>,
        data_type: DataType,
    ) -> Result<Self> {
        let virtual_dim = VirtualDimension::new(name, expression, data_type)?;
        self.schema.add_virtual_dimension(virtual_dim)?;
        Ok(self)
    }

    /// Set the cube description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.schema.set_description(description);
        self
    }

    /// Load data from a CSV file
    ///
    /// # Arguments
    /// * `path` - Path to the CSV file
    ///
    /// # Example
    /// ```rust,ignore
    /// let cube = ElastiCubeBuilder::new("sales")
    ///     .load_csv("data.csv")?
    ///     .build()?;
    /// ```
    pub fn load_csv(mut self, path: impl Into<String>) -> Self {
        let source = CsvSource::new(path);
        self.data_source = Some(Box::new(source));
        self
    }

    /// Load data from a CSV file with custom configuration
    ///
    /// # Arguments
    /// * `source` - Configured CsvSource
    ///
    /// # Example
    /// ```rust,ignore
    /// let source = CsvSource::new("data.csv")
    ///     .with_delimiter(b';')
    ///     .with_batch_size(4096);
    /// let cube = ElastiCubeBuilder::new("sales")
    ///     .load_csv_with(source)
    ///     .build()?;
    /// ```
    pub fn load_csv_with(mut self, source: CsvSource) -> Self {
        self.data_source = Some(Box::new(source));
        self
    }

    /// Load data from a Parquet file
    ///
    /// # Arguments
    /// * `path` - Path to the Parquet file
    pub fn load_parquet(mut self, path: impl Into<String>) -> Self {
        let source = ParquetSource::new(path);
        self.data_source = Some(Box::new(source));
        self
    }

    /// Load data from a Parquet file with custom configuration
    pub fn load_parquet_with(mut self, source: ParquetSource) -> Self {
        self.data_source = Some(Box::new(source));
        self
    }

    /// Load data from a JSON file
    ///
    /// # Arguments
    /// * `path` - Path to the JSON file
    pub fn load_json(mut self, path: impl Into<String>) -> Self {
        let source = JsonSource::new(path);
        self.data_source = Some(Box::new(source));
        self
    }

    /// Load data from a JSON file with custom configuration
    pub fn load_json_with(mut self, source: JsonSource) -> Self {
        self.data_source = Some(Box::new(source));
        self
    }

    /// Load data from Arrow RecordBatches
    ///
    /// # Arguments
    /// * `schema` - Arrow schema for the batches
    /// * `batches` - Vector of RecordBatches containing the data
    pub fn load_record_batches(
        mut self,
        schema: Arc<ArrowSchema>,
        batches: Vec<RecordBatch>,
    ) -> Result<Self> {
        let source = RecordBatchSource::new(schema, batches)?;
        self.data_source = Some(Box::new(source));
        Ok(self)
    }

    /// Load data from RecordBatches (convenience method for testing)
    ///
    /// Infers schema from the first batch. All batches must have the same schema.
    ///
    /// # Arguments
    /// * `batches` - Vector of RecordBatches containing the data
    ///
    /// # Example
    /// ```rust,ignore
    /// let batch = RecordBatch::try_new(schema, columns)?;
    /// let cube = ElastiCubeBuilder::new("test")
    ///     .with_data(vec![batch])?
    ///     .build()?;
    /// ```
    pub fn with_data(mut self, batches: Vec<RecordBatch>) -> Result<Self> {
        if batches.is_empty() {
            return Err(Error::builder("Cannot load empty batch vector"));
        }

        let schema = batches[0].schema();
        let source = RecordBatchSource::new(schema, batches)?;
        self.data_source = Some(Box::new(source));
        Ok(self)
    }

    // ==============================================================================
    // Database Sources (available with "database" feature)
    // ==============================================================================

    /// Load data from PostgreSQL database
    ///
    /// Requires the "database" feature to be enabled.
    ///
    /// # Arguments
    /// * `host` - Database host (e.g., "localhost")
    /// * `database` - Database name
    /// * `username` - Username for authentication
    /// * `password` - Password for authentication
    /// * `query` - SQL query to execute
    ///
    /// # Example
    /// ```rust,ignore
    /// let cube = ElastiCubeBuilder::new("sales")
    ///     .load_postgres("localhost", "mydb", "user", "pass", "SELECT * FROM sales")?
    ///     .build()?;
    /// ```
    #[cfg(feature = "database")]
    pub fn load_postgres(
        mut self,
        host: impl Into<String>,
        database: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
        query: impl Into<String>,
    ) -> Self {
        use crate::sources::database::PostgresSource;
        let source = PostgresSource::new(host, database, username, password)
            .with_query(query);
        self.data_source = Some(Box::new(source));
        self
    }

    /// Load data from PostgreSQL with custom configuration
    ///
    /// Requires the "database" feature to be enabled.
    ///
    /// # Example
    /// ```rust,ignore
    /// let source = PostgresSource::new("localhost", "mydb", "user", "pass")
    ///     .with_port(5433)
    ///     .with_query("SELECT * FROM sales WHERE year = 2024")
    ///     .with_batch_size(4096);
    ///
    /// let cube = ElastiCubeBuilder::new("sales")
    ///     .load_postgres_with(source)
    ///     .build()?;
    /// ```
    #[cfg(feature = "database")]
    pub fn load_postgres_with(mut self, source: crate::sources::database::PostgresSource) -> Self {
        self.data_source = Some(Box::new(source));
        self
    }

    /// Load data from MySQL database
    ///
    /// Requires the "database" feature to be enabled.
    ///
    /// # Arguments
    /// * `host` - Database host (e.g., "localhost")
    /// * `database` - Database name
    /// * `username` - Username for authentication
    /// * `password` - Password for authentication
    /// * `query` - SQL query to execute
    ///
    /// # Example
    /// ```rust,ignore
    /// let cube = ElastiCubeBuilder::new("orders")
    ///     .load_mysql("localhost", "mydb", "user", "pass", "SELECT * FROM orders")?
    ///     .build()?;
    /// ```
    #[cfg(feature = "database")]
    pub fn load_mysql(
        mut self,
        host: impl Into<String>,
        database: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
        query: impl Into<String>,
    ) -> Self {
        use crate::sources::database::MySqlSource;
        let source = MySqlSource::new(host, database, username, password)
            .with_query(query);
        self.data_source = Some(Box::new(source));
        self
    }

    /// Load data from MySQL with custom configuration
    ///
    /// Requires the "database" feature to be enabled.
    #[cfg(feature = "database")]
    pub fn load_mysql_with(mut self, source: crate::sources::database::MySqlSource) -> Self {
        self.data_source = Some(Box::new(source));
        self
    }

    /// Load data via generic ODBC connection
    ///
    /// Supports any ODBC-compatible database (PostgreSQL, MySQL, SQL Server, SQLite, etc.).
    /// Requires the "database" feature to be enabled.
    ///
    /// # Arguments
    /// * `connection_string` - ODBC connection string
    /// * `query` - SQL query to execute
    ///
    /// # Example Connection Strings
    ///
    /// **PostgreSQL**:
    /// ```text
    /// Driver={PostgreSQL Unicode};Server=localhost;Port=5432;Database=mydb;Uid=user;Pwd=pass;
    /// ```
    ///
    /// **SQL Server**:
    /// ```text
    /// Driver={ODBC Driver 17 for SQL Server};Server=localhost;Database=mydb;Uid=user;Pwd=pass;
    /// ```
    ///
    /// # Example
    /// ```rust,ignore
    /// let cube = ElastiCubeBuilder::new("analytics")
    ///     .load_odbc(
    ///         "Driver={PostgreSQL Unicode};Server=localhost;Database=analytics;Uid=admin;Pwd=secret;",
    ///         "SELECT * FROM metrics WHERE date >= '2024-01-01'"
    ///     )
    ///     .build()?;
    /// ```
    #[cfg(feature = "database")]
    pub fn load_odbc(
        mut self,
        connection_string: impl Into<String>,
        query: impl Into<String>,
    ) -> Self {
        use crate::sources::database::OdbcSource;
        let source = OdbcSource::new(connection_string, query);
        self.data_source = Some(Box::new(source));
        self
    }

    /// Load data via ODBC with custom configuration
    ///
    /// Requires the "database" feature to be enabled.
    #[cfg(feature = "database")]
    pub fn load_odbc_with(mut self, source: crate::sources::database::OdbcSource) -> Self {
        self.data_source = Some(Box::new(source));
        self
    }

    // ==============================================================================
    // REST API Sources (available with "rest-api" feature)
    // ==============================================================================

    /// Load data from a REST API endpoint
    ///
    /// Requires the "rest-api" feature to be enabled.
    /// The API must return JSON data (either an array of objects or a single object).
    ///
    /// # Arguments
    /// * `url` - API endpoint URL
    ///
    /// # Example
    /// ```rust,ignore
    /// let cube = ElastiCubeBuilder::new("api_data")
    ///     .load_rest_api("https://api.example.com/sales")
    ///     .build()?;
    /// ```
    #[cfg(feature = "rest-api")]
    pub fn load_rest_api(mut self, url: impl Into<String>) -> Self {
        use crate::sources::rest::RestApiSource;
        let source = RestApiSource::new(url);
        self.data_source = Some(Box::new(source));
        self
    }

    /// Load data from a REST API with custom configuration
    ///
    /// Requires the "rest-api" feature to be enabled.
    ///
    /// # Example
    /// ```rust,ignore
    /// let source = RestApiSource::new("https://api.example.com/data")
    ///     .with_method(HttpMethod::Post)
    ///     .with_header("Authorization", "Bearer token123")
    ///     .with_query_param("limit", "1000")
    ///     .with_timeout_secs(60);
    ///
    /// let cube = ElastiCubeBuilder::new("api_data")
    ///     .load_rest_api_with(source)
    ///     .build()?;
    /// ```
    #[cfg(feature = "rest-api")]
    pub fn load_rest_api_with(mut self, source: crate::sources::rest::RestApiSource) -> Self {
        self.data_source = Some(Box::new(source));
        self
    }

    // ==============================================================================
    // Object Storage Sources (available with "object-storage" feature)
    // ==============================================================================

    /// Load data from AWS S3
    ///
    /// Requires the "object-storage" feature to be enabled.
    ///
    /// # Arguments
    /// * `bucket` - S3 bucket name
    /// * `path` - Path to the file in the bucket (e.g., "data/sales.parquet")
    ///
    /// # Example
    /// ```rust,ignore
    /// // Uses AWS credentials from environment or ~/.aws/credentials
    /// let cube = ElastiCubeBuilder::new("sales")
    ///     .load_s3("my-bucket", "data/sales.parquet")
    ///     .build()?;
    /// ```
    #[cfg(feature = "object-storage")]
    pub fn load_s3(
        mut self,
        bucket: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        use crate::sources::object_storage::S3Source;
        let source = S3Source::new(bucket, path);
        self.data_source = Some(Box::new(source));
        self
    }

    /// Load data from AWS S3 with custom configuration
    ///
    /// Requires the "object-storage" feature to be enabled.
    ///
    /// # Example
    /// ```rust,ignore
    /// use elasticube_core::{S3Source, StorageFileFormat};
    ///
    /// let source = S3Source::new("my-bucket", "data/sales.csv")
    ///     .with_region("us-west-2")
    ///     .with_access_key("AKIAIOSFODNN7EXAMPLE", "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY")
    ///     .with_format(StorageFileFormat::Csv)
    ///     .with_batch_size(4096);
    ///
    /// let cube = ElastiCubeBuilder::new("sales")
    ///     .load_s3_with(source)
    ///     .build()?;
    /// ```
    #[cfg(feature = "object-storage")]
    pub fn load_s3_with(mut self, source: crate::sources::object_storage::S3Source) -> Self {
        self.data_source = Some(Box::new(source));
        self
    }

    /// Load data from Google Cloud Storage (GCS)
    ///
    /// Requires the "object-storage" feature to be enabled.
    ///
    /// # Arguments
    /// * `bucket` - GCS bucket name
    /// * `path` - Path to the file in the bucket
    ///
    /// # Example
    /// ```rust,ignore
    /// // Uses Google Cloud credentials from GOOGLE_APPLICATION_CREDENTIALS env var
    /// let cube = ElastiCubeBuilder::new("analytics")
    ///     .load_gcs("my-gcs-bucket", "data/analytics.parquet")
    ///     .build()?;
    /// ```
    #[cfg(feature = "object-storage")]
    pub fn load_gcs(
        mut self,
        bucket: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        use crate::sources::object_storage::GcsSource;
        let source = GcsSource::new(bucket, path);
        self.data_source = Some(Box::new(source));
        self
    }

    /// Load data from Google Cloud Storage with custom configuration
    ///
    /// Requires the "object-storage" feature to be enabled.
    ///
    /// # Example
    /// ```rust,ignore
    /// use elasticube_core::{GcsSource, StorageFileFormat};
    ///
    /// let source = GcsSource::new("my-bucket", "data/metrics.json")
    ///     .with_service_account_key("/path/to/key.json")
    ///     .with_format(StorageFileFormat::Json)
    ///     .with_batch_size(8192);
    ///
    /// let cube = ElastiCubeBuilder::new("metrics")
    ///     .load_gcs_with(source)
    ///     .build()?;
    /// ```
    #[cfg(feature = "object-storage")]
    pub fn load_gcs_with(mut self, source: crate::sources::object_storage::GcsSource) -> Self {
        self.data_source = Some(Box::new(source));
        self
    }

    /// Load data from Azure Blob Storage
    ///
    /// Requires the "object-storage" feature to be enabled.
    ///
    /// # Arguments
    /// * `account` - Azure storage account name
    /// * `container` - Container name
    /// * `path` - Path to the file in the container
    ///
    /// # Example
    /// ```rust,ignore
    /// let cube = ElastiCubeBuilder::new("reports")
    ///     .load_azure("mystorageaccount", "mycontainer", "data/reports.parquet")
    ///     .build()?;
    /// ```
    #[cfg(feature = "object-storage")]
    pub fn load_azure(
        mut self,
        account: impl Into<String>,
        container: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        use crate::sources::object_storage::AzureSource;
        let source = AzureSource::new(account, container, path);
        self.data_source = Some(Box::new(source));
        self
    }

    /// Load data from Azure Blob Storage with custom configuration
    ///
    /// Requires the "object-storage" feature to be enabled.
    ///
    /// # Example
    /// ```rust,ignore
    /// use elasticube_core::{AzureSource, StorageFileFormat};
    ///
    /// let source = AzureSource::new("mystorageaccount", "mycontainer", "data/logs.csv")
    ///     .with_access_key("your-access-key")
    ///     .with_format(StorageFileFormat::Csv)
    ///     .with_batch_size(4096);
    ///
    /// let cube = ElastiCubeBuilder::new("logs")
    ///     .load_azure_with(source)
    ///     .build()?;
    /// ```
    #[cfg(feature = "object-storage")]
    pub fn load_azure_with(mut self, source: crate::sources::object_storage::AzureSource) -> Self {
        self.data_source = Some(Box::new(source));
        self
    }

    /// Build the cube
    ///
    /// Loads data from the configured source and creates an ElastiCube.
    /// If dimensions and measures were explicitly defined, validates that the
    /// data schema matches. Otherwise, infers the schema from the data.
    pub fn build(mut self) -> Result<ElastiCube> {
        // Ensure we have a data source
        let data_source = self.data_source.take().ok_or_else(|| {
            Error::builder("No data source specified. Use load_csv, load_parquet, load_json, or load_record_batches")
        })?;

        // Load data from the source
        let (loaded_schema, batches) = data_source.load()?;

        // Determine the final Arrow schema
        let arrow_schema = if self.schema.dimension_count() > 0 || self.schema.measure_count() > 0 {
            // User has explicitly defined dimensions/measures
            // Convert our CubeSchema to ArrowSchema and validate against loaded data
            let expected_schema = Arc::new(self.schema.to_arrow_schema());

            // Validate that the loaded schema is compatible
            validate_schema_compatibility(&expected_schema, &loaded_schema)?;

            // Use the loaded schema to avoid mismatch errors with RecordBatch schemas
            // The validation ensures compatibility between expected and loaded schemas
            loaded_schema
        } else {
            // No explicit schema defined - infer from loaded data
            // We'll treat all columns as dimensions for now
            // Users can explicitly specify measures if they want aggregations
            for field in loaded_schema.fields() {
                let dimension = Dimension::new(field.name(), field.data_type().clone());
                self.schema.add_dimension(dimension)?;
            }

            loaded_schema
        };

        // Create the ElastiCube
        ElastiCube::new(self.schema, arrow_schema, batches)
    }
}

/// Validate that a loaded schema is compatible with the expected schema
///
/// Checks that all expected fields exist in the loaded schema with compatible types
fn validate_schema_compatibility(
    expected: &ArrowSchema,
    loaded: &ArrowSchema,
) -> Result<()> {
    for expected_field in expected.fields() {
        let loaded_field = loaded.field_with_name(expected_field.name()).map_err(|_| {
            Error::schema(format!(
                "Field '{}' not found in loaded data",
                expected_field.name()
            ))
        })?;

        // Check if data types match
        if expected_field.data_type() != loaded_field.data_type() {
            return Err(Error::schema(format!(
                "Field '{}' has incompatible type: expected {:?}, found {:?}",
                expected_field.name(),
                expected_field.data_type(),
                loaded_field.data_type()
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Float64Array, Int32Array, StringArray};
    use arrow::datatypes::Field;
    use std::sync::Arc;

    #[test]
    fn test_builder_creation() {
        let builder = ElastiCubeBuilder::new("test_cube");
        assert_eq!(builder.schema.name(), "test_cube");
    }

    #[test]
    fn test_builder_add_dimension() {
        let builder = ElastiCubeBuilder::new("test")
            .add_dimension("region", DataType::Utf8)
            .unwrap();
        assert!(builder.schema.has_dimension("region"));
    }

    #[test]
    fn test_builder_add_measure() {
        let builder = ElastiCubeBuilder::new("test")
            .add_measure("sales", DataType::Float64, AggFunc::Sum)
            .unwrap();
        assert!(builder.schema.has_measure("sales"));
    }

    #[test]
    fn test_build_without_data_source() {
        let builder = ElastiCubeBuilder::new("test")
            .add_dimension("region", DataType::Utf8)
            .unwrap();

        let result = builder.build();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No data source specified"));
    }

    #[test]
    fn test_build_with_record_batches() {
        // Create a simple schema
        let schema = Arc::new(ArrowSchema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("value", DataType::Float64, false),
        ]));

        // Create some data
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int32Array::from(vec![1, 2, 3])),
                Arc::new(Float64Array::from(vec![1.0, 2.0, 3.0])),
            ],
        )
        .unwrap();

        // Build the cube
        let cube = ElastiCubeBuilder::new("test")
            .load_record_batches(schema, vec![batch])
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(cube.row_count(), 3);
        assert_eq!(cube.dimensions().len(), 2); // Both fields treated as dimensions
    }

    #[test]
    fn test_build_with_explicit_schema() {
        // Create a schema
        let schema = Arc::new(ArrowSchema::new(vec![
            Field::new("region", DataType::Utf8, false),
            Field::new("sales", DataType::Float64, false),
        ]));

        // Create some data
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(vec!["North", "South", "East"])),
                Arc::new(Float64Array::from(vec![100.0, 200.0, 150.0])),
            ],
        )
        .unwrap();

        // Build the cube with explicit dimensions and measures
        let cube = ElastiCubeBuilder::new("sales_cube")
            .add_dimension("region", DataType::Utf8)
            .unwrap()
            .add_measure("sales", DataType::Float64, AggFunc::Sum)
            .unwrap()
            .load_record_batches(schema, vec![batch])
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(cube.row_count(), 3);
        assert_eq!(cube.dimensions().len(), 1);
        assert_eq!(cube.measures().len(), 1);
    }

    #[test]
    fn test_schema_validation_failure() {
        // Create a schema with wrong field names
        let schema = Arc::new(ArrowSchema::new(vec![
            Field::new("wrong_name", DataType::Utf8, false),
            Field::new("sales", DataType::Float64, false),
        ]));

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(vec!["North"])),
                Arc::new(Float64Array::from(vec![100.0])),
            ],
        )
        .unwrap();

        // This should fail because "region" is not in the loaded schema
        let result = ElastiCubeBuilder::new("test")
            .add_dimension("region", DataType::Utf8)
            .unwrap()
            .add_measure("sales", DataType::Float64, AggFunc::Sum)
            .unwrap()
            .load_record_batches(schema, vec![batch])
            .unwrap()
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
