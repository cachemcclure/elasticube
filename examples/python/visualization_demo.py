#!/usr/bin/env python3
"""
Visualization Demo for ElastiCube

Demonstrates the visualization capabilities of ElastiCube using matplotlib and seaborn.
Shows how to create various types of charts directly from cube queries.

Requirements:
    pip install elasticube matplotlib seaborn
    # or
    pip install elasticube[viz]
"""

import os
from elasticube import ElastiCubeBuilder


def main():
    print("=== ElastiCube Visualization Demo ===\n")

    # Get the path to the CSV file
    script_dir = os.path.dirname(os.path.abspath(__file__))
    csv_path = os.path.join(script_dir, "sales_data.csv")

    # Build the cube
    print("Building ElastiCube from CSV data...")
    builder = ElastiCubeBuilder("viz_cube")
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

    # Import matplotlib
    try:
        import matplotlib.pyplot as plt
        import matplotlib
        # Use non-interactive backend for this demo
        matplotlib.use('Agg')
    except ImportError:
        print("Error: matplotlib is not installed")
        print("Install with: pip install matplotlib seaborn")
        print("Or: pip install elasticube[viz]")
        return

    # Example 1: Bar Chart - Sales by Region
    print("=== Example 1: Bar Chart - Sales by Region ===")
    query = cube.query()
    query.select(["region", "SUM(sales) as total_sales"])
    query.group_by(["region"])
    query.order_by(["total_sales DESC"])

    fig, ax = query.plot().bar(
        x="region",
        y="total_sales",
        title="Total Sales by Region",
        ylabel="Sales ($)",
        color="#4CAF50"
    )
    plt.savefig("sales_by_region.png", dpi=150, bbox_inches='tight')
    print("✓ Saved: sales_by_region.png\n")
    plt.close()

    # Example 2: Line Chart - Quarterly Trends
    print("=== Example 2: Line Chart - Quarterly Sales Trend ===")
    query = cube.query()
    query.select([
        "quarter",
        "SUM(sales) as total_sales",
        "AVG(sales) as avg_sales"
    ])
    query.group_by(["quarter"])
    query.order_by(["quarter ASC"])

    fig, ax = query.plot().line(
        x="quarter",
        y=["total_sales", "avg_sales"],
        title="Quarterly Sales Trend",
        xlabel="Quarter",
        ylabel="Sales ($)",
    )
    plt.savefig("quarterly_trend.png", dpi=150, bbox_inches='tight')
    print("✓ Saved: quarterly_trend.png\n")
    plt.close()

    # Example 3: Horizontal Bar Chart - Product Performance
    print("=== Example 3: Horizontal Bar Chart - Top Products ===")
    query = cube.query()
    query.select([
        "product",
        "SUM(sales) as revenue",
        "SUM(quantity) as units"
    ])
    query.group_by(["product"])
    query.order_by(["revenue DESC"])

    # Get data and create horizontal bar chart
    df = query.to_pandas()
    fig, ax = plt.subplots(figsize=(10, 6))
    ax.barh(df["product"], df["revenue"], color="#2196F3")
    ax.set_xlabel("Revenue ($)")
    ax.set_title("Product Performance by Revenue")
    plt.tight_layout()
    plt.savefig("product_performance.png", dpi=150, bbox_inches='tight')
    print("✓ Saved: product_performance.png\n")
    plt.close()

    # Example 4: Heatmap - Region x Category Sales
    print("=== Example 4: Heatmap - Sales by Region and Category ===")
    query = cube.query()
    query.select([
        "region",
        "category",
        "SUM(sales) as total_sales"
    ])
    query.group_by(["region", "category"])

    try:
        fig, ax = query.plot().heatmap(
            x="region",
            y="category",
            values="total_sales",
            title="Sales Heatmap: Region × Category",
            cmap="YlOrRd"
        )
        plt.savefig("sales_heatmap.png", dpi=150, bbox_inches='tight')
        print("✓ Saved: sales_heatmap.png\n")
        plt.close()
    except ImportError:
        print("⚠ Seaborn not installed, skipping heatmap")
        print("  Install with: pip install seaborn\n")

    # Example 5: Scatter Plot - Sales vs Quantity
    print("=== Example 5: Scatter Plot - Sales vs Quantity by Region ===")
    query = cube.query()
    query.select([
        "region",
        "SUM(sales) as total_sales",
        "SUM(quantity) as total_quantity"
    ])
    query.group_by(["region"])

    fig, ax = query.plot().scatter(
        x="total_quantity",
        y="total_sales",
        title="Sales vs Quantity by Region",
        xlabel="Total Quantity",
        ylabel="Total Sales ($)",
    )

    # Add region labels to points
    df = query.to_pandas()
    for idx, row in df.iterrows():
        ax.annotate(
            row['region'],
            (row['total_quantity'], row['total_sales']),
            xytext=(5, 5),
            textcoords='offset points',
            fontsize=9
        )

    plt.savefig("sales_vs_quantity.png", dpi=150, bbox_inches='tight')
    print("✓ Saved: sales_vs_quantity.png\n")
    plt.close()

    # Example 6: Pie Chart - Market Share by Region
    print("=== Example 6: Pie Chart - Market Share by Region ===")
    query = cube.query()
    query.select(["region", "SUM(sales) as total"])
    query.group_by(["region"])

    fig, ax = query.plot().pie(
        labels="region",
        values="total",
        title="Market Share by Region"
    )
    plt.savefig("market_share.png", dpi=150, bbox_inches='tight')
    print("✓ Saved: market_share.png\n")
    plt.close()

    # Example 7: Multiple Subplots - Comprehensive Dashboard
    print("=== Example 7: Creating a Dashboard with Multiple Charts ===")
    fig, ((ax1, ax2), (ax3, ax4)) = plt.subplots(2, 2, figsize=(14, 10))

    # Chart 1: Sales by Region (Bar)
    query = cube.query()
    query.select(["region", "SUM(sales) as total"])
    query.group_by(["region"])
    query.order_by(["total DESC"])
    df1 = query.to_pandas()
    ax1.bar(df1["region"], df1["total"], color="#4CAF50")
    ax1.set_title("Sales by Region")
    ax1.set_ylabel("Sales ($)")
    ax1.tick_params(axis='x', rotation=45)

    # Chart 2: Quarterly Trend (Line)
    query = cube.query()
    query.select(["quarter", "SUM(sales) as total"])
    query.group_by(["quarter"])
    query.order_by(["quarter ASC"])
    df2 = query.to_pandas()
    ax2.plot(df2["quarter"], df2["total"], marker='o', color="#2196F3", linewidth=2)
    ax2.set_title("Quarterly Sales Trend")
    ax2.set_xlabel("Quarter")
    ax2.set_ylabel("Sales ($)")

    # Chart 3: Category Distribution (Pie)
    query = cube.query()
    query.select(["category", "SUM(sales) as total"])
    query.group_by(["category"])
    df3 = query.to_pandas()
    ax3.pie(df3["total"], labels=df3["category"], autopct='%1.1f%%')
    ax3.set_title("Sales by Category")

    # Chart 4: Product Rankings (Horizontal Bar)
    query = cube.query()
    query.select(["product", "SUM(quantity) as units"])
    query.group_by(["product"])
    query.order_by(["units DESC"])
    df4 = query.to_pandas()
    ax4.barh(df4["product"], df4["units"], color="#FF9800")
    ax4.set_title("Units Sold by Product")
    ax4.set_xlabel("Units")

    plt.tight_layout()
    plt.savefig("sales_dashboard.png", dpi=150, bbox_inches='tight')
    print("✓ Saved: sales_dashboard.png\n")
    plt.close()

    # Summary
    print("\n=== Visualization Summary ===")
    print("✓ Created 7 different chart types:")
    print("  1. Bar chart - Regional sales")
    print("  2. Line chart - Quarterly trends")
    print("  3. Horizontal bar chart - Product performance")
    print("  4. Heatmap - Region × Category analysis")
    print("  5. Scatter plot - Sales vs quantity correlation")
    print("  6. Pie chart - Market share")
    print("  7. Dashboard - Multi-chart layout")
    print("\n💡 All charts have been saved as PNG files in the current directory")
    print("\n📊 Use query.plot() to create interactive visualizations in Jupyter!")


if __name__ == "__main__":
    main()
