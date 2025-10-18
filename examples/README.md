# ElastiCube Examples

This directory contains comprehensive examples demonstrating various features and use cases of the ElastiCube library.

## Quick Start

Run any example with:
```bash
cargo run --example <example_name>
```

For examples requiring optional features:
```bash
cargo run --example <example_name> --features all-sources
```

## Examples Overview

### 1. Basic Cube Building (`basic_cube_building.rs`)

**Purpose**: Introduction to ElastiCube fundamentals

**Topics Covered**:
- Creating cubes with dimensions and measures
- Loading data from CSV files
- Basic query operations (SELECT, GROUP BY, ORDER BY)
- Filtering with WHERE clauses
- Direct SQL queries
- Multi-dimensional aggregations

**Run with**:
```bash
cargo run --example basic_cube_building
```

**Key Takeaways**:
- ElastiCubeBuilder fluent API
- Difference between dimensions (for slicing) and measures (for aggregation)
- Multiple query styles (fluent API vs SQL)

---

### 2. Query Demo (`query_demo.rs`)

**Purpose**: Comprehensive query capabilities demonstration

**Topics Covered**:
- SQL queries
- Fluent API queries
- OLAP operations (slice, dice, drill-down, roll-up)
- Complex aggregations and filtering
- Query result handling

**Run with**:
```bash
cargo run --example query_demo
```

**Key Takeaways**:
- Full query API surface
- OLAP-specific operations
- Performance of different query patterns

---

### 3. Sales Analytics (`sales_analytics.rs`)

**Purpose**: Real-world business analytics use case

**Topics Covered**:
- Customer segmentation analysis
- Geographic sales analysis
- Product performance tracking
- KPI calculations (revenue, profit, margin)
- Discount impact analysis
- Monthly trend analysis
- High-value transaction identification

**Run with**:
```bash
cargo run --example sales_analytics
```

**Key Takeaways**:
- Calculated measures for derived KPIs
- Multi-dimensional business analysis
- Complex SQL with CASE statements
- Practical analytics patterns

---

### 4. Time-Series Analysis (`time_series_analysis.rs`)

**Purpose**: Time-series and IoT sensor data analytics

**Topics Covered**:
- Temporal data handling (daily, weekly, monthly)
- Virtual dimensions for time extraction
- Trend analysis and pattern detection
- Anomaly detection
- Time-based filtering and windowing
- Period-over-period comparisons

**Run with**:
```bash
cargo run --example time_series_analysis
```

**Key Takeaways**:
- Time-based aggregations
- Virtual dimensions for date/time parsing
- Temporal filtering patterns
- IoT and sensor data use cases

---

### 5. Calculated Fields Demo (`calculated_fields_demo.rs`)

**Purpose**: Demonstrates calculated measures and virtual dimensions

**Topics Covered**:
- Calculated measures (derived from expressions)
- Virtual dimensions (computed fields)
- Schema integration and validation
- Builder pattern for calculated fields

**Run with**:
```bash
cargo run --example calculated_fields_demo
```

**Key Takeaways**:
- Creating KPIs from raw data
- SQL expressions in schema definitions
- Automatic field materialization

---

### 6. Query Materialization Demo (`query_materialization_demo.rs`)

**Purpose**: Shows how calculated fields are expanded in queries

**Topics Covered**:
- Automatic expansion of calculated measures
- Recursive field expansion
- Using calculated fields in WHERE, SELECT, GROUP BY, ORDER BY
- Nested calculated measures

**Run with**:
```bash
cargo run --example query_materialization_demo
```

**Key Takeaways**:
- Calculated fields work seamlessly in queries
- No manual expansion required
- Performance implications of complex calculations

---

### 7. Data Updates Demo (`data_updates_demo.rs`)

**Purpose**: Demonstrates data modification operations

**Topics Covered**:
- Incremental data loading (append)
- Updating existing rows
- Deleting rows by filter
- Batch consolidation
- Python bindings for updates

**Run with**:
```bash
cargo run --example data_updates_demo
```

**Key Takeaways**:
- Cubes are mutable
- Update operations preserve schema
- Batch management for performance

---

### 8. Multi-Source Demo (`multi_source_demo.rs`)

