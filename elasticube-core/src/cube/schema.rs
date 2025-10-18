//! Schema metadata for ElastiCube

use super::{Dimension, Hierarchy, Measure};
use crate::error::{Error, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Schema metadata for an ElastiCube
///
/// Contains all metadata about dimensions, measures, and hierarchies,
/// providing a semantic layer over the raw Arrow data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CubeSchema {
    /// Name of the cube
    name: String,

    /// Dimensions indexed by name for fast lookup
    dimensions: IndexMap<String, Dimension>,

    /// Measures indexed by name for fast lookup
    measures: IndexMap<String, Measure>,

    /// Hierarchies indexed by name for fast lookup
    hierarchies: IndexMap<String, Hierarchy>,

    /// Optional description
    description: Option<String>,
}

impl CubeSchema {
    /// Create a new cube schema
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            dimensions: IndexMap::new(),
            measures: IndexMap::new(),
            hierarchies: IndexMap::new(),
            description: None,
        }
    }

    /// Get the cube name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the description
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Set the description
    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = Some(description.into());
    }

    /// Add a dimension to the schema
    pub fn add_dimension(&mut self, dimension: Dimension) -> Result<()> {
        let name = dimension.name().to_string();
        if self.dimensions.contains_key(&name) {
            return Err(Error::dimension(format!(
                "Dimension '{}' already exists",
                name
            )));
        }
        self.dimensions.insert(name, dimension);
        Ok(())
    }

    /// Add a measure to the schema
    pub fn add_measure(&mut self, measure: Measure) -> Result<()> {
        // Validate the measure
        measure.validate().map_err(Error::measure)?;

        let name = measure.name().to_string();
        if self.measures.contains_key(&name) {
            return Err(Error::measure(format!("Measure '{}' already exists", name)));
        }
        self.measures.insert(name, measure);
        Ok(())
    }

    /// Add a hierarchy to the schema
    pub fn add_hierarchy(&mut self, hierarchy: Hierarchy) -> Result<()> {
        // Validate the hierarchy
        hierarchy.validate().map_err(Error::hierarchy)?;

        // Validate that all levels in the hierarchy reference existing dimensions
        for level in hierarchy.levels() {
            if !self.dimensions.contains_key(level) {
                return Err(Error::hierarchy(format!(
                    "Hierarchy '{}' references non-existent dimension '{}'",
                    hierarchy.name(),
                    level
                )));
            }
        }

        let name = hierarchy.name().to_string();
        if self.hierarchies.contains_key(&name) {
            return Err(Error::hierarchy(format!(
                "Hierarchy '{}' already exists",
                name
            )));
        }
        self.hierarchies.insert(name, hierarchy);
        Ok(())
    }

    /// Get all dimensions
    pub fn dimensions(&self) -> Vec<&Dimension> {
        self.dimensions.values().collect()
    }

    /// Get all measures
    pub fn measures(&self) -> Vec<&Measure> {
        self.measures.values().collect()
    }

    /// Get all hierarchies
    pub fn hierarchies(&self) -> Vec<&Hierarchy> {
        self.hierarchies.values().collect()
    }

    /// Get a dimension by name
    pub fn get_dimension(&self, name: &str) -> Option<&Dimension> {
        self.dimensions.get(name)
    }

    /// Get a mutable dimension by name
    pub fn get_dimension_mut(&mut self, name: &str) -> Option<&mut Dimension> {
        self.dimensions.get_mut(name)
    }

    /// Get a measure by name
    pub fn get_measure(&self, name: &str) -> Option<&Measure> {
        self.measures.get(name)
    }

    /// Get a mutable measure by name
    pub fn get_measure_mut(&mut self, name: &str) -> Option<&mut Measure> {
        self.measures.get_mut(name)
    }

    /// Get a hierarchy by name
    pub fn get_hierarchy(&self, name: &str) -> Option<&Hierarchy> {
        self.hierarchies.get(name)
    }

    /// Remove a dimension
    pub fn remove_dimension(&mut self, name: &str) -> Result<Dimension> {
        // Check if any hierarchies reference this dimension
        for hierarchy in self.hierarchies.values() {
            if hierarchy.contains_level(name) {
                return Err(Error::dimension(format!(
                    "Cannot remove dimension '{}': referenced by hierarchy '{}'",
                    name,
                    hierarchy.name()
                )));
            }
        }

        self.dimensions
            .shift_remove(name)
            .ok_or_else(|| Error::dimension(format!("Dimension '{}' not found", name)))
    }

    /// Remove a measure
    pub fn remove_measure(&mut self, name: &str) -> Result<Measure> {
        self.measures
            .shift_remove(name)
            .ok_or_else(|| Error::measure(format!("Measure '{}' not found", name)))
    }

    /// Remove a hierarchy
    pub fn remove_hierarchy(&mut self, name: &str) -> Result<Hierarchy> {
        self.hierarchies
            .shift_remove(name)
            .ok_or_else(|| Error::hierarchy(format!("Hierarchy '{}' not found", name)))
    }

    /// Get the number of dimensions
    pub fn dimension_count(&self) -> usize {
        self.dimensions.len()
    }

    /// Get the number of measures
    pub fn measure_count(&self) -> usize {
        self.measures.len()
    }

    /// Get the number of hierarchies
    pub fn hierarchy_count(&self) -> usize {
        self.hierarchies.len()
    }

    /// Check if a dimension exists
    pub fn has_dimension(&self, name: &str) -> bool {
        self.dimensions.contains_key(name)
    }

    /// Check if a measure exists
    pub fn has_measure(&self, name: &str) -> bool {
        self.measures.contains_key(name)
    }

    /// Check if a hierarchy exists
    pub fn has_hierarchy(&self, name: &str) -> bool {
        self.hierarchies.contains_key(name)
    }

    /// Get all dimension names
    pub fn dimension_names(&self) -> Vec<&str> {
        self.dimensions.keys().map(|s| s.as_str()).collect()
    }

    /// Get all measure names
    pub fn measure_names(&self) -> Vec<&str> {
        self.measures.keys().map(|s| s.as_str()).collect()
    }

    /// Get all hierarchy names
    pub fn hierarchy_names(&self) -> Vec<&str> {
        self.hierarchies.keys().map(|s| s.as_str()).collect()
    }

    /// Convert CubeSchema to Arrow Schema
    ///
    /// Creates an Arrow schema containing fields for all dimensions and measures.
    /// The order is: dimensions first (in insertion order), then measures.
    pub fn to_arrow_schema(&self) -> arrow::datatypes::Schema {
        use arrow::datatypes::Field;

        let mut fields = Vec::new();

        // Add dimension fields
        for dim in self.dimensions.values() {
            fields.push(Field::new(
                dim.name(),
                dim.data_type().clone(),
                true, // nullable by default
            ));
        }

        // Add measure fields
        for measure in self.measures.values() {
            fields.push(Field::new(
                measure.name(),
                measure.data_type().clone(),
                true, // nullable by default
            ));
        }

        arrow::datatypes::Schema::new(fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cube::{AggFunc, Dimension, Hierarchy, Measure};
    use arrow::datatypes::DataType;

    #[test]
    fn test_schema_creation() {
        let schema = CubeSchema::new("sales_cube");
        assert_eq!(schema.name(), "sales_cube");
        assert_eq!(schema.dimension_count(), 0);
        assert_eq!(schema.measure_count(), 0);
    }

    #[test]
    fn test_add_dimension() {
        let mut schema = CubeSchema::new("test");
        let dim = Dimension::new("region", DataType::Utf8);

        assert!(schema.add_dimension(dim).is_ok());
        assert_eq!(schema.dimension_count(), 1);
        assert!(schema.has_dimension("region"));

        // Test duplicate
        let dim2 = Dimension::new("region", DataType::Utf8);
        assert!(schema.add_dimension(dim2).is_err());
    }

    #[test]
    fn test_add_measure() {
        let mut schema = CubeSchema::new("test");
        let measure = Measure::new("sales", DataType::Float64, AggFunc::Sum);

        assert!(schema.add_measure(measure).is_ok());
        assert_eq!(schema.measure_count(), 1);
        assert!(schema.has_measure("sales"));
    }

    #[test]
    fn test_add_hierarchy() {
        let mut schema = CubeSchema::new("test");

        // Add dimensions first
        schema
            .add_dimension(Dimension::new("year", DataType::Int32))
            .unwrap();
        schema
            .add_dimension(Dimension::new("quarter", DataType::Int32))
            .unwrap();
        schema
            .add_dimension(Dimension::new("month", DataType::Int32))
            .unwrap();

        // Add hierarchy
        let hierarchy = Hierarchy::new(
            "time",
            vec!["year".to_string(), "quarter".to_string(), "month".to_string()],
        );

        assert!(schema.add_hierarchy(hierarchy).is_ok());
        assert_eq!(schema.hierarchy_count(), 1);
        assert!(schema.has_hierarchy("time"));
    }

    #[test]
    fn test_hierarchy_validation() {
        let mut schema = CubeSchema::new("test");

        // Try to add hierarchy without dimensions
        let hierarchy = Hierarchy::new("time", vec!["year".to_string(), "month".to_string()]);

        assert!(schema.add_hierarchy(hierarchy).is_err());
    }

    #[test]
    fn test_remove_dimension_with_hierarchy() {
        let mut schema = CubeSchema::new("test");

        schema
            .add_dimension(Dimension::new("year", DataType::Int32))
            .unwrap();
        schema
            .add_dimension(Dimension::new("month", DataType::Int32))
            .unwrap();

        let hierarchy = Hierarchy::new("time", vec!["year".to_string(), "month".to_string()]);
        schema.add_hierarchy(hierarchy).unwrap();

        // Should fail because hierarchy references it
        assert!(schema.remove_dimension("year").is_err());

        // Remove hierarchy first
        schema.remove_hierarchy("time").unwrap();

        // Now should succeed
        assert!(schema.remove_dimension("year").is_ok());
    }
}
