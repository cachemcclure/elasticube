//! Data Updates Demo
//!
//! This example demonstrates ElastiCube's data update capabilities:
//! - Appending new rows incrementally
//! - Deleting rows based on filter predicates
//! - Updating existing rows
//! - Consolidating fragmented batches
//!
//! Run with: cargo run --example data_updates_demo

use elasticube_core::{AggFunc, ElastiCubeBuilder};
use arrow::array::{Float64Array, Int32Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ElastiCube Data Updates Demo ===\n");

    // ============================================================
    // Step 1: Create initial cube with sales data
    // ============================================================
    println!("Step 1: Creating initial sales cube...");

    let schema = Arc::new(Schema::new(vec![
        Field::new("date", DataType::Utf8, false),
        Field::new("region", DataType::Utf8, false),
        Field::new("product", DataType::Utf8, false),
        Field::new("sales", DataType::Float64, false),
        Field::new("quantity", DataType::Int32, false),
    ]));

    let initial_batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(vec!["2024-01-01", "2024-01-01", "2024-01-02", "2024-01-02"])),
            Arc::new(StringArray::from(vec!["North", "South", "East", "West"])),
            Arc::new(StringArray::from(vec!["Widget", "Gadget", "Widget", "Doohickey"])),
            Arc::new(Float64Array::from(vec![1000.0, 1500.0, 1200.0, 2000.0])),
            Arc::new(Int32Array::from(vec![10, 15, 12, 20])),
        ],
    )?;

    let mut cube = ElastiCubeBuilder::new("sales_tracker")
        .add_dimension("date", DataType::Utf8)?
        .add_dimension("region", DataType::Utf8)?
        .add_dimension("product", DataType::Utf8)?
        .add_measure("sales", DataType::Float64, AggFunc::Sum)?
        .add_measure("quantity", DataType::Int32, AggFunc::Sum)?
        .with_data(vec![initial_batch])?
        .build()?;

    println!("Initial cube created:");
    println!("  Rows: {}", cube.row_count());
    println!("  Batches: {}\n", cube.batch_count());

    // ============================================================
    // Step 2: Append new rows (incremental data loading)
    // ============================================================
    println!("Step 2: Appending new sales data...");

    let new_batch = RecordBatch::try_new(
        cube.arrow_schema().clone(),
        vec![
            Arc::new(StringArray::from(vec!["2024-01-03", "2024-01-03"])),
            Arc::new(StringArray::from(vec!["North", "Central"])),
            Arc::new(StringArray::from(vec!["Gadget", "Widget"])),
            Arc::new(Float64Array::from(vec![1750.0, 1350.0])),
            Arc::new(Int32Array::from(vec![17, 13])),
        ],
    )?;

    let rows_added = cube.append_rows(new_batch)?;
    println!("Added {} new rows", rows_added);
    println!("  Total rows: {}", cube.row_count());
    println!("  Total batches: {}\n", cube.batch_count());

    // ============================================================
    // Step 3: Query the updated data
    // ============================================================
    println!("Step 3: Querying total sales by region...");

    let result = Arc::new(cube.clone())
        .query()?
        .select(&["region", "SUM(sales) as total_sales"])
        .group_by(&["region"])
        .order_by(&["total_sales DESC"])
        .execute()
        .await?;

    result.pretty_print()?;
    println!();

    // ============================================================
    // Step 4: Delete rows based on a filter
    // ============================================================
    println!("Step 4: Deleting sales transactions < $1200...");

    let deleted = cube.delete_rows("sales < 1200").await?;
    println!("Deleted {} rows", deleted);
    println!("  Remaining rows: {}\n", cube.row_count());

    // ============================================================
    // Step 5: Update specific rows
    // ============================================================
    println!("Step 5: Updating North region sales for Widget...");

    // Create updated data for North region Widget sales
    let updated_batch = RecordBatch::try_new(
        cube.arrow_schema().clone(),
        vec![
            Arc::new(StringArray::from(vec!["2024-01-03"])),
            Arc::new(StringArray::from(vec!["North"])),
            Arc::new(StringArray::from(vec!["Gadget"])),
            Arc::new(Float64Array::from(vec![2500.0])), // Corrected sales amount
            Arc::new(Int32Array::from(vec![25])),       // Corrected quantity
        ],
    )?;

    let (rows_deleted, rows_added) = cube
        .update_rows("region = 'North' AND product = 'Gadget'", updated_batch)
        .await?;

    println!("Update complete:");
    println!("  Rows deleted: {}", rows_deleted);
    println!("  Rows added: {}", rows_added);
    println!("  Total rows: {}\n", cube.row_count());

    // ============================================================
    // Step 6: Query after updates
    // ============================================================
    println!("Step 6: Querying updated data...");

    let updated_result = Arc::new(cube.clone())
        .query()?
        .select(&["date", "region", "product", "sales", "quantity"])
        .order_by(&["date", "region"])
        .execute()
        .await?;

    updated_result.pretty_print()?;
    println!();

    // ============================================================
    // Step 7: Consolidate batches
    // ============================================================
    println!("Step 7: Consolidating fragmented batches...");
    println!("  Before consolidation: {} batches", cube.batch_count());

    let old_batch_count = cube.consolidate_batches()?;
    println!("  After consolidation: {} batch", cube.batch_count());
    println!("  Consolidated {} batches into 1\n", old_batch_count);

    // ============================================================
    // Step 8: Batch append (multiple batches at once)
    // ============================================================
    println!("Step 8: Appending multiple batches at once...");

    let batch1 = RecordBatch::try_new(
        cube.arrow_schema().clone(),
        vec![
            Arc::new(StringArray::from(vec!["2024-01-04"])),
            Arc::new(StringArray::from(vec!["South"])),
            Arc::new(StringArray::from(vec!["Widget"])),
            Arc::new(Float64Array::from(vec![1600.0])),
            Arc::new(Int32Array::from(vec![16])),
        ],
    )?;

    let batch2 = RecordBatch::try_new(
        cube.arrow_schema().clone(),
        vec![
            Arc::new(StringArray::from(vec!["2024-01-04"])),
            Arc::new(StringArray::from(vec!["East"])),
            Arc::new(StringArray::from(vec!["Gadget"])),
            Arc::new(Float64Array::from(vec![1850.0])),
            Arc::new(Int32Array::from(vec![18])),
        ],
    )?;

    let total_added = cube.append_batches(vec![batch1, batch2])?;
    println!("Added {} rows from {} batches", total_added, 2);
    println!("  Total rows: {}", cube.row_count());
    println!("  Total batches: {}\n", cube.batch_count());

    // ============================================================
    // Step 9: Final summary statistics
    // ============================================================
    println!("Step 9: Final cube statistics...");

    let final_result = Arc::new(cube.clone())
        .query()?
        .select(&[
            "COUNT(*) as total_transactions",
            "SUM(sales) as total_revenue",
            "SUM(quantity) as total_quantity",
            "AVG(sales) as avg_sale",
        ])
        .execute()
        .await?;

    final_result.pretty_print()?;
    println!();

    // Show cube memory statistics
    let stats = cube.statistics();
    println!("Cube memory usage:");
    println!("{}", stats.summary());

    println!("\n=== Demo Complete ===");
    println!("This demo showed:");
    println!("✓ Appending individual rows");
    println!("✓ Appending multiple batches");
    println!("✓ Deleting rows with SQL filters");
    println!("✓ Updating rows (delete + append pattern)");
    println!("✓ Consolidating fragmented batches");
    println!("✓ Querying updated data");

    Ok(())
}
