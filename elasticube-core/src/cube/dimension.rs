//! Dimension types and operations

use arrow::datatypes::DataType;
use serde::{Deserialize, Serialize};

/// Represents a dimension in the cube
///
/// A dimension is a categorical attribute used for slicing and dicing data
/// (e.g., date, region, product category).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dimension {
    /// Name of the dimension
    name: String,

    /// Data type of the dimension
    data_type: DataType,

    /// Estimated cardinality (number of unique values)
    /// None if unknown
    cardinality: Option<usize>,

    /// Whether this dimension can be null
    nullable: bool,

    /// User-provided description
    description: Option<String>,
}

impl Dimension {
    /// Create a new dimension
    pub fn new(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            data_type,
            cardinality: None,
            nullable: true,
            description: None,
        }
    }

    /// Create a new dimension with full configuration
    pub fn with_config(
        name: impl Into<String>,
        data_type: DataType,
        nullable: bool,
        cardinality: Option<usize>,
        description: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            data_type,
            cardinality,
            nullable,
            description,
        }
    }

    /// Get the dimension name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the data type
    pub fn data_type(&self) -> &DataType {
        &self.data_type
    }

    /// Get the cardinality
    pub fn cardinality(&self) -> Option<usize> {
        self.cardinality
    }

    /// Check if the dimension is nullable
    pub fn is_nullable(&self) -> bool {
        self.nullable
    }

    /// Get the description
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Set the cardinality
    pub fn set_cardinality(&mut self, cardinality: usize) {
        self.cardinality = Some(cardinality);
    }

    /// Set the description
    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = Some(description.into());
    }

    /// Builder-style: set cardinality
    pub fn with_cardinality(mut self, cardinality: usize) -> Self {
        self.cardinality = Some(cardinality);
        self
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_creation() {
        let dim = Dimension::new("region", DataType::Utf8);
        assert_eq!(dim.name(), "region");
        assert_eq!(dim.data_type(), &DataType::Utf8);
        assert!(dim.is_nullable());
        assert_eq!(dim.cardinality(), None);
    }

    #[test]
    fn test_dimension_builder() {
        let dim = Dimension::new("country", DataType::Utf8)
            .with_cardinality(195)
            .with_nullable(false)
            .with_description("ISO country code");

        assert_eq!(dim.name(), "country");
        assert_eq!(dim.cardinality(), Some(195));
        assert!(!dim.is_nullable());
        assert_eq!(dim.description(), Some("ISO country code"));
    }
}
