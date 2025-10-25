# ElastiCube Library

A high-performance, embeddable OLAP cube builder and query library written in Rust with Python bindings.

## Overview

ElastiCube Library provides fast, in-memory multidimensional analytical processing (OLAP) without requiring pre-aggregation or external services. Built on Apache Arrow and DataFusion, it offers columnar storage and efficient query execution for analytical workloads.

## Features

- **DataFrame Loading**: Load from Polars, Pandas, or PyArrow in one line - 10-20x faster than temp files
- **Columnar Storage**: Efficient field-by-field storage using Apache Arrow
- **Dynamic Aggregations**: Query raw data without pre-aggregation
- **Multi-Source Data**: Load from CSV, Parquet, JSON, DataFrames, and RecordBatch sources
- **Data Updates**: Append, delete, and update rows incrementally
- **Calculated Fields**: Define virtual dimensions and calculated measures using SQL expressions
- **Query Optimization**: Built-in caching and performance optimizations
- **OLAP Operations**: Slice, dice, drill-down, and roll-up operations
- **Python Integration**: Full Python API with native Polars, Pandas, and PyArrow support
- **Embeddable**: Pure Rust library with no cloud dependencies
- **Fast**: Near C-level performance with parallel query execution via DataFusion

## Architecture

```
elasticube_library/
├── elasticube-core/          # Rust core library
│   ├── src/
│   │   ├── lib.rs           # Public API exports
│   │   ├── builder.rs       # ElastiCubeBuilder for cube construction
│   │   ├── cube/            # Core cube implementation
│   │   │   ├── mod.rs       # ElastiCube, Dimension, Measure, etc.
│   │   │   ├── schema.rs    # Schema management
│   │   │   ├── hierarchy.rs # Hierarchical dimensions
│   │   │   ├── calculated.rs # Calculated measures & virtual dimensions
│   │   │   └── updates.rs   # Data update operations
│   │   ├── query.rs         # QueryBuilder and execution
│   │   ├── cache.rs         # Query result caching
│   │   ├── optimization.rs  # Performance optimizations
│   │   ├── storage.rs       # Data storage layer
│   │   └── sources.rs       # CSV, Parquet, JSON data sources
│   └── Cargo.toml
│
├── elasticube-py/           # Python bindings (PyO3)
│   ├── src/lib.rs           # Python API wrapper
│   └── Cargo.toml
│
└── examples/                # Usage examples
    ├── query_demo.rs        # Comprehensive query examples
    ├── calculated_fields_demo.rs # Calculated fields demo
    ├── data_updates_demo.rs # Data update operations
    └── python/              # Python examples
        ├── query_demo.py
        ├── polars_demo.py
        └── visualization_demo.py
```

## Quick Start

### Rust

Add to your `Cargo.toml`:
```toml
[dependencies]
elasticube-core = { path = "elasticube-core" }
tokio = { version = "1", features = ["full"] }
arrow = "54"
```

Build and query a cube:

```rust
use elasticube_core::{ElastiCubeBuilder, AggFunc};
use arrow::datatypes::DataType;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build cube from CSV
    let cube = ElastiCubeBuilder::new("sales_cube")
        .add_dimension("region", DataType::Utf8)?
        .add_dimension("product", DataType::Utf8)?
        .add_measure("sales", DataType::Float64, AggFunc::Sum)?
        .add_measure("quantity", DataType::Int32, AggFunc::Sum)?
        .load_csv("data.csv")?
        .build()?;

    // Query with aggregation
    let result = Arc::new(cube)
        .query()?
        .select(&["region", "SUM(sales) as total_sales"])
        .group_by(&["region"])
        .order_by(&["total_sales DESC"])
        .execute()
        .await?;

    result.pretty_print()?;
    Ok(())
}
```

**Key Types & Functions:**
- `ElastiCubeBuilder` - Builder for constructing cubes (elasticube-core/src/builder.rs:19)
- `ElastiCube::query()` - Create query builder (elasticube-core/src/cube/mod.rs:133)
- `QueryBuilder::execute()` - Execute query (elasticube-core/src/query.rs:225)

### Python

Install the Python package:
```bash
cd elasticube-py
pip install maturin
maturin develop
```

Use in Python:

```python
from elasticube import ElastiCube
import pyarrow as pa

# Build cube from CSV
cube = ElastiCube.builder("sales") \
    .add_dimension("region", "string") \
    .add_dimension("product", "string") \
    .add_measure("sales", "float64", "sum") \
    .add_measure("quantity", "int32", "sum") \
    .load_csv("data.csv") \
    .build()

# Query and convert to Pandas
df = cube.query() \
    .select(["region", "SUM(sales) as total_sales"]) \
    .group_by(["region"]) \
    .order_by(["total_sales DESC"]) \
    .to_pandas()

print(df)
```

**Python Bindings:**
- `PyElastiCubeBuilder` - Build cubes from Python (elasticube-py/src/lib.rs:16)
- `PyElastiCube` - Python cube wrapper (elasticube-py/src/lib.rs:116)
- `PyQueryBuilder` - Query builder (elasticube-py/src/lib.rs:332)

