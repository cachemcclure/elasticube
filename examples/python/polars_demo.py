#!/usr/bin/env python3
"""
Polars Integration Demo for ElastiCube

This example demonstrates the high-performance Polars integration.
Polars is significantly faster than Pandas for large datasets and analytical workloads.

Key Benefits of Polars:
- Zero-copy conversion from PyArrow (native Arrow backend)
- Lazy evaluation and query optimization
- Parallel execution by default
- Lower memory footprint
- Expressive API similar to Pandas

Requirements:
    pip install polars pyarrow
"""

import os
import time
from elasticube import ElastiCubeBuilder


def main():
    print("=== ElastiCube + Polars Performance Demo ===\n")

    # Get the path to the CSV file
    script_dir = os.path.dirname(os.path.abspath(__file__))
    csv_path = os.path.join(script_dir, "sales_data.csv")

    # Build the cube
    print("Building ElastiCube from CSV data...")
    builder = ElastiCubeBuilder("sales_cube")
    builder.add_dimension("region", "utf8")
    builder.add_dimension("product", "utf8")
    builder.add_dimension("category", "utf8")
    builder.add_dimension("year", "int64")
    builder.add_dimension("quarter", "int64")
    builder.add_measure("sales", "float64", "sum")
    builder.add_measure("quantity", "int64", "sum")
    builder.load_csv(csv_path)
    cube = builder.build()
    print(f"✓ Cube '{cube.name()}' created with {cube.row_count()} rows\n")

    # Example 1: Basic query with Polars
    print("=== Example 1: Basic Query with Polars ===")
    query = cube.query()
    query.select(["region", "product", "sales", "quantity"])
    query.limit(5)

    start = time.perf_counter()
    df_polars = query.to_polars()
    elapsed = time.perf_counter() - start
    print(f"Query executed in {elapsed*1000:.2f}ms")
    print(df_polars)
    print()

    # Example 2: Aggregation with Polars
    print("=== Example 2: Regional Aggregation (Polars) ===")
    query = cube.query()
    query.select([
        "region",
        "SUM(sales) as total_sales",
        "SUM(quantity) as total_quantity",
        "COUNT(*) as transactions"
    ])
    query.group_by(["region"])
    query.order_by(["total_sales DESC"])
    df = query.to_polars()
    print(df)
    print()

    # Example 3: Polars-specific operations (lazy evaluation)
    print("=== Example 3: Polars Lazy Operations ===")
    query = cube.query()
    query.select(["region", "product", "category", "sales", "quantity"])
    df = query.to_polars()

    # Use Polars lazy API for additional transformations
    result = (
        df.lazy()
        .filter(pl.col("sales") > 900)
        .group_by("category")
        .agg([
            pl.col("sales").sum().alias("total_sales"),
            pl.col("quantity").sum().alias("total_quantity"),
            pl.col("sales").mean().alias("avg_sales"),
            pl.len().alias("count")
        ])
        .sort("total_sales", descending=True)
        .collect()
    )
    print(result)
    print()

    # Example 4: Complex multi-dimensional analysis
    print("=== Example 4: Multi-Dimensional Analysis ===")
    query = cube.query()
    query.select([
        "region",
        "category",
        "quarter",
        "SUM(sales) as total_sales",
        "AVG(sales) as avg_sales",
        "COUNT(*) as count"
    ])
    query.group_by(["region", "category", "quarter"])
    query.order_by(["total_sales DESC"])
    query.limit(10)
    df = query.to_polars()
    print(df)
    print()

    # Example 5: Polars statistical operations
    print("=== Example 5: Statistical Analysis with Polars ===")
    query = cube.query()
    query.select(["region", "sales", "quantity"])
    df = query.to_polars()

    # Polars groupby operations (faster than Pandas)
    stats = df.group_by("region").agg([
        pl.col("sales").sum().alias("total_sales"),
        pl.col("sales").mean().alias("avg_sales"),
        pl.col("sales").std().alias("std_sales"),
        pl.col("sales").min().alias("min_sales"),
        pl.col("sales").max().alias("max_sales"),
        pl.col("quantity").sum().alias("total_qty")
    ]).sort("total_sales", descending=True)
    print(stats)
    print()

    # Example 6: Time-based aggregation
    print("=== Example 6: Quarterly Trend Analysis ===")
    query = cube.query()
    query.select([
        "quarter",
        "SUM(sales) as total_sales",
        "AVG(sales) as avg_sales",
        "COUNT(*) as transactions"
    ])
    query.group_by(["quarter"])
    query.order_by(["quarter ASC"])
    df = query.to_polars()
    print(df)
    print()

    # Example 7: Product performance ranking
    print("=== Example 7: Product Performance Ranking ===")
    query = cube.query()
    query.select([
        "product",
        "category",
        "SUM(sales) as revenue",
        "SUM(quantity) as units",
        "COUNT(*) as transactions"
    ])
    query.group_by(["product", "category"])
    query.order_by(["revenue DESC"])
    df = query.to_polars()

    # Add computed columns with Polars
    result = df.with_columns([
        (pl.col("revenue") / pl.col("units")).alias("avg_price"),
        (pl.col("revenue") / pl.col("transactions")).alias("avg_transaction_value")
    ])
    print(result)
    print()

    # Example 8: Comparison - PyArrow vs Polars vs Pandas
    print("=== Example 8: Performance Comparison ===")
    query = cube.query()
    query.select(["region", "product", "category", "sales", "quantity"])

    # PyArrow Table (zero-copy, fastest)
    start = time.perf_counter()
    arrow_table = query.execute()
    arrow_time = time.perf_counter() - start

    # Polars DataFrame (zero-copy from Arrow, very fast)
    query = cube.query()
    query.select(["region", "product", "category", "sales", "quantity"])
    start = time.perf_counter()
    polars_df = query.to_polars()
    polars_time = time.perf_counter() - start

    # Pandas DataFrame (requires conversion, slower)
    query = cube.query()
    query.select(["region", "product", "category", "sales", "quantity"])
    start = time.perf_counter()
    pandas_df = query.to_pandas()
    pandas_time = time.perf_counter() - start

    print(f"PyArrow Table: {arrow_time*1000:.3f}ms (baseline)")
    print(f"Polars DataFrame: {polars_time*1000:.3f}ms ({polars_time/arrow_time:.2f}x)")
    print(f"Pandas DataFrame: {pandas_time*1000:.3f}ms ({pandas_time/arrow_time:.2f}x)")
    print()

    print("\n=== Performance Summary ===")
    print("✓ Polars provides near-zero-copy conversion from Arrow")
    print("✓ Polars lazy evaluation enables additional optimizations")
    print("✓ Polars parallel execution scales with CPU cores")
    print("✓ Use Polars for maximum performance on large datasets")
    print("✓ Use Pandas for compatibility with existing workflows")


if __name__ == "__main__":
    # Import polars here to show clear error if not installed
    try:
        import polars as pl
    except ImportError:
        print("Error: Polars is not installed")
        print("Install with: pip install polars")
        exit(1)

    main()
