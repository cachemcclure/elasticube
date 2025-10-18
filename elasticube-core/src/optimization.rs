//! Performance optimization features for ElastiCube
//!
//! Provides configuration for query optimization, storage optimization,
//! and caching to improve analytical query performance.

use datafusion::execution::config::SessionConfig;
use datafusion::execution::runtime_env::RuntimeEnv;
use std::sync::Arc;

/// Configuration for query optimization
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    /// Target number of partitions for parallel query execution
    /// Higher values enable more parallelism but increase overhead
    /// Default: number of CPU cores
    pub target_partitions: usize,

    /// Enable/disable query optimization
    /// Default: true
    pub enable_optimizer: bool,

    /// Enable/disable predicate pushdown optimization
    /// Pushes filters as early as possible in the query plan
    /// Default: true
    pub enable_predicate_pushdown: bool,

    /// Enable/disable projection pushdown optimization
    /// Only reads columns that are actually needed
    /// Default: true
    pub enable_projection_pushdown: bool,

    /// Enable/disable filter pushdown to Parquet readers
    /// Uses Parquet row group statistics to skip reading unnecessary data
    /// Default: true
    pub enable_parquet_pushdown: bool,

    /// Batch size for query execution
    /// Larger batches improve throughput but use more memory
    /// Default: 8192
    pub batch_size: usize,

    /// Enable query result caching
    /// Default: true
    pub enable_query_cache: bool,

    /// Maximum number of cached query results
    /// Default: 100
    pub max_cache_entries: usize,

    /// Memory limit for query execution (in bytes)
    /// None means unlimited
    /// Default: None
    pub memory_limit: Option<usize>,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            target_partitions: num_cpus::get(),
            enable_optimizer: true,
            enable_predicate_pushdown: true,
            enable_projection_pushdown: true,
            enable_parquet_pushdown: true,
            batch_size: 8192,
            enable_query_cache: true,
            max_cache_entries: 100,
            memory_limit: None,
        }
    }
}

impl OptimizationConfig {
    /// Create a new optimization configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set target partitions for parallel execution
    pub fn with_target_partitions(mut self, partitions: usize) -> Self {
        self.target_partitions = partitions;
        self
    }

    /// Set batch size for query execution
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Enable or disable predicate pushdown
    pub fn with_predicate_pushdown(mut self, enabled: bool) -> Self {
        self.enable_predicate_pushdown = enabled;
        self
    }

    /// Enable or disable projection pushdown
    pub fn with_projection_pushdown(mut self, enabled: bool) -> Self {
        self.enable_projection_pushdown = enabled;
        self
    }

    /// Enable or disable Parquet pushdown optimizations
    pub fn with_parquet_pushdown(mut self, enabled: bool) -> Self {
        self.enable_parquet_pushdown = enabled;
        self
    }

    /// Set memory limit for query execution
    pub fn with_memory_limit(mut self, limit: usize) -> Self {
        self.memory_limit = Some(limit);
        self
    }

    /// Enable or disable query result caching
    pub fn with_query_cache(mut self, enabled: bool) -> Self {
        self.enable_query_cache = enabled;
        self
    }

    /// Set maximum number of cached query results
    pub fn with_max_cache_entries(mut self, max: usize) -> Self {
        self.max_cache_entries = max;
        self
    }

    /// Create a DataFusion SessionConfig from this optimization config
    pub fn to_session_config(&self) -> SessionConfig {
        let config = SessionConfig::new()
            .with_target_partitions(self.target_partitions)
            .with_batch_size(self.batch_size);

        // Note: DataFusion 50+ has different APIs for optimizer rules
        // The optimizer rules are enabled by default
        // We can configure them via SessionConfig options if needed

        config
    }

    /// Create a DataFusion RuntimeEnv from this optimization config
    pub fn to_runtime_env(&self) -> Arc<RuntimeEnv> {
        // DataFusion 50+ uses RuntimeEnv::default() or RuntimeEnv::new()
        // Memory limits are configured differently in newer versions
        // For now, we'll use the default RuntimeEnv
        Arc::new(RuntimeEnv::default())
    }
}

