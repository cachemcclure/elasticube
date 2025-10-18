//! Demonstration of Calculated Measures and Virtual Dimensions
//!
//! This example shows how to use ElastiCube's Phase 6.1 features:
//! - Calculated Measures (derived metrics from expressions)
//! - Virtual Dimensions (computed dimensions from expressions)
//!
//! Run with: cargo run --example calculated_fields_demo

use elasticube_core::{
    AggFunc, CalculatedMeasure, ElastiCubeBuilder, VirtualDimension,
};
use arrow::datatypes::DataType;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ElastiCube Calculated Fields Demo ===\n");

    // Build a cube with sales data
    println!("1. Building cube with base measures...");
    let mut builder = ElastiCubeBuilder::new("sales_analysis");

    // Add base dimensions
    builder = builder
        .add_dimension("sale_date", DataType::Date32)?
        .add_dimension("region", DataType::Utf8)?
        .add_dimension("product", DataType::Utf8)?;

    // Add base measures
    builder = builder
        .add_measure("revenue", DataType::Float64, AggFunc::Sum)?
        .add_measure("cost", DataType::Float64, AggFunc::Sum)?
        .add_measure("quantity", DataType::Int32, AggFunc::Sum)?;

    println!("✓ Added 3 dimensions and 3 base measures\n");

    // Add calculated measures
    println!("2. Adding calculated measures...");

    // Profit = Revenue - Cost
    builder = builder.add_calculated_measure(
        "profit",
        "revenue - cost",
        DataType::Float64,
        AggFunc::Sum,
    )?;
    println!("✓ Added 'profit' = revenue - cost");

    // Margin = (Profit / Revenue) * 100
    builder = builder.add_calculated_measure(
        "margin",
        "(profit / revenue) * 100",
        DataType::Float64,
        AggFunc::Avg,
    )?;
    println!("✓ Added 'margin' = (profit / revenue) * 100");

    // Average Unit Price = Revenue / Quantity
    builder = builder.add_calculated_measure(
        "avg_unit_price",
        "revenue / quantity",
        DataType::Float64,
        AggFunc::Avg,
    )?;
    println!("✓ Added 'avg_unit_price' = revenue / quantity\n");

    // Add virtual dimensions
    println!("3. Adding virtual dimensions...");

    // Extract year from sale_date
    builder = builder.add_virtual_dimension(
        "year",
        "EXTRACT(YEAR FROM sale_date)",
        DataType::Int32,
    )?;
    println!("✓ Added 'year' = EXTRACT(YEAR FROM sale_date)");

    // Extract month from sale_date
    builder = builder.add_virtual_dimension(
        "month",
        "EXTRACT(MONTH FROM sale_date)",
        DataType::Int32,
    )?;
    println!("✓ Added 'month' = EXTRACT(MONTH FROM sale_date)");

    // Categorize regions
    builder = builder.add_virtual_dimension(
        "region_category",
        "CASE
            WHEN region IN ('US', 'CA') THEN 'North America'
            WHEN region IN ('UK', 'FR', 'DE') THEN 'Europe'
            ELSE 'Other'
         END",
        DataType::Utf8,
    )?;
    println!("✓ Added 'region_category' = CASE expression for region grouping\n");

    // Summary of what we've built
    println!("4. Cube Schema Summary:");
    println!("   Base Dimensions:      3 (sale_date, region, product)");
    println!("   Virtual Dimensions:   3 (year, month, region_category)");
    println!("   Base Measures:        3 (revenue, cost, quantity)");
    println!("   Calculated Measures:  3 (profit, margin, avg_unit_price)");
    println!("\n=== Demo Complete ===\n");

    // Show how calculated measures would be used in queries
    println!("Example Query Usage:");
    println!("```rust");
    println!("// Query using calculated measures and virtual dimensions");
    println!("let results = cube.query()?");
    println!("    .select(&[");
    println!("        \"year\",                    // Virtual dimension");
    println!("        \"region_category\",         // Virtual dimension");
    println!("        \"SUM(revenue) as total_revenue\",");
    println!("        \"SUM(profit) as total_profit\",      // Calculated measure");
    println!("        \"AVG(margin) as avg_margin\"         // Calculated measure");
    println!("    ])");
    println!("    .group_by(&[\"year\", \"region_category\"])");
    println!("    .order_by(&[\"year DESC\", \"total_profit DESC\"])");
    println!("    .execute()");
    println!("    .await?;");
    println!("```\n");

    // Show best practices
    println!("Best Practices:");
    println!("1. Define base measures/dimensions first");
    println!("2. Add calculated fields that reference base fields");
    println!("3. Use calculated measures for derived metrics (profit, margin, etc.)");
    println!("4. Use virtual dimensions for date parts, categorization, transformations");
    println!("5. Calculated fields are materialized at query time (no storage overhead)");
    println!("\nBenefits:");
    println!("• No need to precompute derived fields");
    println!("• Expressions leverage DataFusion's optimization");
    println!("• Schema remains flexible and maintainable");
    println!("• Supports complex SQL expressions");

    Ok(())
}
