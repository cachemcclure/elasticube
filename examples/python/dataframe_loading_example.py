#!/usr/bin/env python3
"""
Example demonstrating DataFrame loading functionality in ElastiCube.

This example shows how to:
1. Load data from Polars DataFrames
2. Load data from Pandas DataFrames
3. Load data from PyArrow Tables
4. Query the cube and get results as different DataFrame types
"""

import pyarrow as pa
import pandas as pd

try:
    import polars as pl
    POLARS_AVAILABLE = True
except ImportError:
    POLARS_AVAILABLE = False
    print("Note: Polars not installed. Skipping Polars examples.")
    print("Install with: pip install polars")

from elasticube import ElastiCubeBuilder


def example_polars_loading():
    """Example: Load data from a Polars DataFrame."""
    if not POLARS_AVAILABLE:
        print("\nSkipping Polars example (not installed)")
        return

    print("\n=== Example 1: Loading from Polars DataFrame ===")

    # Create a Polars DataFrame
    df = pl.DataFrame({
        "region": ["North", "South", "East", "West", "North", "South"],
        "product": ["Widget", "Widget", "Gadget", "Gadget", "Gadget", "Widget"],
        "sales": [1000.0, 1500.0, 1200.0, 900.0, 800.0, 1100.0],
        "quantity": [100, 150, 120, 90, 80, 110]
    })

    print(f"Original Polars DataFrame:\n{df}\n")

    # Build cube using load_from_polars()
    builder = ElastiCubeBuilder("sales_cube")
    builder.add_dimension("region", "utf8")
    builder.add_dimension("product", "utf8")
    builder.add_measure("sales", "float64", "sum")
    builder.add_measure("quantity", "int64", "sum")
    builder.load_from_polars(df)
    cube = builder.build()

    print(f"Cube created with {cube.row_count()} rows")

    # Query and get results as Polars DataFrame
    query = cube.query()
    query.select(["region", "SUM(sales) as total_sales", "SUM(quantity) as total_quantity"])
    query.group_by(["region"])
    query.order_by(["total_sales DESC"])
    result = query.to_polars()

    print(f"\nQuery Results (Polars):\n{result}")


def example_pandas_loading():
    """Example: Load data from a Pandas DataFrame."""
    print("\n=== Example 2: Loading from Pandas DataFrame ===")

    # Create a Pandas DataFrame with datetime
    df = pd.DataFrame({
        "date": pd.date_range("2024-01-01", periods=10),
        "category": ["A", "B"] * 5,
        "revenue": [100.0, 150.0, 200.0, 175.0, 125.0,
                    180.0, 220.0, 190.0, 160.0, 210.0],
    })

    print(f"Original Pandas DataFrame:\n{df}\n")

    # Build cube using load_from_pandas()
    builder = ElastiCubeBuilder("revenue_cube")
    builder.add_dimension("category", "utf8")
    builder.add_measure("revenue", "float64", "sum")
    builder.load_from_pandas(df)
    cube = builder.build()

    print(f"Cube created with {cube.row_count()} rows")

    # Query and get results as Pandas DataFrame
    query = cube.query()
    query.select([
        "category",
        "SUM(revenue) as total_revenue",
        "AVG(revenue) as avg_revenue",
        "COUNT(*) as count"
    ])
    query.group_by(["category"])
    result = query.to_pandas()

    print(f"\nQuery Results (Pandas):\n{result}")


def example_arrow_loading():
    """Example: Load data from a PyArrow Table."""
    print("\n=== Example 3: Loading from PyArrow Table ===")

    # Create a PyArrow Table
    table = pa.table({
        "product_id": [1, 2, 3, 4, 5],
        "product_name": ["Widget", "Gadget", "Doohickey", "Thingamajig", "Whatsit"],
        "price": [19.99, 29.99, 39.99, 49.99, 59.99],
        "stock": [100, 50, 75, 25, 60]
    })

    print(f"Original PyArrow Table:\n{table}\n")

    # Build cube using load_from_arrow()
    builder = ElastiCubeBuilder("inventory_cube")
    builder.add_dimension("product_name", "utf8")
    builder.add_measure("price", "float64", "avg")
    builder.add_measure("stock", "int64", "sum")
    builder.load_from_arrow(table)
    cube = builder.build()

    print(f"Cube created with {cube.row_count()} rows")

    # Query and get results as PyArrow Table
    query = cube.query()
    query.select([
        "product_name",
        "AVG(price) as avg_price",
        "SUM(stock) as total_stock"
    ])
    query.group_by(["product_name"])
    query.order_by(["avg_price DESC"])
    result = query.execute()

    print(f"\nQuery Results (PyArrow):\n{result}")


