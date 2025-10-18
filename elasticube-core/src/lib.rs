//! ElastiCube Core Library
//!
//! A high-performance, embeddable OLAP cube builder and query library built on Apache Arrow.
//!
//! # Features
//!
//! - **Columnar Storage**: Efficient field-by-field storage using Apache Arrow
//! - **No Pre-Aggregation**: Query raw data with dynamic aggregations
//! - **Multi-Source**: Combine data from CSV, Parquet, JSON, and custom sources
//! - **Fast**: Near C-level performance with parallel query execution
//!
//! # Example
//!
//! ```rust,ignore
//! use elasticube_core::{ElastiCubeBuilder, AggFunc};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let cube = ElastiCubeBuilder::new()
//!         .add_dimension("region", DataType::Utf8)
//!         .add_measure("sales", DataType::Float64, AggFunc::Sum)
//!         .load_csv("data.csv")?
//!         .build()?;
//!
//!     let results = cube.query()
//!         .select(&["region", "sum(sales)"])
//!         .group_by(&["region"])
//!         .execute()
//!         .await?;
//!
//!     Ok(())
//! }
//! ```

pub mod builder;
pub mod cache;
pub mod cube;
pub mod error;
pub mod optimization;
pub mod query;
pub mod storage;
pub mod sources;

#[cfg(test)]
mod query_materialization_tests;

#[cfg(test)]
mod cube_update_tests;

// Re-export commonly used types
pub use builder::ElastiCubeBuilder;
pub use cache::{CacheStats, QueryCache, QueryCacheKey};
pub use cube::{
    AggFunc, CalculatedMeasure, CubeSchema, Dimension, ElastiCube, Hierarchy, Measure,
    VirtualDimension,
};
pub use error::{Error, Result};
pub use optimization::{ColumnStatistics, CubeStatistics, OptimizationConfig};
pub use query::{QueryBuilder, QueryResult};
pub use sources::{CsvSource, DataSource, JsonSource, ParquetSource, RecordBatchSource};