/// Statistics for a cube's data
#[derive(Debug, Clone)]
pub struct CubeStatistics {
    /// Total number of rows
    pub row_count: usize,

    /// Number of RecordBatch partitions
    pub partition_count: usize,

    /// Average rows per partition
    pub avg_rows_per_partition: usize,

    /// Total memory usage (estimated)
    pub memory_bytes: usize,

    /// Per-column statistics
    pub column_stats: Vec<ColumnStatistics>,
}

impl CubeStatistics {
    /// Create statistics from cube data
    pub fn from_batches(batches: &[arrow::record_batch::RecordBatch]) -> Self {
        let row_count: usize = batches.iter().map(|b| b.num_rows()).sum();
        let partition_count = batches.len();
        let avg_rows_per_partition = if partition_count > 0 {
            row_count / partition_count
        } else {
            0
        };

        // Estimate memory usage
        let memory_bytes: usize = batches
            .iter()
            .map(|b| b.get_array_memory_size())
            .sum();

        // Collect column statistics
        let column_stats = if let Some(first_batch) = batches.first() {
            let schema = first_batch.schema();
            (0..schema.fields().len())
                .map(|col_idx| ColumnStatistics::from_batches(batches, col_idx))
                .collect()
        } else {
            Vec::new()
        };

        Self {
            row_count,
            partition_count,
            avg_rows_per_partition,
            memory_bytes,
            column_stats,
        }
    }

    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "Rows: {}, Partitions: {}, Memory: {:.2} MB",
            self.row_count,
            self.partition_count,
            self.memory_bytes as f64 / 1_048_576.0
        )
    }
}

/// Statistics for a single column
#[derive(Debug, Clone)]
pub struct ColumnStatistics {
    /// Column index
    pub column_index: usize,

    /// Column name
    pub column_name: String,

    /// Number of null values
    pub null_count: usize,

    /// Null percentage
    pub null_percentage: f64,

    /// Estimated distinct values (cardinality)
    /// None if not computed
    pub distinct_count: Option<usize>,
}

impl ColumnStatistics {
    /// Compute statistics for a column across all batches
    fn from_batches(batches: &[arrow::record_batch::RecordBatch], col_idx: usize) -> Self {
        let schema = batches.first().map(|b| b.schema()).unwrap();
        let column_name = schema.field(col_idx).name().clone();

        let mut total_nulls = 0;
        let mut total_rows = 0;

        for batch in batches {
            let array = batch.column(col_idx);
            total_nulls += array.null_count();
            total_rows += array.len();
        }

        let null_percentage = if total_rows > 0 {
            (total_nulls as f64 / total_rows as f64) * 100.0
        } else {
            0.0
        };

        Self {
            column_index: col_idx,
            column_name,
            null_count: total_nulls,
            null_percentage,
            distinct_count: None, // Computing distinct count is expensive, skip for now
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_config_default() {
        let config = OptimizationConfig::default();
        assert!(config.enable_optimizer);
        assert!(config.enable_predicate_pushdown);
        assert!(config.enable_projection_pushdown);
        assert_eq!(config.batch_size, 8192);
    }

    #[test]
    fn test_optimization_config_builder() {
        let config = OptimizationConfig::new()
            .with_target_partitions(8)
            .with_batch_size(4096)
            .with_predicate_pushdown(false)
            .with_memory_limit(1_000_000_000);

        assert_eq!(config.target_partitions, 8);
        assert_eq!(config.batch_size, 4096);
        assert!(!config.enable_predicate_pushdown);
        assert_eq!(config.memory_limit, Some(1_000_000_000));
    }

    #[test]
    fn test_session_config_creation() {
        let config = OptimizationConfig::new()
            .with_target_partitions(4)
            .with_batch_size(1024);

        let session_config = config.to_session_config();
        assert_eq!(session_config.target_partitions(), 4);
        assert_eq!(session_config.batch_size(), 1024);
    }
}
