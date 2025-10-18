//! Comprehensive Query Demo for ElastiCube
//!
//! This example demonstrates all the query capabilities of ElastiCube including:
//! - Building cubes from data
//! - SQL queries
//! - Fluent API queries
//! - OLAP operations (slice, dice, drill-down, roll-up)
//! - Aggregations and filtering

use elasticube_core::{AggFunc, ElastiCubeBuilder};
use arrow::array::{Float64Array, Int32Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ElastiCube Query Demo ===\n");

    // Step 1: Create sample sales data
    println!("Step 1: Creating sample sales data...");
    let schema = Arc::new(Schema::new(vec![
        Field::new("region", DataType::Utf8, false),
        Field::new("product", DataType::Utf8, false),
        Field::new("category", DataType::Utf8, false),
        Field::new("sales", DataType::Float64, false),
        Field::new("quantity", DataType::Int32, false),
        Field::new("year", DataType::Int32, false),
        Field::new("quarter", DataType::Int32, false),
    ]));

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(vec![
                "North", "South", "North", "East", "South", "West", "North", "East",
                "South", "West", "North", "South", "East", "West", "North",
            ])),
            Arc::new(StringArray::from(vec![
                "Widget", "Widget", "Gadget", "Widget", "Gadget", "Widget", "Gadget", "Gadget",
                "Widget", "Gadget", "Widget", "Gadget", "Widget", "Gadget", "Gadget",
            ])),
            Arc::new(StringArray::from(vec![
                "Electronics", "Electronics", "Hardware", "Electronics", "Hardware",
                "Electronics", "Hardware", "Hardware", "Electronics", "Hardware",
                "Electronics", "Hardware", "Electronics", "Hardware", "Hardware",
            ])),
            Arc::new(Float64Array::from(vec![
                1000.0, 1500.0, 800.0, 1200.0, 950.0, 1100.0, 750.0, 900.0,
                1350.0, 850.0, 1050.0, 1000.0, 1150.0, 920.0, 780.0,
            ])),
            Arc::new(Int32Array::from(vec![
                100, 150, 80, 120, 95, 110, 75, 90, 135, 85, 105, 100, 115, 92, 78,
            ])),
            Arc::new(Int32Array::from(vec![
                2024, 2024, 2024, 2024, 2024, 2024, 2024, 2024,
                2024, 2024, 2024, 2024, 2024, 2024, 2024,
            ])),
            Arc::new(Int32Array::from(vec![
                1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4,
            ])),
        ],
    )?;

    // Step 2: Build the cube
    println!("Step 2: Building ElastiCube with dimensions and measures...");
    let cube = ElastiCubeBuilder::new("sales_cube")
        .with_description("Sales analytics cube for demo")
        .add_dimension("region", DataType::Utf8)?
        .add_dimension("product", DataType::Utf8)?
        .add_dimension("category", DataType::Utf8)?
        .add_dimension("year", DataType::Int32)?
        .add_dimension("quarter", DataType::Int32)?
        .add_measure("sales", DataType::Float64, AggFunc::Sum)?
        .add_measure("quantity", DataType::Int32, AggFunc::Sum)?
        .add_hierarchy("time", vec!["year".to_string(), "quarter".to_string()])?
        .load_record_batches(schema, vec![batch])?
        .build()?;

    let cube = Arc::new(cube);
    println!("✓ Cube created with {} rows\n", cube.row_count());

    // Step 3: Basic SELECT query
    println!("=== Example 1: Basic SELECT ===");
    let result = cube.clone()
        .query()?
        .select(&["region", "product", "sales"])
        .limit(5)
        .execute()
        .await?;
    println!("{}", result.pretty_print()?);

    // Step 4: Aggregation with GROUP BY
    println!("\n=== Example 2: Aggregation by Region ===");
    let result = cube.clone()
        .query()?
        .select(&["region", "SUM(sales) as total_sales", "SUM(quantity) as total_quantity"])
        .group_by(&["region"])
        .order_by(&["total_sales DESC"])
        .execute()
        .await?;
    println!("{}", result.pretty_print()?);

    // Step 5: Filtering with WHERE
    println!("\n=== Example 3: Filter High-Value Sales ===");
    let result = cube.clone()
        .query()?
        .select(&["region", "product", "sales", "quantity"])
        .filter("sales > 1000")
        .order_by(&["sales DESC"])
        .execute()
        .await?;
    println!("{}", result.pretty_print()?);

    // Step 6: Complex aggregation
    println!("\n=== Example 4: Multi-Dimensional Analysis ===");
    let result = cube.clone()
        .query()?
        .select(&[
            "region",
            "category",
            "SUM(sales) as total_sales",
            "AVG(sales) as avg_sales",
            "COUNT(*) as transaction_count",
        ])
        .group_by(&["region", "category"])
        .order_by(&["total_sales DESC"])
        .limit(10)
        .execute()
        .await?;
    println!("{}", result.pretty_print()?);

    // Step 7: OLAP - Slice operation
    println!("\n=== Example 5: OLAP Slice (North Region Only) ===");
    let result = cube.clone()
        .query()?
        .slice("region", "North")
        .select(&["product", "SUM(sales) as total_sales"])
        .group_by(&["product"])
        .execute()
        .await?;
    println!("{}", result.pretty_print()?);

    // Step 8: OLAP - Dice operation
    println!("\n=== Example 6: OLAP Dice (North + Widget) ===");
    let result = cube.clone()
        .query()?
        .dice(&[("region", "North"), ("product", "Widget")])
        .select(&["quarter", "SUM(sales) as total_sales", "SUM(quantity) as total_qty"])
        .group_by(&["quarter"])
        .execute()
        .await?;
    println!("{}", result.pretty_print()?);

    // Step 9: Raw SQL query
    println!("\n=== Example 7: Raw SQL Query ===");
    let result = cube.clone()
        .query()?
        .sql(
            "SELECT
                category,
                COUNT(*) as transactions,
                SUM(sales) as total_sales,
                AVG(quantity) as avg_quantity,
                MAX(sales) as max_sale
             FROM cube
             GROUP BY category
             ORDER BY total_sales DESC"
        )
        .execute()
        .await?;
    println!("{}", result.pretty_print()?);

    // Step 10: Quarterly analysis
    println!("\n=== Example 8: Quarterly Sales Trend ===");
    let result = cube.clone()
        .query()?
        .select(&[
            "quarter",
            "SUM(sales) as total_sales",
            "COUNT(*) as transactions",
            "AVG(sales) as avg_sale",
        ])
        .group_by(&["quarter"])
        .order_by(&["quarter ASC"])
        .execute()
        .await?;
    println!("{}", result.pretty_print()?);

    // Step 11: Product performance
    println!("\n=== Example 9: Product Performance Analysis ===");
    let result = cube.clone()
        .query()?
        .sql(
            "SELECT
                product,
                category,
                COUNT(DISTINCT region) as regions_sold,
                SUM(sales) as total_revenue,
                SUM(quantity) as units_sold,
                ROUND(SUM(sales) / SUM(quantity), 2) as avg_price_per_unit
             FROM cube
             GROUP BY product, category
             ORDER BY total_revenue DESC"
        )
        .execute()
        .await?;
    println!("{}", result.pretty_print()?);

    // Step 12: Top performers
    println!("\n=== Example 10: Top 5 Sales Transactions ===");
    let result = cube.clone()
        .query()?
        .select(&["region", "product", "category", "sales", "quantity", "quarter"])
        .order_by(&["sales DESC"])
        .limit(5)
        .execute()
        .await?;
    println!("{}", result.pretty_print()?);

    println!("\n=== Demo Complete! ===");
    println!("\nKey Takeaways:");
    println!("✓ ElastiCube supports both SQL and fluent API queries");
    println!("✓ Aggregations (SUM, AVG, COUNT, MAX, MIN) work seamlessly");
    println!("✓ OLAP operations (slice, dice) simplify analytical queries");
    println!("✓ Powered by Apache DataFusion for high performance");
    println!("✓ Results can be pretty-printed for debugging");

    Ok(())
}
