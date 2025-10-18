//! Basic Cube Building Example
//!
//! This example demonstrates the fundamental concepts of building an ElastiCube:
//! 1. Creating a cube with dimensions and measures
//! 2. Loading data from a CSV file
//! 3. Executing basic queries
//! 4. Working with query results
//!
//! Run with: cargo run --example basic_cube_building

use arrow_schema::DataType;
use elasticube_core::{AggFunc, ElastiCubeBuilder, Result};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== ElastiCube: Basic Cube Building Example ===\n");

    // Step 1: Create sample data
    println!("Step 1: Creating sample sales data...");
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let csv_path = temp_dir.path().join("sales_data.csv");
    let mut file = File::create(&csv_path).expect("Failed to create CSV file");

    writeln!(
        file,
        "date,store,product,category,sales,quantity"
    )
    .expect("Failed to write CSV header");
    writeln!(file, "2024-01-01,Store A,Laptop,Electronics,1200.00,2").unwrap();
    writeln!(file, "2024-01-01,Store B,Phone,Electronics,800.00,3").unwrap();
    writeln!(file, "2024-01-01,Store A,Desk,Furniture,350.00,1").unwrap();
    writeln!(file, "2024-01-02,Store A,Laptop,Electronics,1800.00,3").unwrap();
    writeln!(file, "2024-01-02,Store B,Chair,Furniture,200.00,4").unwrap();
    writeln!(file, "2024-01-03,Store A,Phone,Electronics,1600.00,6").unwrap();
    writeln!(file, "2024-01-03,Store B,Laptop,Electronics,2400.00,4").unwrap();
    file.flush().unwrap();

    println!("  ✓ Created sales_data.csv with 7 rows\n");

    // Step 2: Build the cube
    println!("Step 2: Building the ElastiCube...");
    let cube = ElastiCubeBuilder::new("sales_cube")
        // Define dimensions (categorical fields for slicing/dicing)
        .add_dimension("date", DataType::Utf8)?
        .add_dimension("store", DataType::Utf8)?
        .add_dimension("product", DataType::Utf8)?
        .add_dimension("category", DataType::Utf8)?
        // Define measures (numeric fields for aggregation)
        .add_measure("sales", DataType::Float64, AggFunc::Sum)?
        .add_measure("quantity", DataType::Int64, AggFunc::Sum)?
        // Load data from CSV
        .load_csv(csv_path.to_str().unwrap())
        .build()?;

    println!("  ✓ Cube built successfully!");
    println!("  - Total rows: {}", cube.row_count());
    println!("  - Dimensions: {}", cube.dimensions().len());
    println!("  - Measures: {}\n", cube.measures().len());

    // Step 3: Query the cube - Total sales by store
    println!("Step 3: Querying total sales by store...");
    let result = cube
        .query()?
        .select(&["store", "sum(sales) as total_sales", "sum(quantity) as total_qty"])
        .group_by(&["store"])
        .order_by(&["total_sales DESC"])
        .execute()
        .await?;

    println!("{}", result);

    // Step 4: Query by category
    println!("\nStep 4: Querying sales by category...");
    let result = cube
        .query()?
        .select(&["category", "sum(sales) as total_sales"])
        .group_by(&["category"])
        .order_by(&["total_sales DESC"])
        .execute()
        .await?;

    println!("{}", result);

    // Step 5: Filter query - Electronics only
    println!("\nStep 5: Filtering for Electronics category...");
    let result = cube
        .query()?
        .select(&["product", "sum(sales) as total_sales", "sum(quantity) as units_sold"])
        .filter("category = 'Electronics'")
        .group_by(&["product"])
        .order_by(&["total_sales DESC"])
        .execute()
        .await?;

    println!("{}", result);

    // Step 6: Using SQL query
    println!("\nStep 6: Using direct SQL query...");
    let result = cube
        .query()?
        .sql("SELECT date, SUM(sales) as daily_total FROM cube GROUP BY date ORDER BY date")
        .execute()
        .await?;

    println!("{}", result);

    // Step 7: Complex aggregation
    println!("\nStep 7: Multi-dimensional analysis (store + category)...");
    let result = cube
        .query()?
        .select(&[
            "store",
            "category",
            "sum(sales) as total_sales",
            "avg(sales) as avg_sales",
            "count(sales) as num_transactions",
        ])
        .group_by(&["store", "category"])
        .order_by(&["store", "total_sales DESC"])
        .execute()
        .await?;

    println!("{}", result);

    println!("\n=== Example Complete ===");
    println!("\nKey Concepts Demonstrated:");
    println!("  1. ElastiCubeBuilder fluent API");
    println!("  2. Defining dimensions (for slicing) and measures (for aggregation)");
    println!("  3. Loading data from CSV with schema inference");
    println!("  4. Multiple query patterns:");
    println!("     - SELECT + GROUP BY + ORDER BY");
    println!("     - Filtering with WHERE conditions");
    println!("     - Direct SQL queries");
    println!("     - Multi-dimensional aggregations");
    println!("  5. Working with query results");

    Ok(())
}