## DataFrame Loading

ElastiCube provides **first-class support for loading data directly from DataFrames** - no temp files, no manual conversion, just one line of code.

### Streaming Support for Large Datasets

For very large datasets (>10M rows), ElastiCube supports **chunked/streaming loading** to manage memory efficiently:

```python
from elasticube.streaming import load_polars_chunked
import polars as pl

# Load 10M row DataFrame in 1M row chunks
large_df = pl.DataFrame({"id": range(10_000_000), "value": range(10_000_000)})

builder = ElastiCubeBuilder("large_cube")
builder.add_dimension("id", "int64")
builder.add_measure("value", "int64", "sum")

cube = load_polars_chunked(
    builder,
    large_df,
    chunk_size=1_000_000,
    progress_callback=lambda chunk, total, rows: print(f"Chunk {chunk}/{total}: {rows:,} rows")
)
```

**Features:**
- Reduced memory usage (process data in chunks)
- Progress tracking for long-running operations
- Handles datasets larger than available RAM
- Automatic chunk size estimation via `estimate_chunk_size()`
- Stream from Parquet files with `stream_from_parquet()`

See `examples/python/streaming_dataframes_example.py` for comprehensive examples.

### Installation

```bash
# Install with Polars support (recommended for analytics)
pip install elasticube polars pyarrow

# Or with just Pandas
pip install elasticube pyarrow
```

### Before vs After

**Old approach (10+ lines with temp files):**
```python
import polars as pl
import tempfile
import pyarrow.parquet as pq

df = pl.read_csv("data.csv")

# Cumbersome workaround
arrow_table = df.to_arrow()
with tempfile.NamedTemporaryFile(suffix=".parquet", delete=False) as tmp:
    pq.write_table(arrow_table, tmp.name)
    builder.load_parquet(tmp.name)

cube = builder.build()
os.unlink(tmp.name)  # Manual cleanup
```

**New approach (1 line - 10-20x faster):**
```python
import polars as pl

df = pl.read_csv("data.csv")

cube = ElastiCube.builder("sales") \
    .load_from_polars(df) \
    .build()
```

### Loading from Polars DataFrames

```python
import polars as pl
from elasticube import ElastiCubeBuilder

# Create Polars DataFrame
df = pl.DataFrame({
    "region": ["North", "South", "East", "West"],
    "product": ["Widget", "Gadget", "Widget", "Gadget"],
    "sales": [1000.0, 1500.0, 1200.0, 900.0],
    "quantity": [100, 150, 120, 90]
})

# Load directly into ElastiCube (zero-copy, no disk I/O)
cube = ElastiCubeBuilder("sales") \
    .add_dimension("region", "utf8") \
    .add_dimension("product", "utf8") \
    .add_measure("sales", "float64", "sum") \
    .add_measure("quantity", "int64", "sum") \
    .load_from_polars(df) \
    .build()

# Query and get results back as Polars
result = cube.query() \
    .select(["region", "SUM(sales) as total"]) \
    .group_by(["region"]) \
    .to_polars()  # Zero-copy conversion
```

### Loading from Pandas DataFrames

```python
import pandas as pd
from elasticube import ElastiCubeBuilder

# Create Pandas DataFrame
df = pd.DataFrame({
    "date": pd.date_range("2024-01-01", periods=100),
    "category": ["A", "B"] * 50,
    "revenue": range(100)
})

# Load directly into ElastiCube
cube = ElastiCubeBuilder("revenue") \
    .add_dimension("category", "utf8") \
    .add_measure("revenue", "float64", "sum") \
    .load_from_pandas(df) \
    .build()

# Query and convert to Pandas
result = cube.query() \
    .select(["category", "SUM(revenue) as total"]) \
    .group_by(["category"]) \
    .to_pandas()
```

### Loading from PyArrow Tables

```python
import pyarrow as pa
from elasticube import ElastiCubeBuilder

# Create PyArrow Table
table = pa.table({
    "product": ["Widget", "Gadget", "Doohickey"],
    "price": [19.99, 29.99, 39.99],
    "stock": [100, 50, 75]
})

# Load directly into ElastiCube (zero-copy when schema compatible)
cube = ElastiCubeBuilder("inventory") \
    .add_dimension("product", "utf8") \
    .add_measure("price", "float64", "avg") \
    .add_measure("stock", "int64", "sum") \
    .load_from_arrow(table) \
    .build()
```

### Key Benefits

- **10-20x faster**: No disk I/O, pure memory operations
- **Zero-copy**: Efficient data transfer via Apache Arrow
- **Automatic type normalization**: Handles `large_utf8`, timezone-aware timestamps, etc.
- **One line of code**: `load_from_polars(df)`, `load_from_pandas(df)`, or `load_from_arrow(table)`
- **No temp files**: No cleanup, no disk space usage

See `examples/python/dataframe_loading_example.py` for comprehensive examples.

## Advanced Features

### Calculated Fields

Define derived metrics and dimensions using SQL expressions:

