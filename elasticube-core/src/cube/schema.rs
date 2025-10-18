//! Schema metadata for ElastiCube

use super::{CalculatedMeasure, Dimension, Hierarchy, Measure, VirtualDimension};
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

    /// Calculated measures (derived from expressions)
    calculated_measures: IndexMap<String, CalculatedMeasure>,

    /// Virtual dimensions (computed dimensions)
    virtual_dimensions: IndexMap<String, VirtualDimension>,

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
            calculated_measures: IndexMap::new(),
            virtual_dimensions: IndexMap::new(),
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

    /// Add a calculated measure to the schema
    pub fn add_calculated_measure(&mut self, calc_measure: CalculatedMeasure) -> Result<()> {
        let name = calc_measure.name().to_string();

        // Check for name conflicts with regular measures and calculated measures
        if self.measures.contains_key(&name) {
            return Err(Error::measure(format!(
                "A measure named '{}' already exists",
                name
            )));
        }
        if self.calculated_measures.contains_key(&name) {
            return Err(Error::measure(format!(
                "Calculated measure '{}' already exists",
                name
            )));
        }

        self.calculated_measures.insert(name, calc_measure);
        Ok(())
    }

    /// Add a virtual dimension to the schema
    pub fn add_virtual_dimension(&mut self, virtual_dim: VirtualDimension) -> Result<()> {
        let name = virtual_dim.name().to_string();

        // Check for name conflicts with regular dimensions and virtual dimensions
        if self.dimensions.contains_key(&name) {
            return Err(Error::dimension(format!(
                "A dimension named '{}' already exists",
                name
            )));
        }
        if self.virtual_dimensions.contains_key(&name) {
            return Err(Error::dimension(format!(
                "Virtual dimension '{}' already exists",
                name
            )));
        }

        self.virtual_dimensions.insert(name, virtual_dim);
        Ok(())
    }

    /// Get all calculated measures
    pub fn calculated_measures(&self) -> Vec<&CalculatedMeasure> {
        self.calculated_measures.values().collect()
    }

    /// Get all virtual dimensions
    pub fn virtual_dimensions(&self) -> Vec<&VirtualDimension> {
        self.virtual_dimensions.values().collect()
    }

    /// Get a calculated measure by name
    pub fn get_calculated_measure(&self, name: &str) -> Option<&CalculatedMeasure> {
        self.calculated_measures.get(name)
    }

    /// Get a virtual dimension by name
    pub fn get_virtual_dimension(&self, name: &str) -> Option<&VirtualDimension> {
        self.virtual_dimensions.get(name)
    }

    /// Remove a calculated measure
    pub fn remove_calculated_measure(&mut self, name: &str) -> Result<CalculatedMeasure> {
        self.calculated_measures.shift_remove(name).ok_or_else(|| {
            Error::measure(format!("Calculated measure '{}' not found", name))
        })
    }

    /// Remove a virtual dimension
    pub fn remove_virtual_dimension(&mut self, name: &str) -> Result<VirtualDimension> {
        self.virtual_dimensions.shift_remove(name).ok_or_else(|| {
            Error::dimension(format!("Virtual dimension '{}' not found", name))
        })
    }

    /// Check if a calculated measure exists
    pub fn has_calculated_measure(&self, name: &str) -> bool {
        self.calculated_measures.contains_key(name)
    }

    /// Check if a virtual dimension exists
    pub fn has_virtual_dimension(&self, name: &str) -> bool {
        self.virtual_dimensions.contains_key(name)
    }

    /// Get the number of calculated measures
    pub fn calculated_measure_count(&self) -> usize {
        self.calculated_measures.len()
    }

    /// Get the number of virtual dimensions
    pub fn virtual_dimension_count(&self) -> usize {
        self.virtual_dimensions.len()
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

    #[test]
    fn test_add_calculated_measure() {
        use super::CalculatedMeasure;

        let mut schema = CubeSchema::new("test");

        // Add base measures first
        schema
            .add_measure(Measure::new("revenue", DataType::Float64, AggFunc::Sum))
            .unwrap();
        schema
            .add_measure(Measure::new("cost", DataType::Float64, AggFunc::Sum))
            .unwrap();

        // Add calculated measure
        let profit = CalculatedMeasure::new(
            "profit",
            "revenue - cost",
            DataType::Float64,
            AggFunc::Sum,
        )
        .unwrap();

        assert!(schema.add_calculated_measure(profit).is_ok());
        assert_eq!(schema.calculated_measure_count(), 1);
        assert!(schema.has_calculated_measure("profit"));

        // Test duplicate
        let profit2 = CalculatedMeasure::new(
            "profit",
            "revenue - cost",
            DataType::Float64,
            AggFunc::Sum,
        )
        .unwrap();
        assert!(schema.add_calculated_measure(profit2).is_err());
    }

    #[test]
    fn test_add_virtual_dimension() {
        use super::VirtualDimension;

        let mut schema = CubeSchema::new("test");

        // Add base dimension
        schema
            .add_dimension(Dimension::new("sale_date", DataType::Date32))
            .unwrap();

        // Add virtual dimension
        let year = VirtualDimension::new(
            "year",
            "EXTRACT(YEAR FROM sale_date)",
            DataType::Int32,
        )
        .unwrap();

        assert!(schema.add_virtual_dimension(year).is_ok());
        assert_eq!(schema.virtual_dimension_count(), 1);
        assert!(schema.has_virtual_dimension("year"));

        // Test duplicate
        let year2 =
            VirtualDimension::new("year", "EXTRACT(YEAR FROM sale_date)", DataType::Int32)
                .unwrap();
        assert!(schema.add_virtual_dimension(year2).is_err());
    }

    #[test]
    fn test_calculated_measure_name_conflict() {
        use super::CalculatedMeasure;

        let mut schema = CubeSchema::new("test");

        // Add a regular measure
        schema
            .add_measure(Measure::new("sales", DataType::Float64, AggFunc::Sum))
            .unwrap();

        // Try to add calculated measure with same name - should fail
        let calc_sales =
            CalculatedMeasure::new("sales", "revenue * 0.8", DataType::Float64, AggFunc::Sum)
                .unwrap();
        assert!(schema.add_calculated_measure(calc_sales).is_err());
    }

    #[test]
    fn test_virtual_dimension_name_conflict() {
        use super::VirtualDimension;

        let mut schema = CubeSchema::new("test");

        // Add a regular dimension
        schema
            .add_dimension(Dimension::new("region", DataType::Utf8))
            .unwrap();

        // Try to add virtual dimension with same name - should fail
        let virtual_region =
            VirtualDimension::new("region", "UPPER(region)", DataType::Utf8).unwrap();
        assert!(schema.add_virtual_dimension(virtual_region).is_err());
    }

    #[test]
    fn test_get_calculated_measure() {
        use super::CalculatedMeasure;

        let mut schema = CubeSchema::new("test");

        let margin =
            CalculatedMeasure::new("margin", "profit / revenue", DataType::Float64, AggFunc::Avg)
                .unwrap();
        schema.add_calculated_measure(margin).unwrap();

        let retrieved = schema.get_calculated_measure("margin").unwrap();
        assert_eq!(retrieved.name(), "margin");
        assert_eq!(retrieved.expression(), "profit / revenue");
    }

    #[test]
    fn test_remove_calculated_measure() {
        use super::CalculatedMeasure;

        let mut schema = CubeSchema::new("test");

        let calc = CalculatedMeasure::new("test", "a + b", DataType::Float64, AggFunc::Sum)
            .unwrap();
        schema.add_calculated_measure(calc).unwrap();

        assert!(schema.remove_calculated_measure("test").is_ok());
        assert_eq!(schema.calculated_measure_count(), 0);

        // Try to remove again - should fail
        assert!(schema.remove_calculated_measure("test").is_err());
    }
}
