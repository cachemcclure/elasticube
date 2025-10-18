//! Core ElastiCube data structures

mod dimension;
mod hierarchy;
mod measure;
mod schema;

pub use dimension::Dimension;
pub use hierarchy::Hierarchy;
pub use measure::{AggFunc, Measure};
pub use schema::CubeSchema;

use crate::error::Result;
use crate::query::QueryBuilder;
use arrow::datatypes::Schema as ArrowSchema;
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

/// The main ElastiCube structure
///
/// Represents a multidimensional cube with dimensions, measures, and data stored
/// in Apache Arrow's columnar format for efficient analytical queries.
#[derive(Debug, Clone)]
pub struct ElastiCube {
    /// Cube metadata and schema definition
    schema: CubeSchema,

    /// Underlying Arrow schema
    arrow_schema: Arc<ArrowSchema>,

    /// Data stored as Arrow RecordBatches
    /// Using Vec to support chunked data (each RecordBatch is a chunk)
    data: Vec<RecordBatch>,

    /// Total number of rows across all batches
    row_count: usize,
}

impl ElastiCube {
    /// Create a new ElastiCube
    pub fn new(
        schema: CubeSchema,
        arrow_schema: Arc<ArrowSchema>,
        data: Vec<RecordBatch>,
    ) -> Result<Self> {
        let row_count = data.iter().map(|batch| batch.num_rows()).sum();

        Ok(Self {
            schema,
            arrow_schema,
            data,
            row_count,
        })
    }

    /// Get the cube schema
    pub fn schema(&self) -> &CubeSchema {
        &self.schema
    }

    /// Get the Arrow schema
    pub fn arrow_schema(&self) -> &Arc<ArrowSchema> {
        &self.arrow_schema
    }

    /// Get the data batches
    pub fn data(&self) -> &[RecordBatch] {
        &self.data
    }

    /// Get the total number of rows
    pub fn row_count(&self) -> usize {
        self.row_count
    }

    /// Get all dimensions
    pub fn dimensions(&self) -> Vec<&Dimension> {
        self.schema.dimensions()
    }

    /// Get all measures
    pub fn measures(&self) -> Vec<&Measure> {
        self.schema.measures()
    }

    /// Get all hierarchies
    pub fn hierarchies(&self) -> Vec<&Hierarchy> {
        self.schema.hierarchies()
    }

    /// Get a dimension by name
    pub fn get_dimension(&self, name: &str) -> Option<&Dimension> {
        self.schema.get_dimension(name)
    }

    /// Get a measure by name
    pub fn get_measure(&self, name: &str) -> Option<&Measure> {
        self.schema.get_measure(name)
    }

    /// Get a hierarchy by name
    pub fn get_hierarchy(&self, name: &str) -> Option<&Hierarchy> {
        self.schema.get_hierarchy(name)
    }

    /// Create a query builder for this cube
    ///
    /// # Returns
    /// A QueryBuilder instance for executing queries against this cube
    ///
    /// # Example
    /// ```rust,ignore
    /// let results = cube.query()?
    ///     .select(&["region", "SUM(sales) as total"])
    ///     .group_by(&["region"])
    ///     .execute()
    ///     .await?;
    /// ```
    pub fn query(self: Arc<Self>) -> Result<QueryBuilder> {
        QueryBuilder::new(self)
    }

    /// Get cube statistics for performance analysis
    ///
    /// Returns statistics about the cube's data including row count,
    /// partition count, memory usage, and column-level statistics.
    ///
    /// # Example
    /// ```rust,ignore
    /// let stats = cube.statistics();
    /// println!("Cube: {}", stats.summary());
    /// ```
    pub fn statistics(&self) -> crate::optimization::CubeStatistics {
        crate::optimization::CubeStatistics::from_batches(&self.data)
    }

    /// Create a query builder with custom optimization configuration
    ///
    /// # Arguments
    /// * `config` - Optimization configuration to use for queries
    ///
    /// # Returns
    /// A QueryBuilder instance with the specified optimization settings
    ///
    /// # Example
    /// ```rust,ignore
    /// use elasticube_core::OptimizationConfig;
    ///
    /// let config = OptimizationConfig::new()
    ///     .with_target_partitions(8)
    ///     .with_batch_size(4096);
    ///
    /// let results = cube.query_with_config(config)?
    ///     .select(&["region", "SUM(sales)"])
    ///     .execute()
    ///     .await?;
    /// ```
    pub fn query_with_config(
        self: Arc<Self>,
        config: crate::optimization::OptimizationConfig,
    ) -> Result<QueryBuilder> {
        QueryBuilder::with_config(self, config)
    }
}
