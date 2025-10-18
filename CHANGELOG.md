# Changelog

All notable changes to ElastiCube will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-10-18

### Added

- API Review and Stabilization (Phase 8.1)
  - Comprehensive API review document (`docs/API_REVIEW.md`)
  - Detailed compliance check against Rust API Guidelines
  - Semantic versioning plan for path to 1.0

### Changed

- **BREAKING (Minor)**: Added `#[non_exhaustive]` to `Error` enum
  - Error matching now requires a catch-all pattern (`_ => ...`)
  - Allows adding new error variants without breaking changes in future versions
  - Migration: Add `_ => ...` arm to any `match` statements on `Error`

- Improved trait implementations:
  - `OptimizationConfig` now implements `PartialEq` and `Eq`
  - `CacheStats` now implements `PartialEq`
  - Enables better testing and comparison of configuration/stats

- Enhanced documentation:
  - Added detailed explanation of `Arc<ElastiCube>` requirement for query methods
  - Added comprehensive examples showing Arc usage pattern
  - Documented feature gate requirements for optional data sources
  - Improved doc comments for database, REST API, and object storage sources

### Fixed

- Minor dead code warnings in internal modules

## [0.1.0] - 2025-10-18

### Added

- **Phase 1**: Foundation & Core Infrastructure
  - Core data structures: `Dimension`, `Measure`, `Hierarchy`, `CubeSchema`
  - Comprehensive error handling with `thiserror`
  - Apache Arrow integration for columnar storage

- **Phase 2**: Cube Builder
  - Fluent builder API (`ElastiCubeBuilder`)
  - CSV, Parquet, JSON, and in-memory data source connectors
  - Schema validation and inference

- **Phase 3**: Query Engine
  - DataFusion-powered query execution
  - Fluent query API (`QueryBuilder`)
  - SQL query support
  - OLAP operations: slice, dice, drill-down, roll-up
  - Aggregation functions: SUM, AVG, MIN, MAX, COUNT, COUNT DISTINCT

- **Phase 4**: Performance Optimization
  - Query optimization configuration (`OptimizationConfig`)
  - LRU query result caching (`QueryCache`)
  - Cube statistics tracking (`CubeStatistics`)
  - Parallel query execution
  - Predicate and projection pushdown

- **Phase 5**: Python Bindings
  - PyO3-based Python API (`elasticube-py`)
  - Pandas and Polars DataFrame integration
  - Jupyter notebook support
  - Visualization support (matplotlib/seaborn)
  - Parquet-based serialization

- **Phase 6**: Advanced Features
  - **6.1**: Calculated measures and virtual dimensions
    - SQL expression-based computed fields
    - Automatic query materialization
  - **6.2**: Data update operations
    - Append, update, delete, and consolidate operations
    - Incremental data loading
  - **6.3**: Serialization
    - Save/load cube state to Parquet
    - Metadata versioning
  - **6.4**: Multi-source support
    - Database connectors (PostgreSQL, MySQL, ODBC) - feature gated
    - REST API data source - feature gated
    - Object storage (AWS S3, GCS, Azure Blob) - feature gated
    - Feature flags: `database`, `rest-api`, `object-storage`, `all-sources`

- **Phase 7**: Testing & Documentation
  - **7.1**: Testing infrastructure
    - 90 unit tests across all modules
    - 17 integration tests for query execution
    - 6 property-based tests with quickcheck
    - 12 benchmark groups with Criterion
    - 3 reusable test datasets
  - **7.2**: Documentation
    - Comprehensive rustdoc API documentation (193+ doc comments)
    - User guide (600+ lines covering all features)
    - Performance tuning guide
  - **7.3**: Examples
    - 9 runnable examples demonstrating all features
    - Examples: basic usage, sales analytics, time-series analysis
    - Multi-source and object storage demos

### Technical Details

- **Dependencies**:
  - Apache Arrow 56.2.0
  - DataFusion 50.2.0
  - PyO3 0.26 (for Python bindings)
  - arrow-odbc 20.1.0 (optional)
  - reqwest 0.12 (optional)
  - object_store 0.11 (optional)

- **Rust Edition**: 2021
- **MSRV**: 1.90.0
- **License**: MIT OR Apache-2.0

---

## Version History

- **0.2.0** (2025-10-18) - API Stabilization & Documentation
- **0.1.0** (2025-10-18) - Initial release with complete feature set

---

## Upgrade Guide

### Upgrading from 0.1.0 to 0.2.0

The upgrade is mostly seamless. The only breaking change affects error handling:

**Error Matching**:

```rust
// Before (0.1.0) - exhaustive matching
match error {
    Error::Arrow(e) => handle_arrow(e),
    Error::DataFusion(e) => handle_datafusion(e),
    Error::Schema(msg) => handle_schema(msg),
    // ... must list all variants
}

// After (0.2.0) - must include catch-all
match error {
    Error::Arrow(e) => handle_arrow(e),
    Error::DataFusion(e) => handle_datafusion(e),
    Error::Schema(msg) => handle_schema(msg),
    _ => handle_other(error), // Required!
}
```

**Recommended**: Use `if let` for specific error handling:

```rust
// Best practice (works in both versions)
if let Error::Schema(msg) = error {
    handle_schema(msg)
} else {
    handle_other(error)
}
```

---

## Links

- [Repository](https://github.com/cachemcclure/elasticube)
- [Documentation](https://docs.rs/elasticube-core)
- [Issue Tracker](https://github.com/cachemcclure/elasticube/issues)
