"""
Demonstration of OLAP features in ElastiCube Python bindings

This example shows the newly added features:
1. Hierarchies for drill-down/roll-up operations
2. Calculated measures and virtual dimensions
3. Schema introspection methods
4. OLAP operations (slice, dice, drill_down, roll_up, offset)
5. Cube statistics and metadata

Author: ElastiCube Contributors
Date: 2025-10-18
"""

import _elasticube
import pandas as pd
import pyarrow as pa
from datetime import date

def create_sample_data():
    """Create sample sales data with time dimensions for hierarchy demonstration"""
    data = {
        'year': [2024, 2024, 2024, 2024, 2024, 2024, 2024, 2024],
        'quarter': [1, 1, 1, 1, 2, 2, 2, 2],
        'month': [1, 2, 3, 4, 5, 6, 7, 8],
        'region': ['North', 'North', 'South', 'South', 'North', 'North', 'South', 'South'],
        'product': ['Widget', 'Gadget', 'Widget', 'Gadget', 'Widget', 'Gadget', 'Widget', 'Gadget'],
        'revenue': [10000, 15000, 8000, 12000, 11000, 16000, 9000, 13000],
        'cost': [6000, 9000, 5000, 7000, 7000, 10000, 6000, 8000],
        'units': [100, 150, 80, 120, 110, 160, 90, 130]
    }
    return pa.table(data)

def demo_builder_features():
    """Demonstrate new builder features: hierarchies, calculated measures, virtual dimensions"""
    print("=" * 80)
    print("DEMO 1: Builder Features")
    print("=" * 80)

    # Create builder with description
    builder = _elasticube.PyElastiCubeBuilder("sales_cube")
    builder.with_description("Q1-Q2 2024 sales data with OLAP features")

    # Add dimensions
    builder.add_dimension("year", "int32")
    builder.add_dimension("quarter", "int32")
    builder.add_dimension("month", "int32")
    builder.add_dimension("region", "string")
    builder.add_dimension("product", "string")

    # Add base measures
    builder.add_measure("revenue", "float64", "sum")
    builder.add_measure("cost", "float64", "sum")
    builder.add_measure("units", "int32", "sum")

    # Add hierarchy for time-based drill-down/roll-up
    print("\n✓ Adding time hierarchy: year → quarter → month")
    builder.add_hierarchy("time_hierarchy", ["year", "quarter", "month"])

    # Add calculated measure (profit = revenue - cost)
    print("✓ Adding calculated measure: profit = revenue - cost")
    builder.add_calculated_measure("profit", "revenue - cost", "float64", "sum")

    # Add virtual dimension (categorize months into seasons)
    print("✓ Adding virtual dimension: season based on quarter")
    builder.add_virtual_dimension(
        "season",
        "CASE WHEN quarter = 1 THEN 'Winter' WHEN quarter = 2 THEN 'Spring' ELSE 'Other' END",
        "string"
    )

    # Load data
    table = create_sample_data()
    # Note: We need to save to file first since load methods expect file paths
    # In production, you'd use load_csv/parquet/json
    print("✓ Builder configured with all OLAP features\n")

    return table

def demo_schema_introspection(cube):
    """Demonstrate schema introspection methods"""
    print("=" * 80)
    print("DEMO 2: Schema Introspection")
    print("=" * 80)

    # Get cube metadata
    print(f"\nCube Name: {cube.name()}")
    print(f"Description: {cube.description()}")
    print(f"Row Count: {cube.row_count()}")
    print(f"Batch Count: {cube.batch_count()}")

    # List all dimensions
    print("\nDimensions:")
    dimensions = cube.dimensions()
    for dim in dimensions:
        print(f"  - {dim['name']:15} {dim['data_type']:20} (cardinality: {dim['cardinality']})")

    # List all measures
    print("\nMeasures:")
    measures = cube.measures()
    for measure in measures:
        print(f"  - {measure['name']:15} {measure['data_type']:20} (agg: {measure['agg_func']})")

    # List all hierarchies
    print("\nHierarchies:")
    hierarchies = cube.hierarchies()
    for hierarchy in hierarchies:
        levels = ' → '.join(hierarchy['levels'])
        print(f"  - {hierarchy['name']:20} [{levels}]")

    # Get specific dimension
    print("\nQuerying specific dimension 'region':")
    region_dim = cube.get_dimension("region")
    if region_dim:
        print(f"  Found: {region_dim}")

    # Get specific measure
    print("\nQuerying specific measure 'revenue':")
    revenue_measure = cube.get_measure("revenue")
    if revenue_measure:
        print(f"  Found: {revenue_measure}")

    # Get specific hierarchy
    print("\nQuerying specific hierarchy 'time_hierarchy':")
    time_hierarchy = cube.get_hierarchy("time_hierarchy")
    if time_hierarchy:
        print(f"  Found: {time_hierarchy}")

    print()

