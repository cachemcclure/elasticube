//! ElastiCube builder for constructing cubes

use crate::cube::{AggFunc, CubeSchema, Dimension, ElastiCube, Hierarchy, Measure};
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