**Purpose**: Loading data from multiple heterogeneous sources

**Topics Covered**:
- Database connectors (PostgreSQL, MySQL, ODBC)
- REST API data sources
- Combining data from different sources
- Feature flags for optional dependencies

**Run with**:
```bash
cargo run --example multi_source_demo --features all-sources
```

**Requirements**:
- Requires `all-sources` feature flag
- Demonstrates database and REST API connectivity

**Key Takeaways**:
- Flexible data source architecture
- DataSource trait for custom sources
- Real-world data integration patterns

---

### 9. Object Storage Demo (`object_storage_demo.rs`)

**Purpose**: Cloud object storage integration (S3, GCS, Azure)

**Topics Covered**:
- AWS S3 connector
- Google Cloud Storage (GCS) connector
- Azure Blob Storage connector
- Multiple file format support (Parquet, CSV, JSON)
- Authentication methods (credentials, environment vars)

**Run with**:
```bash
cargo run --example object_storage_demo --features object-storage
```

**Requirements**:
- Requires `object-storage` feature flag
- Cloud credentials (or uses mock data)

**Key Takeaways**:
- Cloud-native data access
- S3-compatible storage support
- Multi-cloud architecture

---

## Example Dependencies

| Example | Required Features | External Dependencies |
|---------|------------------|----------------------|
| basic_cube_building | None | None |
| query_demo | None | None |
| sales_analytics | None | None |
| time_series_analysis | None | None |
| calculated_fields_demo | None | None |
| query_materialization_demo | None | None |
| data_updates_demo | None | None |
| multi_source_demo | `all-sources` or `database` + `rest-api` | Database server (optional) |
| object_storage_demo | `object-storage` | Cloud credentials (optional) |

## Running All Examples

To run all examples sequentially:

```bash
# Basic examples (no features required)
cargo run --example basic_cube_building
cargo run --example query_demo
cargo run --example sales_analytics
cargo run --example time_series_analysis
cargo run --example calculated_fields_demo
cargo run --example query_materialization_demo
cargo run --example data_updates_demo

# Advanced examples (require features)
cargo run --example multi_source_demo --features all-sources
cargo run --example object_storage_demo --features object-storage
```

## Learning Path

**Recommended order for beginners**:

1. **basic_cube_building** - Start here to understand core concepts
2. **query_demo** - Learn the full query API
3. **sales_analytics** - See practical business use case
4. **calculated_fields_demo** - Add computed fields
5. **query_materialization_demo** - Understand field expansion
6. **time_series_analysis** - Work with temporal data
7. **data_updates_demo** - Modify cube data
8. **multi_source_demo** - Integrate multiple sources
9. **object_storage_demo** - Access cloud storage

## Creating Your Own Examples

To create a new example:

1. Create a new file in `examples/` directory:
   ```bash
   touch examples/my_example.rs
   ```

2. Add the standard structure:
   ```rust
   //! My Example
   //!
   //! Description of what this example demonstrates.

   use elasticube_core::{ElastiCubeBuilder, Result};

   #[tokio::main]
   async fn main() -> Result<()> {
       println!("=== My Example ===\n");
       // Your code here
       Ok(())
   }
   ```

3. Run your example:
   ```bash
   cargo run --example my_example
   ```

## Troubleshooting

**Issue**: "Example not found"
- **Solution**: Ensure the file is in the `examples/` directory and has a `.rs` extension

**Issue**: "Feature X not enabled"
- **Solution**: Add `--features <feature_name>` to your cargo command

**Issue**: "Cannot connect to database/cloud"
- **Solution**: Examples with external dependencies use mock data when connections fail

## Additional Resources

- **API Documentation**: `cargo doc --no-deps --open`
- **Main README**: `../README.md`
- **User Guide**: `../docs/USER_GUIDE.md` (if available)
- **Project Checklist**: `../PROJECT_CHECKLIST.md`

## Contributing Examples

We welcome new examples! Ideal examples:
- Solve a real-world problem
- Demonstrate a specific feature clearly
- Include comments explaining key concepts
- Are self-contained and runnable
- Follow the existing code style

Submit examples via pull request with:
- The example code
- Updates to this README
- Test that it compiles and runs