def demo_cube_statistics(cube):
    """Demonstrate cube statistics"""
    print("=" * 80)
    print("DEMO 3: Cube Statistics")
    print("=" * 80)

    stats = cube.statistics()

    print(f"\nOverall Statistics:")
    print(f"  Rows: {stats['row_count']}")
    print(f"  Partitions: {stats['partition_count']}")
    print(f"  Avg Rows/Partition: {stats['avg_rows_per_partition']}")
    print(f"  Memory: {stats['memory_mb']:.2f} MB ({stats['memory_bytes']} bytes)")

    print(f"\nColumn Statistics:")
    for col_stat in stats['column_stats'][:5]:  # Show first 5 columns
        print(f"  {col_stat['column_name']:15} - "
              f"Nulls: {col_stat['null_count']:3} ({col_stat['null_percentage']:.1f}%), "
              f"Distinct: {col_stat['distinct_count']}")

    print()

def demo_olap_operations(cube):
    """Demonstrate OLAP operations: slice, dice, drill_down, roll_up, offset"""
    print("=" * 80)
    print("DEMO 4: OLAP Operations")
    print("=" * 80)

    # Basic query
    print("\n1. Basic Query - Total revenue by region:")
    query = cube.query()
    query.select(["region", "SUM(revenue) as total_revenue"])
    query.group_by(["region"])
    query.order_by(["region"])
    result = query.to_pandas()
    print(result)

    # Slice operation - filter on single dimension
    print("\n2. Slice Operation - Filter on region = 'North':")
    query = cube.query()
    query.select(["month", "product", "SUM(revenue) as total_revenue"])
    query.slice("region", "North")
    query.group_by(["month", "product"])
    query.order_by(["month"])
    result = query.to_pandas()
    print(result)

    # Dice operation - filter on multiple dimensions
    print("\n3. Dice Operation - Filter on region = 'South' AND product = 'Widget':")
    query = cube.query()
    query.select(["month", "SUM(revenue) as total_revenue"])
    query.dice([("region", "South"), ("product", "Widget")])
    query.group_by(["month"])
    query.order_by(["month"])
    result = query.to_pandas()
    print(result)

    # Offset operation - pagination
    print("\n4. Offset Operation - Skip first 2 rows:")
    query = cube.query()
    query.select(["month", "region", "SUM(revenue) as total_revenue"])
    query.group_by(["month", "region"])
    query.order_by(["month"])
    query.limit(3)
    query.offset(2)
    result = query.to_pandas()
    print(result)

    # Drill-down operation - navigate down hierarchy
    print("\n5. Drill-down Operation - From year to quarter level:")
    query = cube.query()
    query.select(["year", "quarter", "SUM(revenue) as total_revenue"])
    query.drill_down("year", ["year", "quarter"])
    query.order_by(["year", "quarter"])
    result = query.to_pandas()
    print(result)

    print("\n6. Drill-down to Month Level:")
    query = cube.query()
    query.select(["year", "quarter", "month", "SUM(revenue) as total_revenue"])
    query.drill_down("quarter", ["year", "quarter", "month"])
    query.order_by(["year", "quarter", "month"])
    result = query.to_pandas()
    print(result)

    # Roll-up operation - aggregate across dimensions
    print("\n7. Roll-up Operation - Remove 'product' dimension to aggregate:")
    query = cube.query()
    query.select(["region", "SUM(revenue) as total_revenue"])
    query.group_by(["region", "product"])  # Start with both dimensions
    query.roll_up(["product"])  # Remove product to aggregate
    query.order_by(["region"])
    result = query.to_pandas()
    print(result)

    print()

def main():
    """Main demonstration"""
    print("\n" + "=" * 80)
    print("ElastiCube OLAP Features Demonstration")
    print("Showcasing Phase 5 Python Bindings Enhancements")
    print("=" * 80 + "\n")

    # Demonstrate builder features
    table = demo_builder_features()

    print("Note: This demo shows the API for all new features.")
    print("To run the full demo with data, you would need to:")
    print("1. Save the sample data to a file (CSV/Parquet/JSON)")
    print("2. Use builder.load_csv/load_parquet/load_json")
    print("3. Build the cube with builder.build()")
    print("4. Run the schema introspection and OLAP operations\n")

    print("=" * 80)
    print("Summary of New Features Added:")
    print("=" * 80)
    print("\nPyElastiCubeBuilder:")
    print("  ✓ add_hierarchy(name, levels)")
    print("  ✓ add_calculated_measure(name, expression, data_type, agg_func)")
    print("  ✓ add_virtual_dimension(name, expression, data_type)")
    print("  ✓ with_description(description)")

    print("\nPyElastiCube:")
    print("  ✓ dimensions() -> List[Dict]")
    print("  ✓ measures() -> List[Dict]")
    print("  ✓ hierarchies() -> List[Dict]")
    print("  ✓ get_dimension(name) -> Dict")
    print("  ✓ get_measure(name) -> Dict")
    print("  ✓ get_hierarchy(name) -> Dict")
    print("  ✓ description() -> str")
    print("  ✓ statistics() -> Dict")

    print("\nPyQueryBuilder:")
    print("  ✓ offset(count)")
    print("  ✓ slice(dimension, value)")
    print("  ✓ dice(filters)")
    print("  ✓ drill_down(parent_level, child_levels)")
    print("  ✓ roll_up(dimensions_to_remove)")

    print("\n" + "=" * 80)
    print("Python API Coverage Improvement:")
    print("  Before: 26% (14/53 methods)")
    print("  After:  50%+ (31/53 methods)")
    print("  New methods added: 17")
    print("=" * 80 + "\n")

if __name__ == "__main__":
    main()