```rust
let cube = ElastiCubeBuilder::new("sales")
    .add_measure("revenue", DataType::Float64, AggFunc::Sum)?
    .add_measure("cost", DataType::Float64, AggFunc::Sum)?
    // Calculated measure
    .add_calculated_measure(
        "profit",
        "revenue - cost",
        DataType::Float64,
        AggFunc::Sum
    )?
    // Virtual dimension
    .add_virtual_dimension(
        "year",
        "EXTRACT(YEAR FROM sale_date)",
        DataType::Int32
    )?
    .build()?;
```

See `CalculatedMeasure` and `VirtualDimension` in elasticube-core/src/cube/calculated.rs

### Data Updates

Incrementally update cube data without rebuilding:

```rust
// Append new rows
let new_batch = create_record_batch()?;
cube.append_rows(new_batch)?;

// Delete rows matching filter
cube.delete_rows("sales < 100").await?;

// Update specific rows
cube.update_rows("region = 'North'", updated_batch).await?;

// Consolidate fragmented batches
cube.consolidate_batches()?;
```

See update methods in elasticube-core/src/cube/mod.rs:279-373

### OLAP Operations

```rust
// Slice: filter on one dimension
let result = cube.query()?
    .slice("region", "North")
    .select(&["product", "SUM(sales)"])
    .execute()
    .await?;

// Dice: filter on multiple dimensions
let result = cube.query()?
    .dice(&[("region", "North"), ("product", "Widget")])
    .select(&["quarter", "SUM(sales)"])
    .execute()
    .await?;
```

See elasticube-core/src/query.rs:75-103

### Hierarchies

Define drill-down paths for dimensional analysis:

```rust
let cube = ElastiCubeBuilder::new("sales")
    .add_dimension("year", DataType::Int32)?
    .add_dimension("quarter", DataType::Int32)?
    .add_dimension("month", DataType::Int32)?
    .add_hierarchy("time", vec!["year", "quarter", "month"])?
    .build()?;
```

See elasticube-core/src/cube/hierarchy.rs

### Performance Optimization

```rust
// Enable query caching
let cube = ElastiCubeBuilder::new("sales")
    .with_cache_size(100)?
    .build()?;

// Get statistics
let stats = cube.statistics();
println!("{}", stats.summary());

// Get cache stats
let cache_stats = cube.cache_stats();
println!("Cache hit rate: {:.2}%", cache_stats.hit_rate());
```

See elasticube-core/src/cache.rs and elasticube-core/src/optimization.rs

## Python Integration

### Polars (High Performance)

```python
# Zero-copy conversion to Polars DataFrame
df = cube.query() \
    .select(["region", "SUM(sales)"]) \
    .to_polars()  # 642x faster than to_pandas()

# Leverage Polars for further analysis
result = df.group_by("region").agg(pl.col("sales").sum())
```

### Pandas

```python
# Convert to Pandas DataFrame
df = cube.query() \
    .select(["region", "product", "SUM(sales)"]) \
    .to_pandas()

# Use familiar Pandas API
summary = df.groupby("region")["sales"].describe()
```

### Visualization

```python
import matplotlib.pyplot as plt

df = cube.query() \
    .select(["region", "SUM(sales) as total"]) \
    .group_by(["region"]) \
    .to_pandas()

df.plot(x="region", y="total", kind="bar")
plt.show()
```

See examples/python/ for complete examples.

## Examples

### Rust Examples

Run with `cargo run --example <name>`:

- `query_demo` - Comprehensive query examples with all features
- `calculated_fields_demo` - Virtual dimensions and calculated measures
- `data_updates_demo` - Append, delete, update operations

### Python Examples

Located in `examples/python/`:

- `query_demo.py` - Basic queries and aggregations
- `polars_demo.py` - High-performance Polars integration
- `visualization_demo.py` - Chart creation with Matplotlib
- `serialization_demo.py` - Save and load cubes
- `elasticube_tutorial.ipynb` - Interactive Jupyter notebook

## Development

### Build and Test

```bash
# Build Rust library
cargo build --release

# Run all tests (84 tests)
cargo test

# Run specific test module
cargo test --lib cube::updates

# Build Python bindings
cd elasticube-py
maturin develop
```

### Project Structure

- **Core Types**: ElastiCube, Dimension, Measure, Hierarchy (elasticube-core/src/cube/)
- **Builder Pattern**: ElastiCubeBuilder (elasticube-core/src/builder.rs)
- **Query Engine**: QueryBuilder, QueryResult (elasticube-core/src/query.rs)
- **Data Sources**: CsvSource, ParquetSource, JsonSource (elasticube-core/src/sources.rs)
- **Caching**: QueryCache, CacheStats (elasticube-core/src/cache.rs)
- **Optimization**: CubeStatistics, OptimizationConfig (elasticube-core/src/optimization.rs)

## Performance

- **Apache Arrow**: Columnar memory format for efficient data access
- **DataFusion**: SQL query optimizer and execution engine
- **Parallel Execution**: Multi-threaded query processing
- **Query Caching**: Automatic result caching for repeated queries
- **Zero-Copy**: Efficient data transfer between Rust and Python via PyArrow

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.