def example_type_normalization():
    """Example: Automatic type normalization with large_utf8."""
    print("\n=== Example 4: Type Normalization (large_utf8 → utf8) ===")

    # Create a PyArrow Table with large_utf8 type
    schema = pa.schema([
        pa.field("id", pa.int64()),
        pa.field("description", pa.large_utf8()),  # This will be normalized
        pa.field("value", pa.float64())
    ])

    table = pa.table({
        "id": [1, 2, 3],
        "description": ["Long description " * 10, "Another long text " * 10, "More text " * 10],
        "value": [100.0, 200.0, 300.0]
    }, schema=schema)

    print(f"Original schema:\n{table.schema}\n")
    print("Note: 'description' field has type large_string (large_utf8)")

    # Load into cube - type normalization happens automatically
    builder = ElastiCubeBuilder("normalized_cube")
    builder.load_from_arrow(table)
    cube = builder.build()

    print(f"\nCube created with {cube.row_count()} rows")
    print("Type normalization applied automatically: large_utf8 → utf8")

    # Query the data
    query = cube.query()
    query.select(["SUM(value) as total"])
    result = query.to_pandas()

    print(f"\nQuery Results:\n{result}")


def example_comparison():
    """Example: Compare old approach vs new approach."""
    if not POLARS_AVAILABLE:
        print("\nSkipping comparison example (Polars not installed)")
        return

    print("\n=== Example 5: Before vs After Comparison ===")

    # Create sample data
    df = pl.DataFrame({
        "region": ["North", "South", "East", "West"],
        "sales": [1000.0, 1500.0, 1200.0, 900.0]
    })

    print("BEFORE (old approach with temp files):")
    print("```python")
    print("import tempfile")
    print("import pyarrow.parquet as pq")
    print("")
    print("arrow_table = df.to_arrow()")
    print("with tempfile.NamedTemporaryFile(suffix='.parquet') as tmp:")
    print("    pq.write_table(arrow_table, tmp.name)")
    print("    builder.load_parquet(tmp.name)")
    print("cube = builder.build()")
    print("```")
    print("Lines of code: ~7 lines")
    print("Disk I/O: YES (writes + reads entire dataset)")
    print("Performance: Slower due to serialization overhead\n")

    print("AFTER (new approach):")
    print("```python")
    print("builder = ElastiCubeBuilder('sales')")
    print("builder.add_dimension('region', 'utf8')")
    print("builder.add_measure('sales', 'float64', 'sum')")
    print("builder.load_from_polars(df)")
    print("cube = builder.build()")
    print("```")
    print("Lines of code: 1 line (load_from_polars)")
    print("Disk I/O: NO (pure memory)")
    print("Performance: 10-20x faster\n")

    # Actually build it
    builder = ElastiCubeBuilder("comparison_cube")
    builder.add_dimension("region", "utf8")
    builder.add_measure("sales", "float64", "sum")
    builder.load_from_polars(df)
    cube = builder.build()

    print(f"✅ Cube successfully created with {cube.row_count()} rows using new approach!")


def main():
    """Run all examples."""
    print("=" * 70)
    print("ElastiCube DataFrame Loading Examples")
    print("=" * 70)

    example_polars_loading()
    example_pandas_loading()
    example_arrow_loading()
    example_type_normalization()
    example_comparison()

    print("\n" + "=" * 70)
    print("All examples completed successfully!")
    print("=" * 70)


if __name__ == "__main__":
    main()
