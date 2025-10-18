//! Hierarchy types for drill-down/roll-up operations

use serde::{Deserialize, Serialize};

/// Represents a hierarchy in the cube
///
/// A hierarchy defines a drill-down path through related dimensions
/// (e.g., Year → Quarter → Month → Day for time-based analysis).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hierarchy {
    /// Name of the hierarchy
    name: String,

    /// Ordered list of dimension names from coarsest to finest granularity
    /// Example: ["year", "quarter", "month", "day"]
    levels: Vec<String>,

    /// User-provided description
    description: Option<String>,
}

impl Hierarchy {
    /// Create a new hierarchy
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the hierarchy
    /// * `levels` - Ordered list of dimension names from coarse to fine granularity
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let time_hierarchy = Hierarchy::new(
    ///     "time",
    ///     vec!["year", "quarter", "month", "day"]
    /// );
    /// ```
    pub fn new(name: impl Into<String>, levels: Vec<String>) -> Self {
        Self {
            name: name.into(),
            levels,
            description: None,
        }
    }

    /// Create a new hierarchy with description
    pub fn with_config(
        name: impl Into<String>,
        levels: Vec<String>,
        description: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            levels,
            description,
        }
    }

    /// Get the hierarchy name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the levels
    pub fn levels(&self) -> &[String] {
        &self.levels
    }

    /// Get the description
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get the number of levels
    pub fn depth(&self) -> usize {
        self.levels.len()
    }

    /// Get a level by index (0 = coarsest, depth-1 = finest)
    pub fn level_at(&self, index: usize) -> Option<&str> {
        self.levels.get(index).map(|s| s.as_str())
    }

    /// Get the coarsest (top) level
    pub fn top_level(&self) -> Option<&str> {
        self.levels.first().map(|s| s.as_str())
    }

    /// Get the finest (bottom) level
    pub fn bottom_level(&self) -> Option<&str> {
        self.levels.last().map(|s| s.as_str())
    }

    /// Get the parent level of a given level
    pub fn parent_of(&self, level: &str) -> Option<&str> {
        self.levels
            .iter()
            .position(|l| l == level)
            .and_then(|idx| {
                if idx > 0 {
                    self.levels.get(idx - 1).map(|s| s.as_str())
                } else {
                    None
                }
            })
    }

    /// Get the child level of a given level
    pub fn child_of(&self, level: &str) -> Option<&str> {
        self.levels
            .iter()
            .position(|l| l == level)
            .and_then(|idx| self.levels.get(idx + 1).map(|s| s.as_str()))
    }

    /// Check if a level exists in this hierarchy
    pub fn contains_level(&self, level: &str) -> bool {
        self.levels.iter().any(|l| l == level)
    }

    /// Get all ancestor levels of a given level (from top to parent)
    pub fn ancestors_of(&self, level: &str) -> Vec<&str> {
        if let Some(idx) = self.levels.iter().position(|l| l == level) {
            self.levels[..idx].iter().map(|s| s.as_str()).collect()
        } else {
            vec![]
        }
    }

    /// Get all descendant levels of a given level (from child to bottom)
    pub fn descendants_of(&self, level: &str) -> Vec<&str> {
        if let Some(idx) = self.levels.iter().position(|l| l == level) {
            self.levels[idx + 1..]
                .iter()
                .map(|s| s.as_str())
                .collect()
        } else {
            vec![]
        }
    }

    /// Set the description
    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = Some(description.into());
    }

    /// Builder-style: set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Validate the hierarchy
    pub fn validate(&self) -> Result<(), String> {
        if self.levels.is_empty() {
            return Err("Hierarchy must have at least one level".to_string());
        }

        // Check for duplicate levels
        let mut seen = std::collections::HashSet::new();
        for level in &self.levels {
            if !seen.insert(level) {
                return Err(format!("Duplicate level '{}' in hierarchy", level));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchy_creation() {
        let hierarchy = Hierarchy::new(
            "time",
            vec![
                "year".to_string(),
                "quarter".to_string(),
                "month".to_string(),
                "day".to_string(),
            ],
        );

        assert_eq!(hierarchy.name(), "time");
        assert_eq!(hierarchy.depth(), 4);
        assert_eq!(hierarchy.top_level(), Some("year"));
        assert_eq!(hierarchy.bottom_level(), Some("day"));
    }

    #[test]
    fn test_hierarchy_navigation() {
        let hierarchy = Hierarchy::new(
            "geography",
            vec![
                "country".to_string(),
                "state".to_string(),
                "city".to_string(),
            ],
        );

        assert_eq!(hierarchy.parent_of("state"), Some("country"));
        assert_eq!(hierarchy.parent_of("country"), None);
        assert_eq!(hierarchy.child_of("country"), Some("state"));
        assert_eq!(hierarchy.child_of("city"), None);
    }

    #[test]
    fn test_hierarchy_ancestors_descendants() {
        let hierarchy = Hierarchy::new(
            "time",
            vec![
                "year".to_string(),
                "quarter".to_string(),
                "month".to_string(),
                "day".to_string(),
            ],
        );

        assert_eq!(hierarchy.ancestors_of("month"), vec!["year", "quarter"]);
        assert_eq!(hierarchy.descendants_of("quarter"), vec!["month", "day"]);
        assert_eq!(hierarchy.ancestors_of("year"), Vec::<&str>::new());
        assert_eq!(hierarchy.descendants_of("day"), Vec::<&str>::new());
    }

    #[test]
    fn test_hierarchy_validation() {
        let valid = Hierarchy::new(
            "test",
            vec!["level1".to_string(), "level2".to_string()],
        );
        assert!(valid.validate().is_ok());

        let empty = Hierarchy::new("test", vec![]);
        assert!(empty.validate().is_err());

        let duplicate = Hierarchy::new(
            "test",
            vec!["level1".to_string(), "level1".to_string()],
        );
        assert!(duplicate.validate().is_err());
    }
}
