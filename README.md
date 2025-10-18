# ElastiCube Library

A high-performance, embeddable OLAP cube builder and query library written in Rust with optional Python bindings.

## Overview

ElastiCube Library provides fast, in-memory multidimensional analytical processing (OLAP) without requiring pre-aggregation or external services. Built on Apache Arrow and DataFusion, it offers columnar storage and efficient query execution for analytical workloads.

## Features

- **Columnar Storage**: Efficient field-by-field storage using Apache Arrow
- **No Pre-Aggregation**: Query raw data with dynamic aggregations
- **Multi-Source**: Combine data from CSV, Parquet, JSON, and custom sources
- **Embeddable**: Pure Rust library with no cloud dependencies
- **Fast**: Near C-level performance with parallel query execution
- **Flexible**: Use from Rust or Python (via PyO3 bindings)

## Project Status

Currently in **Phase 1** of development. See [PROJECT_CHECKLIST.md](PROJECT_CHECKLIST.md) for detailed implementation roadmap.

## Quick Start (Planned API)

### Rust

```rust
use elasticube_core::{ElastiCubeBuilder, AggFunc};
use arrow::datatypes::DataType;

#[tokio::main]
async fn main() -> Result<()> {
    let cube = ElastiCubeBuilder::new("sales_cube")
        .add_dimension("region", DataType::Utf8)?
        .add_measure("sales", DataType::Float64, AggFunc::Sum)?
        .load_csv("data.csv")?
        .build()?;

    let results = cube.query()
        .select(&["region", "sum(sales)"])
        .group_by(&["region"])
        .execute()
        .await?;

    Ok(())
}
```

### Python (Coming Soon)

```python
from elasticube import ElastiCube

cube = ElastiCube.builder() \
    .add_measure("revenue", agg="sum") \
    .load_csv("data.csv") \
    .build()

results = cube.query() \
    .select(["category", "sum(revenue)"]) \
    .group_by(["category"]) \
    .to_pandas()
```

## Documentation

- [CLAUDE.md](CLAUDE.md) - Project overview and architecture
- [PROJECT_CHECKLIST.md](PROJECT_CHECKLIST.md) - Implementation roadmap

## Development

```bash
# Build the project
cargo build

# Run tests
cargo test

# Check code
cargo check
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contributing

See [PROJECT_CHECKLIST.md](PROJECT_CHECKLIST.md) for current development status and planned features.
