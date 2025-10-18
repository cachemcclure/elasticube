#!/usr/bin/env python3
"""
Comprehensive Query Demo for ElastiCube Python Bindings

This example demonstrates all the query capabilities of ElastiCube including:
- Building cubes from CSV data
- Fluent API queries with PyArrow output
- Pandas DataFrame integration
- Aggregations and filtering
- Basic OLAP-style operations

Requirements:
    pip install pandas pyarrow
"""

import os
from elasticube import ElastiCubeBuilder


def main():
    print("=== ElastiCube Python Query Demo ===\n")

    # Get the path to the CSV file (relative to this script)
    script_dir = os.path.dirname(os.path.abspath(__file__))
    csv_path = os.path.join(script_dir, "sales_data.csv")

    # Step 1: Build the cube from CSV
    print("Step 1: Building ElastiCube from CSV data...")
    builder = ElastiCubeBuilder("sales_cube")

    # Define dimensions (categorical fields)
    builder.add_dimension("region", "utf8")
    builder.add_dimension("product", "utf8")
    builder.add_dimension("category", "utf8")
    builder.add_dimension("year", "int64")
    builder.add_dimension("quarter", "int64")

    # Define measures (numeric fields with aggregation functions)
    builder.add_measure("sales", "float64", "sum")
    builder.add_measure("quantity", "int64", "sum")

    # Load data from CSV
    builder.load_csv(csv_path)

    # Build the cube
    cube = builder.build()
    print(f"✓ Cube '{cube.name()}' created with {cube.row_count()} rows\n")

    # Step 2: Basic SELECT query with PyArrow Table output
    print("=== Example 1: Basic SELECT (PyArrow Table) ===")
    query = cube.query()
    query.select(["region", "product", "sales"])
    query.limit(5)
    arrow_table = query.execute()
    print(f"Arrow Table: {arrow_table}")
    print(f"Schema: {arrow_table.schema}")
    print()

    # Step 3: Query with Pandas DataFrame output
    print("=== Example 2: Aggregation by Region (Pandas DataFrame) ===")
    query = cube.query()
    query.select([
        "region",
        "SUM(sales) as total_sales",
        "SUM(quantity) as total_quantity"
    ])
    query.group_by(["region"])
    query.order_by(["total_sales DESC"])
    df = query.to_pandas()
    print(df)
    print()

    # Step 4: Filtering with WHERE
    print("=== Example 3: Filter High-Value Sales ===")
    query = cube.query()
    query.select(["region", "product", "sales", "quantity"])
    query.filter("sales > 1000")
    query.order_by(["sales DESC"])
    df = query.to_pandas()
    print(df)
    print()

    # Step 5: Complex aggregation
    print("=== Example 4: Multi-Dimensional Analysis ===")
    query = cube.query()
    query.select([
        "region",
        "category",
        "SUM(sales) as total_sales",
        "AVG(sales) as avg_sales",
        "COUNT(*) as transaction_count"
    ])
    query.group_by(["region", "category"])
    query.order_by(["total_sales DESC"])
    query.limit(10)
    df = query.to_pandas()
    print(df.to_string(index=False))
    print()

    # Step 6: OLAP-style Slice (filter on single dimension)
    print("=== Example 5: Slice - North Region Only ===")
    query = cube.query()
    query.filter("region = 'North'")
    query.select(["product", "SUM(sales) as total_sales"])
    query.group_by(["product"])
    df = query.to_pandas()
    print(df.to_string(index=False))
    print()

    # Step 7: OLAP-style Dice (filter on multiple dimensions)
    print("=== Example 6: Dice - North Region + Widget Product ===")
    query = cube.query()
    query.filter("region = 'North' AND product = 'Widget'")
    query.select([
        "quarter",
        "SUM(sales) as total_sales",
        "SUM(quantity) as total_qty"
    ])
    query.group_by(["quarter"])
    df = query.to_pandas()
    print(df.to_string(index=False))
    print()

    # Step 8: Category performance
    print("=== Example 7: Category Performance ===")
    query = cube.query()
    query.select([
        "category",
        "COUNT(*) as transactions",
        "SUM(sales) as total_sales",
        "AVG(quantity) as avg_quantity",
        "MAX(sales) as max_sale"
    ])
    query.group_by(["category"])
    query.order_by(["total_sales DESC"])
    df = query.to_pandas()
    print(df.to_string(index=False))
    print()

    # Step 9: Quarterly sales trend
    print("=== Example 8: Quarterly Sales Trend ===")
    query = cube.query()
    query.select([
        "quarter",
        "SUM(sales) as total_sales",
        "COUNT(*) as transactions",
        "AVG(sales) as avg_sale"
    ])
    query.group_by(["quarter"])
    query.order_by(["quarter ASC"])
    df = query.to_pandas()
    print(df.to_string(index=False))
    print()

    # Step 10: Product performance by category
    print("=== Example 9: Product Performance by Category ===")
    query = cube.query()
    query.select([
        "product",
        "category",
        "SUM(sales) as total_revenue",
        "SUM(quantity) as units_sold",
        "COUNT(*) as num_transactions"
    ])
    query.group_by(["product", "category"])
    query.order_by(["total_revenue DESC"])
    df = query.to_pandas()
    print(df.to_string(index=False))
    print()

    # Step 11: Top 5 sales transactions
    print("=== Example 10: Top 5 Sales Transactions ===")
    query = cube.query()
    query.select(["region", "product", "category", "sales", "quantity", "quarter"])
    query.order_by(["sales DESC"])
    query.limit(5)
    df = query.to_pandas()
    print(df.to_string(index=False))
    print()

    # Step 12: Using Pandas for additional analysis
    print("=== Example 11: Pandas Integration - Statistical Analysis ===")
    query = cube.query()
    query.select(["region", "sales", "quantity"])
    df = query.to_pandas()

    print("Sales statistics by region:")
    stats = df.groupby('region')['sales'].describe()
    print(stats)
    print()

    print("\n=== Demo Complete! ===")
    print("\nKey Takeaways:")
    print("✓ ElastiCube Python bindings support fluent query API")
    print("✓ Seamless integration with PyArrow Tables")
    print("✓ Direct conversion to Pandas DataFrames via to_pandas()")
    print("✓ Aggregations (SUM, AVG, COUNT, MAX, MIN) work seamlessly")
    print("✓ OLAP-style filtering (slice/dice patterns)")
    print("✓ Powered by Apache DataFusion for high performance")


if __name__ == "__main__":
    main()
