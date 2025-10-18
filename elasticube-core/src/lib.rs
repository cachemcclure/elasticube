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

// Re-export database sources when feature is enabled
/// Database source connectors (PostgreSQL, MySQL, SQL Server, etc.)
///
/// These types are only available when the `database` feature is enabled:
/// ```toml
/// [dependencies]
/// elasticube-core = { version = "0.2", features = ["database"] }
/// ```
///
/// See [`ElastiCubeBuilder::load_postgres`], [`ElastiCubeBuilder::load_mysql`],
/// and [`ElastiCubeBuilder::load_odbc`] for usage examples.
#[cfg(feature = "database")]
pub use sources::database::{MySqlSource, OdbcSource, PostgresSource};

// Re-export REST API sources when feature is enabled
/// REST API data source connector
///
/// This type is only available when the `rest-api` feature is enabled:
/// ```toml
/// [dependencies]
/// elasticube-core = { version = "0.2", features = ["rest-api"] }
/// ```
///
/// See [`ElastiCubeBuilder::load_rest_api`] for usage examples.
#[cfg(feature = "rest-api")]
pub use sources::rest::{HttpMethod, RestApiSource};

// Re-export object storage sources when feature is enabled
/// Object storage source connectors (AWS S3, Google Cloud Storage, Azure Blob Storage)
///
/// These types are only available when the `object-storage` feature is enabled:
/// ```toml
/// [dependencies]
/// elasticube-core = { version = "0.2", features = ["object-storage"] }
/// ```
///
/// Or enable all data sources at once:
/// ```toml
/// [dependencies]
/// elasticube-core = { version = "0.2", features = ["all-sources"] }
/// ```
///
/// See [`ElastiCubeBuilder::load_s3`], [`ElastiCubeBuilder::load_gcs`],
/// and [`ElastiCubeBuilder::load_azure`] for usage examples.
#[cfg(feature = "object-storage")]
pub use sources::object_storage::{AzureSource, GcsSource, ObjectStorageSource, S3Source, StorageFileFormat};
