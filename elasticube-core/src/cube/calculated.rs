//! Calculated measures and virtual dimensions
//!
//! Support for derived fields using DataFusion SQL expressions.

use arrow::datatypes::DataType;
use serde::{Deserialize, Serialize};

use super::measure::AggFunc;
use crate::error::{Error, Result};

/// A calculated measure derived from an expression
///
/// Calculated measures are derived from other measures or dimensions using
/// SQL-like expressions. They're computed at query time using DataFusion's
/// expression engine.
///
/// # Examples
///
/// ```rust,ignore
/// // profit = revenue - cost
/// let profit = CalculatedMeasure::new(
///     "profit",
///     "revenue - cost",
///     DataType::Float64,
///     AggFunc::Sum
/// )?;
///
/// // margin = (profit / revenue) * 100
/// let margin = CalculatedMeasure::new(
///     "margin",
///     "(profit / revenue) * 100",
///     DataType::Float64,
///     AggFunc::Avg
/// )?;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalculatedMeasure {
    /// Name of the calculated measure
    name: String,

    /// SQL expression to compute this measure
    /// Can reference other measures and dimensions
    expression: String,

    /// Expected data type of the result
    data_type: DataType,

    /// Default aggregation function
    default_agg: AggFunc,

    /// Whether the result can be null
    nullable: bool,

    /// User-provided description
    description: Option<String>,

    /// Format string for display
    format: Option<String>,
}

impl CalculatedMeasure {
    /// Create a new calculated measure
    ///
    /// # Arguments
    /// * `name` - Name for the calculated measure
    /// * `expression` - SQL expression (e.g., "revenue - cost")
    /// * `data_type` - Expected result data type
    /// * `default_agg` - Default aggregation function
    ///
    /// # Returns
    /// A new CalculatedMeasure instance
    pub fn new(
        name: impl Into<String>,
        expression: impl Into<String>,
        data_type: DataType,
        default_agg: AggFunc,
    ) -> Result<Self> {
        let name = name.into();
        let expression = expression.into();

        // Basic validation
        if name.is_empty() {
            return Err(Error::Schema("Calculated measure name cannot be empty".into()));
        }
        if expression.is_empty() {
            return Err(Error::Schema("Expression cannot be empty".into()));
        }

        // Validate aggregation is compatible with data type
        if !default_agg.is_compatible_with(&data_type) {
            return Err(Error::Schema(format!(
                "Aggregation function {} is not compatible with data type {:?}",
                default_agg, data_type
            )));
        }

        Ok(Self {
            name,
            expression,
            data_type,
            default_agg,
            nullable: true,
            description: None,
            format: None,
        })
    }

    /// Get the measure name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the SQL expression
    pub fn expression(&self) -> &str {
        &self.expression
    }

    /// Get the data type
    pub fn data_type(&self) -> &DataType {
        &self.data_type
    }

    /// Get the default aggregation function
    pub fn default_agg(&self) -> AggFunc {
        self.default_agg
    }

    /// Check if the measure is nullable
    pub fn is_nullable(&self) -> bool {
        self.nullable
    }

    /// Get the description
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get the format string
    pub fn format(&self) -> Option<&str> {
        self.format.as_deref()
    }

    /// Builder-style: set nullable
    pub fn with_nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    /// Builder-style: set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Builder-style: set format
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }
}

/// A virtual dimension computed from an expression
///
/// Virtual dimensions are derived from other dimensions or measures using
/// SQL-like expressions. Common use cases include date part extraction,
/// categorization, and transformations.
///
/// # Examples
///
/// ```rust,ignore
/// // Extract year from date
/// let year = VirtualDimension::new(
///     "year",
///     "EXTRACT(YEAR FROM sale_date)",
///     DataType::Int32
/// )?;
///
/// // Categorize ages
/// let age_group = VirtualDimension::new(
///     "age_group",
///     "CASE WHEN age < 18 THEN 'Minor' WHEN age < 65 THEN 'Adult' ELSE 'Senior' END",
///     DataType::Utf8
/// )?;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VirtualDimension {
    /// Name of the virtual dimension
    name: String,

    /// SQL expression to compute this dimension
    expression: String,

    /// Expected data type of the result
    data_type: DataType,

    /// Whether the result can be null
    nullable: bool,

    /// Estimated cardinality (number of unique values)
    cardinality: Option<usize>,

    /// User-provided description
    description: Option<String>,
}

