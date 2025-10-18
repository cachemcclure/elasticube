//! Complete demonstration of Query Materialization for Calculated Fields
//!
//! This example shows how calculated measures and virtual dimensions are
//! automatically expanded in queries, allowing you to use them naturally
//! without manually writing the underlying expressions.
//!
//! Run with: cargo run --example query_materialization_demo

use elasticube_core::{AggFunc, ElastiCubeBuilder};
use arrow::array::{Date32Array, Float64Array, Int32Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== ElastiCube Query Materialization Demo ===\n");

    // Create sample sales data
    println!("1. Creating sample sales data...");
    let schema = Arc::new(Schema::new(vec![
        Field::new("sale_date", DataType::Date32, false),
        Field::new("region", DataType::Utf8, false),
        Field::new("product", DataType::Utf8, false),
        Field::new("revenue", DataType::Float64, false),
        Field::new("cost", DataType::Float64, false),
        Field::new("quantity", DataType::Int32, false),
    ]));

    // Days since epoch: 19358 = Jan 1, 2023, 19723 = Jan 1, 2024
    let dates = Arc::new(Date32Array::from(vec![
        19358, 19400, 19450, 19500, // 2023
        19723, 19765, 19810, 19850, // 2024
    ]));

    let regions = Arc::new(StringArray::from(vec![
        "North", "South", "East", "West",
        "North", "South", "East", "West",
    ]));

    let products = Arc::new(StringArray::from(vec![
        "Widget", "Gadget", "Widget", "Gadget",
        "Widget", "Gadget", "Widget", "Gadget",
    ]));

    let revenue = Arc::new(Float64Array::from(vec![
        1200.0, 1800.0, 1500.0, 2100.0,
        1400.0, 2000.0, 1700.0, 2300.0,
    ]));

    let cost = Arc::new(Float64Array::from(vec![
        700.0, 1000.0, 850.0, 1200.0,
        800.0, 1100.0, 950.0, 1300.0,
    ]));

    let quantity = Arc::new(Int32Array::from(vec![
        12, 18, 15, 21,
        14, 20, 17, 23,
    ]));

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![dates, regions, products, revenue, cost, quantity],
    )?;

    println!("✓ Created {} rows of sales data\n", batch.num_rows());

    // Build cube with calculated fields
    println!("2. Building cube with calculated fields...");

    let cube = Arc::new(
        ElastiCubeBuilder::new("sales_analysis")
            // Base dimensions
            .add_dimension("sale_date", DataType::Date32)?
            .add_dimension("region", DataType::Utf8)?
            .add_dimension("product", DataType::Utf8)?

            // Base measures
            .add_measure("revenue", DataType::Float64, AggFunc::Sum)?
            .add_measure("cost", DataType::Float64, AggFunc::Sum)?
            .add_measure("quantity", DataType::Int32, AggFunc::Sum)?

            // Virtual dimension: Extract year from date
            .add_virtual_dimension(
                "year",
                "EXTRACT(YEAR FROM sale_date)",
                DataType::Int32,
            )?

            // Calculated measure: Profit = Revenue - Cost
            .add_calculated_measure(
                "profit",
                "revenue - cost",
                DataType::Float64,
                AggFunc::Sum,
            )?

            // Calculated measure: Margin % = (Profit / Revenue) * 100
            .add_calculated_measure(
                "margin",
                "(profit / revenue) * 100",  // References "profit" calculated measure!
                DataType::Float64,
                AggFunc::Avg,
            )?

            // Calculated measure: Average Unit Price
            .add_calculated_measure(
                "avg_unit_price",
                "revenue / quantity",
                DataType::Float64,
                AggFunc::Avg,
            )?

            .with_data(vec![batch])?
            .build()?,
    );

    println!("✓ Cube built with:");
    println!("  - 3 base dimensions (sale_date, region, product)");
    println!("  - 1 virtual dimension (year)");
    println!("  - 3 base measures (revenue, cost, quantity)");
    println!("  - 3 calculated measures (profit, margin, avg_unit_price)\n");

    // Example 1: Simple calculated measure query
    println!("3. Example 1: Query using calculated measures");
    println!("   Query: SELECT region, SUM(profit) as total_profit GROUP BY region\n");

    let result = cube
        .clone()
        .query()?
        .select(&["region", "SUM(profit) as total_profit"])
        .group_by(&["region"])
        .order_by(&["total_profit DESC"])
        .execute()
        .await?;

    println!("   Results:");
    println!("{}", result.pretty_print()?);
    println!("   ✓ 'profit' was automatically expanded to '(revenue - cost)'\n");

    // Example 2: Virtual dimension query
    println!("4. Example 2: Query using virtual dimension");
    println!("   Query: SELECT year, SUM(revenue) as total GROUP BY year\n");

    let result = cube
        .clone()
        .query()?
        .select(&["year", "SUM(revenue) as total"])
        .group_by(&["year"])
        .execute()
        .await?;

    println!("   Results:");
    println!("{}", result.pretty_print()?);
    println!("   ✓ 'year' was expanded to 'EXTRACT(YEAR FROM sale_date)'\n");

    // Example 3: Nested calculated measures
    println!("5. Example 3: Query using nested calculated measure");
    println!("   Query: SELECT region, AVG(margin) as avg_margin GROUP BY region\n");

    let result = cube
        .clone()
        .query()?
        .select(&["region", "AVG(margin) as avg_margin"])
        .group_by(&["region"])
        .order_by(&["avg_margin DESC"])
        .execute()
        .await?;

    println!("   Results:");
    println!("{}", result.pretty_print()?);
    println!("   ✓ 'margin' was expanded recursively:");
    println!("     margin → (profit / revenue) * 100");
    println!("     → ((revenue - cost) / revenue) * 100\n");

    // Example 4: Filter using calculated measure
    println!("6. Example 4: Filter using calculated measure");
    println!("   Query: SELECT region, profit WHERE profit > 550\n");

    let result = cube
        .clone()
        .query()?
        .select(&["region", "product", "profit"])
        .filter("profit > 550")
        .execute()
        .await?;

    println!("   Results:");
    println!("{}", result.pretty_print()?);
    println!("   ✓ Filter 'profit > 550' expanded to '(revenue - cost) > 550'\n");

    // Example 5: Complex query with multiple calculated fields
    println!("7. Example 5: Complex multi-dimensional analysis");
    println!("   Combining virtual dimensions, calculated measures, and filters\n");

    let result = cube
        .clone()
        .query()?
        .select(&[
            "year",
            "region",
            "SUM(revenue) as total_revenue",
            "SUM(profit) as total_profit",
            "AVG(margin) as avg_margin",
            "AVG(avg_unit_price) as avg_price",
        ])
        .group_by(&["year", "region"])
        .filter("year >= 2023")
        .order_by(&["year", "total_profit DESC"])
        .execute()
        .await?;

    println!("   Results:");
    println!("{}", result.pretty_print()?);
    println!("   ✓ All calculated fields expanded automatically:");
    println!("     - year: EXTRACT(YEAR FROM sale_date)");
    println!("     - profit: (revenue - cost)");
    println!("     - margin: ((revenue - cost) / revenue) * 100");
    println!("     - avg_unit_price: revenue / quantity\n");

    // Summary
    println!("=== Summary ===\n");
    println!("Query Materialization enables you to:");
    println!("1. ✓ Reference calculated measures in SELECT clauses");
    println!("2. ✓ Reference virtual dimensions in GROUP BY clauses");
    println!("3. ✓ Filter on calculated fields in WHERE clauses");
    println!("4. ✓ Order by calculated fields in ORDER BY clauses");
    println!("5. ✓ Use nested calculated measures (recursive expansion)");
    println!("6. ✓ Combine multiple calculated fields in one query\n");

    println!("Benefits:");
    println!("• Write cleaner, more intuitive queries");
    println!("• Reuse business logic defined in the schema");
    println!("• No need to remember underlying expressions");
    println!("• Automatic recursive expansion for nested fields");
    println!("• Consistent calculations across all queries\n");

    Ok(())
}
