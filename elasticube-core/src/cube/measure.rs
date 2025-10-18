//! Measure types and aggregation functions

use arrow::datatypes::DataType;
use serde::{Deserialize, Serialize};

/// Aggregation function for measures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AggFunc {
    /// Sum of values
    Sum,
    /// Average (mean) of values
    Avg,
    /// Minimum value
    Min,
    /// Maximum value
    Max,
    /// Count of values (non-null)
    Count,
    /// Count of distinct values
    CountDistinct,
    /// Median value
    Median,
    /// Standard deviation
    StdDev,
    /// Variance
    Variance,
    /// First value
    First,
    /// Last value
    Last,
}

impl AggFunc {
    /// Get the SQL function name for this aggregation
    pub fn sql_name(&self) -> &'static str {
        match self {
            AggFunc::Sum => "SUM",
            AggFunc::Avg => "AVG",
            AggFunc::Min => "MIN",
            AggFunc::Max => "MAX",
            AggFunc::Count => "COUNT",
            AggFunc::CountDistinct => "COUNT",
            AggFunc::Median => "MEDIAN",
            AggFunc::StdDev => "STDDEV",
            AggFunc::Variance => "VAR",
            AggFunc::First => "FIRST_VALUE",
            AggFunc::Last => "LAST_VALUE",
        }
    }

    /// Check if this aggregation is compatible with the given data type
    pub fn is_compatible_with(&self, data_type: &DataType) -> bool {
        use DataType::*;
        match self {
            AggFunc::Sum | AggFunc::Avg | AggFunc::StdDev | AggFunc::Variance => {
                matches!(
                    data_type,
                    Int8 | Int16
                        | Int32
                        | Int64
                        | UInt8
                        | UInt16
                        | UInt32
                        | UInt64
                        | Float32
                        | Float64
                        | Decimal128(_, _)
                        | Decimal256(_, _)
                )
            }
            AggFunc::Min | AggFunc::Max | AggFunc::First | AggFunc::Last => true,
            AggFunc::Count | AggFunc::CountDistinct => true,
            AggFunc::Median => {
                matches!(
                    data_type,
                    Int8 | Int16
                        | Int32
                        | Int64
                        | UInt8
                        | UInt16
                        | UInt32
                        | UInt64
                        | Float32
                        | Float64
                )
            }
        }
    }
}

impl std::fmt::Display for AggFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.sql_name())
    }
}

/// Represents a measure in the cube
///
/// A measure is a numeric value that can be aggregated
/// (e.g., sales amount, quantity, revenue).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Measure {
    /// Name of the measure
    name: String,

    /// Data type of the measure
    data_type: DataType,

    /// Default aggregation function
    default_agg: AggFunc,

    /// Whether this measure can be null
    nullable: bool,

    /// User-provided description
    description: Option<String>,

    /// Format string for display (e.g., "$,.2f" for currency)
    format: Option<String>,
}

impl Measure {
    /// Create a new measure with a default aggregation function
    pub fn new(name: impl Into<String>, data_type: DataType, default_agg: AggFunc) -> Self {
        Self {
            name: name.into(),
            data_type,
            default_agg,
            nullable: true,
            description: None,
            format: None,
        }
    }

    /// Create a new measure with full configuration
    pub fn with_config(
        name: impl Into<String>,
        data_type: DataType,
        default_agg: AggFunc,
        nullable: bool,
        description: Option<String>,
        format: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            data_type,
            default_agg,
            nullable,
            description,
            format,
        }
    }

    /// Get the measure name
    pub fn name(&self) -> &str {
        &self.name
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

    /// Set the description
    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = Some(description.into());
    }

    /// Set the format
    pub fn set_format(&mut self, format: impl Into<String>) {
        self.format = Some(format.into());
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

    /// Validate that the default aggregation is compatible with the data type
    pub fn validate(&self) -> Result<(), String> {
        if !self.default_agg.is_compatible_with(&self.data_type) {
            return Err(format!(
                "Aggregation function {} is not compatible with data type {:?}",
                self.default_agg, self.data_type
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measure_creation() {
        let measure = Measure::new("revenue", DataType::Float64, AggFunc::Sum);
        assert_eq!(measure.name(), "revenue");
        assert_eq!(measure.data_type(), &DataType::Float64);
        assert_eq!(measure.default_agg(), AggFunc::Sum);
        assert!(measure.is_nullable());
    }

    #[test]
    fn test_measure_validation() {
        let valid_measure = Measure::new("amount", DataType::Float64, AggFunc::Sum);
        assert!(valid_measure.validate().is_ok());

        let invalid_measure = Measure::new("category", DataType::Utf8, AggFunc::Sum);
        assert!(invalid_measure.validate().is_err());
    }

    #[test]
    fn test_agg_func_compatibility() {
        assert!(AggFunc::Sum.is_compatible_with(&DataType::Float64));
        assert!(AggFunc::Sum.is_compatible_with(&DataType::Int32));
        assert!(!AggFunc::Sum.is_compatible_with(&DataType::Utf8));

        assert!(AggFunc::Count.is_compatible_with(&DataType::Utf8));
        assert!(AggFunc::Max.is_compatible_with(&DataType::Utf8));
    }

    #[test]
    fn test_measure_builder() {
        let measure = Measure::new("sales", DataType::Float64, AggFunc::Sum)
            .with_nullable(false)
            .with_description("Total sales amount")
            .with_format("$,.2f");

        assert_eq!(measure.name(), "sales");
        assert!(!measure.is_nullable());
        assert_eq!(measure.description(), Some("Total sales amount"));
        assert_eq!(measure.format(), Some("$,.2f"));
    }
}
