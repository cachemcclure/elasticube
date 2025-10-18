# ElastiCube Python Bindings

High-performance OLAP cube library for Python, powered by Rust, Apache Arrow, and DataFusion.

## Features

- **Fast**: Native Rust implementation with near C-level performance
- **Columnar Storage**: Efficient memory layout using Apache Arrow
- **Familiar API**: Pandas-like interface for data scientists
- **Type Safe**: Full type hints and stub files for IDE support
- **No Pre-Aggregation**: Query raw data with dynamic aggregations

## Installation

```bash
pip install elasticube
```

## Quick Start

```python
from elasticube import ElastiCubeBuilder

# Build a cube from CSV data
builder = ElastiCubeBuilder()
builder.add_dimension("region", "string")
builder.add_dimension("date", "date")
builder.add_measure("sales", "float64", "sum")
builder.add_measure("quantity", "int32", "sum")
builder.load_csv("sales_data.csv")

cube = builder.build()

# Query the cube
query = cube.query()
query.select(["region", "SUM(sales)", "SUM(quantity)"])
query.filter("date >= '2024-01-01'")
query.group_by(["region"])
query.order_by(["region"])

# Get results as Pandas DataFrame
df = query.to_pandas()
print(df)
```

## Supported Data Types

- **Numeric**: `int32`, `int64`, `float32`, `float64`
- **Text**: `string` (utf8)
- **Temporal**: `date`, `timestamp`
- **Boolean**: `bool`

## Aggregation Functions

- `sum` - Sum of values
- `avg` / `mean` - Average
- `min` - Minimum value
- `max` - Maximum value
- `count` - Count of non-null values
- `count_distinct` - Count of unique values
- `median` - Median value
- `stddev` - Standard deviation
- `variance` - Variance

## Data Sources

ElastiCube supports multiple data source formats:

- **CSV**: `builder.load_csv("data.csv")`
- **Parquet**: `builder.load_parquet("data.parquet")`
- **JSON**: `builder.load_json("data.json")`

## License

Dual licensed under MIT or Apache-2.0.
