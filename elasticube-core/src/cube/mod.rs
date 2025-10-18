//! Core ElastiCube data structures

mod calculated;
mod dimension;
mod hierarchy;
mod measure;
mod schema;
mod updates;

pub use calculated::{CalculatedMeasure, VirtualDimension};
pub use dimension::Dimension;
pub use hierarchy::Hierarchy;
pub use measure::{AggFunc, Measure};
pub use schema::CubeSchema;

use crate::error::{Error, Result};
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

    // ============================================================
    // Data Update Operations
    // ============================================================

    /// Append new rows from a RecordBatch to the cube
    ///
    /// This method adds new rows to the cube by appending a RecordBatch.
    /// The schema of the new batch must match the cube's schema exactly.
    ///
    /// # Arguments
    /// * `batch` - RecordBatch containing rows to append
    ///
    /// # Returns
    /// Number of rows added
    ///
    /// # Example
    /// ```rust,ignore
    /// let new_batch = RecordBatch::try_new(schema, columns)?;
    /// let rows_added = cube.append_rows(new_batch)?;
    /// println!("Added {} rows", rows_added);
    /// ```
    pub fn append_rows(&mut self, batch: RecordBatch) -> Result<usize> {
        // Validate schema compatibility
        updates::validate_batch_schema(&self.arrow_schema, &batch.schema())?;

        let rows_added = batch.num_rows();

        // Add the batch to our data
        self.data.push(batch);
        self.row_count += rows_added;

        Ok(rows_added)
    }

    /// Append multiple RecordBatches to the cube (incremental loading)
    ///
    /// This method adds new data incrementally by appending multiple batches.
    /// All batches must have schemas compatible with the cube's schema.
    ///
    /// # Arguments
    /// * `batches` - Vector of RecordBatches to append
    ///
    /// # Returns
    /// Total number of rows added
    ///
    /// # Example
    /// ```rust,ignore
    /// let batches = vec![batch1, batch2, batch3];
    /// let total_rows = cube.append_batches(batches)?;
    /// println!("Appended {} rows total", total_rows);
    /// ```
    pub fn append_batches(&mut self, batches: Vec<RecordBatch>) -> Result<usize> {
        if batches.is_empty() {
            return Ok(0);
        }

        // Validate all batches first
        for batch in &batches {
            updates::validate_batch_schema(&self.arrow_schema, &batch.schema())?;
        }

        // Count total rows
        let rows_added: usize = batches.iter().map(|b| b.num_rows()).sum();

        // Append all batches
        self.data.extend(batches);
        self.row_count += rows_added;

        Ok(rows_added)
    }

    /// Delete rows from the cube based on a SQL filter expression
    ///
    /// This method removes rows that match the given SQL WHERE clause predicate.
    /// Since RecordBatch is immutable, this creates new batches without the deleted rows.
    ///
    /// # Arguments
    /// * `filter_expr` - SQL WHERE clause expression (e.g., "age < 18" or "region = 'North'")
    ///
    /// # Returns
    /// Number of rows deleted
    ///
    /// # Example
    /// ```rust,ignore
    /// // Delete all rows where sales < 100
    /// let deleted = cube.delete_rows("sales < 100").await?;
    /// println!("Deleted {} rows", deleted);
    /// ```
    pub async fn delete_rows(&mut self, filter_expr: &str) -> Result<usize> {
        // We need to evaluate the filter using DataFusion to get a boolean mask
        // Then apply the inverse of that mask to keep only non-matching rows

        use datafusion::prelude::*;

        // Create a session context
        let ctx = SessionContext::new();

        // Register the current data as a table
        let table = datafusion::datasource::MemTable::try_new(
            self.arrow_schema.clone(),
            vec![self.data.clone()],
        )
        .map_err(|e| Error::query(format!("Failed to create temp table: {}", e)))?;

        ctx.register_table("temp_table", Arc::new(table))
            .map_err(|e| Error::query(format!("Failed to register table: {}", e)))?;

        // Build a query that selects all rows NOT matching the filter
        // We invert the filter by wrapping it with NOT
        let query = format!("SELECT * FROM temp_table WHERE NOT ({})", filter_expr);

        // Execute the query
        let df = ctx
            .sql(&query)
            .await
            .map_err(|e| Error::query(format!("Failed to execute delete filter: {}", e)))?;

        let results = df
            .collect()
            .await
            .map_err(|e| Error::query(format!("Failed to collect delete results: {}", e)))?;

        // Calculate rows deleted
        let new_row_count: usize = results.iter().map(|b| b.num_rows()).sum();
        let rows_deleted = self.row_count - new_row_count;

        // Update the cube data
        self.data = results;
        self.row_count = new_row_count;

        Ok(rows_deleted)
    }

    /// Update rows in the cube based on a filter and replacement batch
    ///
    /// This method updates rows matching a filter expression by:
    /// 1. Deleting rows that match the filter
    /// 2. Appending the replacement batch
    ///
    /// The replacement batch must have a schema compatible with the cube.
    ///
    /// # Arguments
    /// * `filter_expr` - SQL WHERE clause to identify rows to update
    /// * `replacement_batch` - RecordBatch containing updated rows
    ///
    /// # Returns
    /// Tuple of (rows_deleted, rows_added)
    ///
    /// # Example
    /// ```rust,ignore
    /// // Update all North region sales with new values
    /// let updated_data = create_updated_batch()?;
    /// let (deleted, added) = cube.update_rows("region = 'North'", updated_data).await?;
    /// println!("Updated {} rows", deleted);
    /// ```
    pub async fn update_rows(
        &mut self,
        filter_expr: &str,
        replacement_batch: RecordBatch,
    ) -> Result<(usize, usize)> {
        // Validate the replacement batch schema
        updates::validate_batch_schema(&self.arrow_schema, &replacement_batch.schema())?;

        // Delete matching rows
        let rows_deleted = self.delete_rows(filter_expr).await?;

        // Append the replacement batch
        let rows_added = self.append_rows(replacement_batch)?;

        Ok((rows_deleted, rows_added))
    }

    /// Consolidate all data batches into a single batch
    ///
    /// This operation can improve query performance by reducing the number of
    /// batches, but may increase memory usage temporarily during consolidation.
    ///
    /// # Returns
    /// Number of batches before consolidation
    ///
    /// # Example
    /// ```rust,ignore
    /// let old_batch_count = cube.consolidate_batches()?;
    /// println!("Consolidated from {} batches to 1 batch", old_batch_count);
    /// ```
    pub fn consolidate_batches(&mut self) -> Result<usize> {
        let old_batch_count = self.data.len();

        if old_batch_count <= 1 {
            return Ok(old_batch_count);
        }

        // Concatenate all batches into one
        let consolidated = updates::concat_record_batches(&self.arrow_schema, &self.data)?;

        self.data = vec![consolidated];

        Ok(old_batch_count)
    }

    /// Get the number of data batches in the cube
    ///
    /// Useful for monitoring fragmentation and deciding when to consolidate.
    pub fn batch_count(&self) -> usize {
        self.data.len()
    }
}
