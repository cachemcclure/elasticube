# ElastiCube Library - User Guide

**Version**: 0.1.0
**Last Updated**: October 2025

## Table of Contents

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [Core Concepts](#core-concepts)
5. [Building Cubes](#building-cubes)
6. [Querying Data](#querying-data)
7. [Advanced Features](#advanced-features)
8. [Performance Tuning](#performance-tuning)
9. [Best Practices](#best-practices)
10. [Troubleshooting](#troubleshooting)

---

## Introduction

ElastiCube is a high-performance, embeddable OLAP (Online Analytical Processing) cube library built in Rust with Python bindings. It provides fast, in-memory multidimensional analytical processing without requiring pre-aggregation or external services.

### Key Features

- **Columnar Storage**: Efficient field-by-field storage using Apache Arrow
- **No Pre-Aggregation**: Query raw data with dynamic aggregations
- **Multi-Source**: Load data from CSV, Parquet, JSON, databases, REST APIs, and cloud storage
- **Embeddable**: Pure Rust library with no mandatory cloud dependencies
- **Fast**: Near C-level performance with parallel query execution
- **Flexible**: Use from Rust or Python

### When to Use ElastiCube

**Ideal for**:
- Analytics dashboards and reporting
- Business intelligence applications
- Time-series analysis
- IoT sensor data aggregation
- Multi-dimensional data exploration
- Embedded analytics in applications

**Not ideal for**:
- Transaction processing (OLTP)
- Very large datasets that don't fit in memory (>100GB)
- Real-time streaming (batch-oriented)

---

## Installation

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
elasticube-core = "0.1.0"

# Optional features
# elasticube-core = { version = "0.1.0", features = ["database", "rest-api", "object-storage"] }
# Or enable all sources:
# elasticube-core = { version = "0.1.0", features = ["all-sources"] }
```

### Python

```bash
pip install elasticube
```

---

## Quick Start

### Rust Quick Start

```rust
use elasticube_core::{ElastiCubeBuilder, AggFunc, Result};
use arrow_schema::DataType;

#[tokio::main]
async fn main() -> Result<()> {
    // Build a cube from CSV data
    let cube = ElastiCubeBuilder::new("sales")
        .add_dimension("region", DataType::Utf8)?
        .add_measure("revenue", DataType::Float64, AggFunc::Sum)?
        .load_csv("sales_data.csv")
        .build()?;

    // Query the cube
    let result = cube.query()?
        .select(&["region", "sum(revenue)"])
        .group_by(&["region"])
        .execute()
        .await?;

    println!("{}", result);
    Ok(())
}
```

### Python Quick Start

```python
from elasticube import ElastiCube

# Build a cube
cube = ElastiCube.builder() \
    .add_dimension("region") \
    .add_measure("revenue", agg="sum") \
    .load_csv("sales_data.csv") \
    .build()

# Query with Pandas
df = cube.query() \
    .select(["region", "sum(revenue)"]) \
    .group_by(["region"]) \
    .to_pandas()

print(df)
```

---

## Core Concepts

### 1. Dimensions

**Dimensions** are categorical or ordinal fields used for slicing and dicing data.

**Examples**:
- Date/Time (year, month, day)
- Geography (country, region, city)
- Products (category, SKU, brand)
- Customers (segment, type, tier)

**Characteristics**:
- Low-to-medium cardinality
- Used in GROUP BY and WHERE clauses
- Form the "axes" of the cube

```rust
.add_dimension("region", DataType::Utf8)?
.add_dimension("product_category", DataType::Utf8)?
.add_dimension("date", DataType::Date32)?
```

### 2. Measures

**Measures** are numeric fields that are aggregated.

**Examples**:
- Sales revenue
- Quantity sold
- Cost
- Profit

**Characteristics**:
- Numeric data types (Float64, Int64, etc.)
- Associated with an aggregation function
- Calculated during queries

**Aggregation Functions**:
- `AggFunc::Sum` - Total sum
- `AggFunc::Avg` - Average value
- `AggFunc::Min` - Minimum value
- `AggFunc::Max` - Maximum value
- `AggFunc::Count` - Count of values

```rust
.add_measure("sales", DataType::Float64, AggFunc::Sum)?
.add_measure("quantity", DataType::Int64, AggFunc::Sum)?
.add_measure("price", DataType::Float64, AggFunc::Avg)?
```

### 3. Hierarchies

**Hierarchies** define drill-down paths through dimensions.

**Example**: Time Hierarchy
```
Year → Quarter → Month → Day
```

**Example**: Geography Hierarchy
```
Country → State → City → Store
```

```rust
.add_hierarchy("time", vec!["year".to_string(), "quarter".to_string(), "month".to_string()])?
.add_hierarchy("geography", vec!["country".to_string(), "state".to_string(), "city".to_string()])?
```

### 4. Calculated Measures

**Calculated measures** are derived from expressions.

**Examples**:
- Profit = Revenue - Cost
- Margin% = (Revenue - Cost) / Revenue × 100
- Average Order Value = Revenue / Orders

```rust
.add_calculated_measure(
    "profit",
    "revenue - cost",
    DataType::Float64,
    AggFunc::Sum
)?
```

### 5. Virtual Dimensions

**Virtual dimensions** are computed from other fields.

**Examples**:
- Year/Month extracted from timestamp
- Age bucket from birth date
- Price tier from price value

```rust
.add_virtual_dimension(
    "year_month",
    "SUBSTRING(timestamp, 1, 7)",
    DataType::Utf8,
    Some(12) // cardinality estimate
)?
```

---

## Building Cubes

### From CSV

```rust
let cube = ElastiCubeBuilder::new("my_cube")
    .add_dimension("category", DataType::Utf8)?
    .add_measure("sales", DataType::Float64, AggFunc::Sum)?
    .load_csv("data.csv")
    .build()?;
```

**CSV Options**:
```rust
use elasticube_core::sources::CsvSource;

let csv_source = CsvSource::new("data.csv")
    .with_header(true)
    .with_delimiter(b',')
    .with_batch_size(10000);

let cube = ElastiCubeBuilder::new("my_cube")
    .load_from(Box::new(csv_source))
    .build()?;
```

### From Parquet

```rust
let cube = ElastiCubeBuilder::new("my_cube")
    .load_parquet("data.parquet")
    .build()?;
```

### From JSON

```rust
let cube = ElastiCubeBuilder::new("my_cube")
    .load_json("data.json")
    .build()?;
```

### From Database

Requires `database` feature:

```rust
let cube = ElastiCubeBuilder::new("my_cube")
    .load_postgres(
        "localhost",
        "mydb",
        "user",
        "password",
        "SELECT * FROM sales"
    )
    .build()?;
```

### From REST API

Requires `rest-api` feature:

```rust
let cube = ElastiCubeBuilder::new("my_cube")
    .load_rest_api("https://api.example.com/data")
    .build()?;
```

### From Cloud Storage

Requires `object-storage` feature:

```rust
// AWS S3
let cube = ElastiCubeBuilder::new("my_cube")
    .load_s3("my-bucket", "path/to/data.parquet", "us-east-1")
    .build()?;

// Google Cloud Storage
let cube = ElastiCubeBuilder::new("my_cube")
    .load_gcs("my-bucket", "path/to/data.parquet", "/path/to/service-account.json")
    .build()?;

// Azure Blob Storage
let cube = ElastiCubeBuilder::new("my_cube")
    .load_azure("my-container", "path/to/data.parquet", "account-name", "access-key")
    .build()?;
```

### From In-Memory Data

```rust
use arrow::record_batch::RecordBatch;

let batch: RecordBatch = /* ... */;

let cube = ElastiCubeBuilder::new("my_cube")
    .with_data(vec![batch])?
    .build()?;
```

---

## Querying Data

### Fluent API

```rust
let result = cube.query()?
    .select(&["region", "sum(sales) as total_sales"])
    .filter("date >= '2024-01-01'")
    .group_by(&["region"])
    .order_by(&["total_sales DESC"])
    .limit(10)
    .execute()
    .await?;
```

### SQL Queries

```rust
let result = cube.query()?
    .sql("SELECT region, SUM(sales) as total FROM cube GROUP BY region")
    .execute()
    .await?;
```

**Note**: The cube is always referenced as `cube` in SQL queries.

### OLAP Operations

#### Slice (filter on one dimension)

```rust
let result = cube.query()?
    .slice("region", "North America")
    .select(&["product", "sum(sales)"])
    .group_by(&["product"])
    .execute()
    .await?;
```

#### Dice (filter on multiple dimensions)

```rust
let result = cube.query()?
    .dice(&[
        ("region", "North America"),
        ("category", "Electronics")
    ])
    .select(&["product", "sum(sales)"])
    .group_by(&["product"])
    .execute()
    .await?;
```

#### Roll-Up (aggregate across dimensions)

```rust
let result = cube.query()?
    .roll_up(&["region"]) // Aggregate by region only
    .select(&["region", "sum(sales)"])
    .group_by(&["region"])
    .execute()
    .await?;
```

### Aggregation Functions

```rust
let result = cube.query()?
    .select(&[
        "region",
        "sum(sales) as total_sales",
        "avg(sales) as avg_sales",
        "min(sales) as min_sales",
        "max(sales) as max_sales",
        "count(sales) as num_transactions",
        "count(DISTINCT customer_id) as unique_customers"
    ])
    .group_by(&["region"])
    .execute()
    .await?;
```

### Working with Query Results

```rust
let result = cube.query()?./* ... */.execute().await?;

// Print results (pretty-printed table)
println!("{}", result);

// Access raw data
for batch in result.batches() {
    println!("Batch with {} rows", batch.num_rows());
}

// Get total row count
println!("Total rows: {}", result.row_count());
```

---

## Advanced Features

### Data Updates

```rust
use arrow::array::{Float64Array, StringArray};
use arrow::record_batch::RecordBatch;

// Append new rows
let new_batch = /* create RecordBatch */;
cube.append_batches(vec![new_batch])?;

// Delete rows by filter
cube.delete_rows("date < '2024-01-01'")?;

// Update rows
let update_batch = /* create RecordBatch with new values */;
cube.update_rows("region = 'North'", update_batch)?;

// Consolidate batches for better performance
cube.consolidate_batches()?;
```

### Caching

```rust
use elasticube_core::optimization::OptimizationConfig;

let config = OptimizationConfig::default()
    .with_query_cache(true)
    .with_max_cache_entries(1000);

let cube = ElastiCubeBuilder::new("my_cube")
    .with_optimization_config(config)
    .load_csv("data.csv")
    .build()?;
```

### Statistics

```rust
use elasticube_core::CubeStatistics;

let stats = CubeStatistics::from_cube(&cube)?;

println!("Total memory: {} bytes", stats.total_memory_bytes());
println!("Null counts: {:?}", stats.null_counts());
println!("Cardinality estimates: {:?}", stats.cardinality_estimates());
```

---

## Performance Tuning

### 1. Batch Size

Adjust batch size for better memory/performance trade-offs:

```rust
use elasticube_core::sources::CsvSource;

let source = CsvSource::new("data.csv")
    .with_batch_size(16384); // Larger batches = better throughput

let cube = ElastiCubeBuilder::new("my_cube")
    .load_from(Box::new(source))
    .build()?;
```

### 2. Parallel Execution

```rust
use elasticube_core::optimization::OptimizationConfig;

let config = OptimizationConfig::default()
    .with_target_partitions(8); // Use 8 CPU cores

let cube = ElastiCubeBuilder::new("my_cube")
    .with_optimization_config(config)
    .load_csv("data.csv")
    .build()?;
```

### 3. Query Caching

Enable query result caching for repeated queries:

```rust
let config = OptimizationConfig::default()
    .with_query_cache(true)
    .with_max_cache_entries(500);
```

### 4. Batch Consolidation

Consolidate multiple small batches into larger ones:

```rust
// After many append operations
cube.consolidate_batches()?;
```

### 5. Schema Design

**Best Practices**:
- Use appropriate data types (Int32 vs Int64)
- Limit dimension cardinality when possible
- Pre-calculate complex expressions as calculated measures
- Use hierarchies for common drill-down paths

---

## Best Practices

### 1. Schema Design

✅ **DO**:
- Define dimensions for all fields used in GROUP BY
- Define measures for all numeric fields to aggregate
- Use calculated measures for derived KPIs
- Create hierarchies for common drill-down patterns

❌ **DON'T**:
- Mix dimensions and measures
- Use high-cardinality dimensions (>1M unique values)
- Store pre-aggregated data (let ElastiCube aggregate)

### 2. Query Patterns

✅ **DO**:
- Use specific column selections (avoid SELECT *)
- Apply filters early (push down predicates)
- Use appropriate aggregation functions
- Leverage query cache for repeated queries

❌ **DON'T**:
- Return massive result sets (use LIMIT)
- Perform complex calculations in application code
- Re-scan data unnecessarily

### 3. Data Loading

✅ **DO**:
- Use Parquet for best performance
- Set appropriate batch sizes (8192-16384)
- Consolidate batches after bulk inserts
- Validate schema compatibility

❌ **DON'T**:
- Load data row-by-row
- Mix incompatible schemas
- Ignore batch count warnings

### 4. Memory Management

✅ **DO**:
- Monitor cube size with `CubeStatistics`
- Consolidate batches periodically
- Use appropriate data types
- Consider data retention policies

❌ **DON'T**:
- Load entire database into memory
- Keep unnecessary historical data
- Ignore memory warnings

---

## Troubleshooting

### Common Issues

#### "Schema mismatch" Error

**Problem**: Data types don't match cube schema

**Solution**:
```rust
// Ensure data types match when loading
.add_dimension("date", DataType::Utf8)?  // Must match CSV data type
.add_measure("sales", DataType::Float64)?  // Must be numeric
```

#### "Query execution failed"

**Problem**: Invalid SQL or field references

**Solution**:
- Check field names (case-sensitive)
- Verify aggregation function syntax
- Ensure calculated fields are defined

#### Out of Memory

**Problem**: Dataset too large for available RAM

**Solution**:
- Increase system memory
- Filter data during load
- Use smaller batch sizes
- Consider data sampling

#### Slow Query Performance

**Problem**: Queries taking too long

**Solution**:
- Enable query caching
- Increase `target_partitions`
- Use more selective filters
- Consolidate batches
- Consider pre-calculated fields

### Getting Help

- **GitHub Issues**: https://github.com/yourorg/elasticube/issues
- **API Documentation**: `cargo doc --open`
- **Examples**: See `examples/` directory
- **Community**: [Link to community forum/Discord]

---

## Appendix A: Data Type Reference

| Rust Type | Arrow DataType | Use Case |
|-----------|---------------|----------|
| `i32` | `DataType::Int32` | Small integers |
| `i64` | `DataType::Int64` | Large integers |
| `f32` | `DataType::Float32` | Single-precision floats |
| `f64` | `DataType::Float64` | Double-precision floats (default for measures) |
| `String` | `DataType::Utf8` | Text/categorical data |
| `bool` | `DataType::Boolean` | True/false values |
| `Date` | `DataType::Date32` | Calendar dates |
| `DateTime` | `DataType::Timestamp` | Date + time |

## Appendix B: Feature Flags

| Feature | Description | Dependencies |
|---------|-------------|-------------|
| `database` | PostgreSQL, MySQL, ODBC connectors | arrow-odbc |
| `rest-api` | REST API data source | reqwest |
| `object-storage` | S3, GCS, Azure Blob Storage | object_store |
| `all-sources` | All data sources | All of the above |

Enable features in `Cargo.toml`:
```toml
elasticube-core = { version = "0.1.0", features = ["database", "rest-api"] }
```

## Appendix C: SQL Function Reference

### Aggregate Functions

- `SUM(column)` - Sum of values
- `AVG(column)` - Average value
- `MIN(column)` - Minimum value
- `MAX(column)` - Maximum value
- `COUNT(column)` - Count of non-null values
- `COUNT(DISTINCT column)` - Count of unique values

### String Functions

- `SUBSTRING(column, start, length)` - Extract substring
- `UPPER(column)` - Convert to uppercase
- `LOWER(column)` - Convert to lowercase
- `CONCAT(col1, col2)` - Concatenate strings

### Mathematical Functions

- `ABS(column)` - Absolute value
- `ROUND(column, decimals)` - Round to decimal places
- `FLOOR(column)` - Round down
- `CEIL(column)` - Round up

### Conditional

- `CASE WHEN condition THEN value ELSE other END` - Conditional logic
- `COALESCE(col1, col2, default)` - First non-null value

---

**End of User Guide**

For the latest updates, see: https://github.com/cachemcclure/elasticube