impl VirtualDimension {
    /// Create a new virtual dimension
    ///
    /// # Arguments
    /// * `name` - Name for the virtual dimension
    /// * `expression` - SQL expression (e.g., "EXTRACT(YEAR FROM date)")
    /// * `data_type` - Expected result data type
    ///
    /// # Returns
    /// A new VirtualDimension instance
    pub fn new(
        name: impl Into<String>,
        expression: impl Into<String>,
        data_type: DataType,
    ) -> Result<Self> {
        let name = name.into();
        let expression = expression.into();

        // Basic validation
        if name.is_empty() {
            return Err(Error::Schema("Virtual dimension name cannot be empty".into()));
        }
        if expression.is_empty() {
            return Err(Error::Schema("Expression cannot be empty".into()));
        }

        Ok(Self {
            name,
            expression,
            data_type,
            nullable: true,
            cardinality: None,
            description: None,
        })
    }

    /// Get the dimension name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the SQL expression
    pub fn expression(&self) -> &str {
        &self.expression
    }

    /// Get the data type
    pub fn data_type(&self) -> &DataType {
        &self.data_type
    }

    /// Check if the dimension is nullable
    pub fn is_nullable(&self) -> bool {
        self.nullable
    }

    /// Get the cardinality
    pub fn cardinality(&self) -> Option<usize> {
        self.cardinality
    }

    /// Get the description
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Builder-style: set nullable
    pub fn with_nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    /// Builder-style: set cardinality
    pub fn with_cardinality(mut self, cardinality: usize) -> Self {
        self.cardinality = Some(cardinality);
        self
    }

    /// Builder-style: set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculated_measure_creation() {
        let measure = CalculatedMeasure::new(
            "profit",
            "revenue - cost",
            DataType::Float64,
            AggFunc::Sum,
        )
        .unwrap();

        assert_eq!(measure.name(), "profit");
        assert_eq!(measure.expression(), "revenue - cost");
        assert_eq!(measure.data_type(), &DataType::Float64);
        assert_eq!(measure.default_agg(), AggFunc::Sum);
        assert!(measure.is_nullable());
    }

    #[test]
    fn test_calculated_measure_validation() {
        // Empty name should fail
        let result = CalculatedMeasure::new("", "a + b", DataType::Float64, AggFunc::Sum);
        assert!(result.is_err());

        // Empty expression should fail
        let result = CalculatedMeasure::new("test", "", DataType::Float64, AggFunc::Sum);
        assert!(result.is_err());

        // Incompatible aggregation should fail
        let result = CalculatedMeasure::new("test", "a || b", DataType::Utf8, AggFunc::Sum);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculated_measure_builder() {
        let measure = CalculatedMeasure::new(
            "margin",
            "profit / revenue * 100",
            DataType::Float64,
            AggFunc::Avg,
        )
        .unwrap()
        .with_nullable(false)
        .with_description("Profit margin percentage")
        .with_format(",.2f%");

        assert_eq!(measure.name(), "margin");
        assert!(!measure.is_nullable());
        assert_eq!(measure.description(), Some("Profit margin percentage"));
        assert_eq!(measure.format(), Some(",.2f%"));
    }

    #[test]
    fn test_virtual_dimension_creation() {
        let dim = VirtualDimension::new(
            "year",
            "EXTRACT(YEAR FROM sale_date)",
            DataType::Int32,
        )
        .unwrap();

        assert_eq!(dim.name(), "year");
        assert_eq!(dim.expression(), "EXTRACT(YEAR FROM sale_date)");
        assert_eq!(dim.data_type(), &DataType::Int32);
        assert!(dim.is_nullable());
    }

    #[test]
    fn test_virtual_dimension_validation() {
        // Empty name should fail
        let result = VirtualDimension::new("", "EXTRACT(YEAR FROM date)", DataType::Int32);
        assert!(result.is_err());

        // Empty expression should fail
        let result = VirtualDimension::new("year", "", DataType::Int32);
        assert!(result.is_err());
    }

    #[test]
    fn test_virtual_dimension_builder() {
        let dim = VirtualDimension::new(
            "age_group",
            "CASE WHEN age < 18 THEN 'Minor' ELSE 'Adult' END",
            DataType::Utf8,
        )
        .unwrap()
        .with_nullable(false)
        .with_cardinality(2)
        .with_description("Age category");

        assert_eq!(dim.name(), "age_group");
        assert!(!dim.is_nullable());
        assert_eq!(dim.cardinality(), Some(2));
        assert_eq!(dim.description(), Some("Age category"));
    }
}
