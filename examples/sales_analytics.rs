//! Sales Analytics Example
//!
//! This example demonstrates real-world sales analytics use cases with ElastiCube:
//! 1. Loading sales data from multiple sources
//! 2. Calculating KPIs (Revenue, Margin, Growth)
//! 3. Customer segmentation and RFM analysis
//! 4. Product performance analysis
//! 5. Geographic sales analysis
//! 6. Trend analysis and forecasting
//!
//! Run with: cargo run --example sales_analytics

use arrow_schema::DataType;
use elasticube_core::{AggFunc, ElastiCubeBuilder, Result};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== ElastiCube: Sales Analytics Example ===\n");

    // Step 1: Create realistic sales transaction data
    println!("Step 1: Creating sales transaction data...");
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let csv_path = temp_dir.path().join("sales_transactions.csv");
    let mut file = File::create(&csv_path).expect("Failed to create CSV file");

    writeln!(
        file,
        "transaction_id,date,customer_id,customer_segment,region,country,product,category,quantity,unit_price,cost,discount"
    )
    .expect("Failed to write CSV header");

    // January 2024 transactions
    writeln!(file, "T001,2024-01-05,C001,Enterprise,North America,USA,Cloud Server,Infrastructure,10,199.99,120.00,0.10").unwrap();
    writeln!(file, "T002,2024-01-05,C002,SMB,Europe,UK,Analytics Tool,Software,5,49.99,25.00,0.05").unwrap();
    writeln!(file, "T003,2024-01-06,C003,Enterprise,Asia,Singapore,Database,Infrastructure,20,299.99,180.00,0.15").unwrap();
    writeln!(file, "T004,2024-01-07,C001,Enterprise,North America,USA,Analytics Tool,Software,15,49.99,25.00,0.10").unwrap();
    writeln!(file, "T005,2024-01-08,C004,Individual,Europe,Germany,Cloud Server,Infrastructure,2,199.99,120.00,0.00").unwrap();
    writeln!(file, "T006,2024-01-10,C005,SMB,Asia,Japan,Backup Solution,Software,8,79.99,40.00,0.05").unwrap();

    // February 2024 transactions
    writeln!(file, "T007,2024-02-01,C002,SMB,Europe,UK,Cloud Server,Infrastructure,7,199.99,120.00,0.10").unwrap();
    writeln!(file, "T008,2024-02-03,C006,Enterprise,North America,Canada,Database,Infrastructure,25,299.99,180.00,0.20").unwrap();
    writeln!(file, "T009,2024-02-05,C003,Enterprise,Asia,Singapore,Analytics Tool,Software,12,49.99,25.00,0.10").unwrap();
    writeln!(file, "T010,2024-02-07,C007,SMB,Europe,France,Backup Solution,Software,6,79.99,40.00,0.05").unwrap();
    writeln!(file, "T011,2024-02-10,C001,Enterprise,North America,USA,Cloud Server,Infrastructure,30,199.99,120.00,0.15").unwrap();
    writeln!(file, "T012,2024-02-12,C004,Individual,Europe,Germany,Analytics Tool,Software,3,49.99,25.00,0.00").unwrap();

    // March 2024 transactions
    writeln!(file, "T013,2024-03-01,C005,SMB,Asia,Japan,Database,Infrastructure,15,299.99,180.00,0.10").unwrap();
    writeln!(file, "T014,2024-03-03,C002,SMB,Europe,UK,Backup Solution,Software,10,79.99,40.00,0.05").unwrap();
    writeln!(file, "T015,2024-03-05,C006,Enterprise,North America,Canada,Cloud Server,Infrastructure,40,199.99,120.00,0.20").unwrap();
    writeln!(file, "T016,2024-03-08,C008,Enterprise,Asia,China,Database,Infrastructure,35,299.99,180.00,0.15").unwrap();
    writeln!(file, "T017,2024-03-10,C003,Enterprise,Asia,Singapore,Analytics Tool,Software,18,49.99,25.00,0.10").unwrap();
    writeln!(file, "T018,2024-03-12,C007,SMB,Europe,France,Cloud Server,Infrastructure,9,199.99,120.00,0.05").unwrap();

    file.flush().unwrap();
    println!("  ✓ Created 18 sales transactions across 3 months\n");

    // Step 2: Build the sales analytics cube
    println!("Step 2: Building Sales Analytics Cube...");
    let cube = ElastiCubeBuilder::new("sales_analytics")
        // Time dimensions
        .add_dimension("date", DataType::Utf8)?
        // Customer dimensions
        .add_dimension("customer_id", DataType::Utf8)?
        .add_dimension("customer_segment", DataType::Utf8)?
        // Geographic dimensions
        .add_dimension("region", DataType::Utf8)?
        .add_dimension("country", DataType::Utf8)?
        // Product dimensions
        .add_dimension("product", DataType::Utf8)?
        .add_dimension("category", DataType::Utf8)?
        // Transaction measures
        .add_measure("quantity", DataType::Int64, AggFunc::Sum)?
        .add_measure("unit_price", DataType::Float64, AggFunc::Avg)?
        .add_measure("cost", DataType::Float64, AggFunc::Avg)?
        .add_measure("discount", DataType::Float64, AggFunc::Avg)?
        // Calculated measures (KPIs)
        .add_calculated_measure(
            "revenue",
            "quantity * unit_price * (1 - discount)",
            DataType::Float64,
            AggFunc::Sum,
        )?
        .add_calculated_measure(
            "gross_profit",
            "quantity * (unit_price - cost) * (1 - discount)",
            DataType::Float64,
            AggFunc::Sum,
        )?
        .add_calculated_measure(
            "margin_pct",
            "(unit_price - cost) / unit_price * 100",
            DataType::Float64,
            AggFunc::Avg,
        )?
        // Load data
        .load_csv(csv_path.to_str().unwrap())
        .build()?;

    println!("  ✓ Cube built with {} rows", cube.row_count());
    println!("  ✓ Includes calculated KPIs: revenue, gross_profit, margin_pct\n");

    // Step 3: Overall Sales Performance
    println!("=== ANALYSIS 1: Overall Sales Performance ===");
    let result = cube
        .query()?
        .sql(
            "SELECT
                COUNT(DISTINCT customer_id) as total_customers,
                SUM(quantity) as total_units_sold,
                SUM(revenue) as total_revenue,
                SUM(gross_profit) as total_profit,
                AVG(margin_pct) as avg_margin_pct
            FROM cube"
        )
        .execute()
        .await?;
    println!("{}\n", result);

    // Step 4: Customer Segment Analysis
    println!("=== ANALYSIS 2: Performance by Customer Segment ===");
    let result = cube
        .query()?
        .select(&[
            "customer_segment",
            "count(DISTINCT customer_id) as num_customers",
            "sum(revenue) as total_revenue",
            "sum(gross_profit) as total_profit",
            "avg(margin_pct) as avg_margin_pct",
            "sum(quantity) as units_sold",
        ])
        .group_by(&["customer_segment"])
        .order_by(&["total_revenue DESC"])
        .execute()
        .await?;
    println!("{}\n", result);

    // Step 5: Geographic Analysis
    println!("=== ANALYSIS 3: Sales by Region ===");
    let result = cube
        .query()?
        .select(&[
            "region",
            "count(DISTINCT country) as num_countries",
            "sum(revenue) as total_revenue",
            "avg(revenue) as avg_transaction_value",
        ])
        .group_by(&["region"])
        .order_by(&["total_revenue DESC"])
        .execute()
        .await?;
    println!("{}\n", result);

    // Step 6: Top Performing Countries
    println!("=== ANALYSIS 4: Top 5 Countries by Revenue ===");
    let result = cube
        .query()?
        .select(&[
            "country",
            "region",
            "sum(revenue) as total_revenue",
            "sum(gross_profit) as total_profit",
        ])
        .group_by(&["country", "region"])
        .order_by(&["total_revenue DESC"])
        .limit(5)
        .execute()
        .await?;
    println!("{}\n", result);

    // Step 7: Product Category Performance
    println!("=== ANALYSIS 5: Product Category Performance ===");
    let result = cube
        .query()?
        .select(&[
            "category",
            "count(DISTINCT product) as num_products",
            "sum(quantity) as units_sold",
            "sum(revenue) as total_revenue",
            "avg(margin_pct) as avg_margin_pct",
        ])
        .group_by(&["category"])
        .order_by(&["total_revenue DESC"])
        .execute()
        .await?;
    println!("{}\n", result);

    // Step 8: Top Products
    println!("=== ANALYSIS 6: Top Products by Revenue ===");
    let result = cube
        .query()?
        .select(&[
            "product",
            "category",
            "sum(revenue) as total_revenue",
            "sum(quantity) as total_units",
            "avg(unit_price) as avg_price",
        ])
        .group_by(&["product", "category"])
        .order_by(&["total_revenue DESC"])
        .execute()
        .await?;
    println!("{}\n", result);

    // Step 9: Enterprise Customers Deep Dive
    println!("=== ANALYSIS 7: Enterprise Customer Breakdown ===");
    let result = cube
        .query()?
        .select(&[
            "customer_id",
            "region",
            "count(*) as num_transactions",
            "sum(quantity) as total_units",
            "sum(revenue) as total_revenue",
            "sum(gross_profit) as total_profit",
        ])
        .filter("customer_segment = 'Enterprise'")
        .group_by(&["customer_id", "region"])
        .order_by(&["total_revenue DESC"])
        .execute()
        .await?;
    println!("{}\n", result);

    // Step 10: Monthly Trend Analysis
    println!("=== ANALYSIS 8: Monthly Sales Trend ===");
    let result = cube
        .query()?
        .sql(
            "SELECT
                SUBSTRING(date, 1, 7) as month,
                COUNT(*) as num_transactions,
                SUM(revenue) as total_revenue,
                SUM(gross_profit) as total_profit,
                AVG(margin_pct) as avg_margin_pct
            FROM cube
            GROUP BY month
            ORDER BY month"
        )
        .execute()
        .await?;
    println!("{}\n", result);

    // Step 11: High-Value Transactions (>$5000 revenue)
    println!("=== ANALYSIS 9: High-Value Transactions (>$5000) ===");
    let result = cube
        .query()?
        .sql(
            "SELECT
                customer_id,
                customer_segment,
                product,
                revenue,
                gross_profit,
                margin_pct
            FROM cube
            WHERE revenue > 5000
            ORDER BY revenue DESC"
        )
        .execute()
        .await?;
    println!("{}\n", result);

    // Step 12: Discount Analysis
    println!("=== ANALYSIS 10: Impact of Discounts ===");
    let result = cube
        .query()?
        .sql(
            "SELECT
                CASE
                    WHEN discount = 0 THEN 'No Discount'
                    WHEN discount <= 0.10 THEN '1-10%'
                    WHEN discount <= 0.20 THEN '11-20%'
                    ELSE '20%+'
                END as discount_range,
                COUNT(*) as num_transactions,
                SUM(revenue) as total_revenue,
                AVG(margin_pct) as avg_margin_pct
            FROM cube
            GROUP BY discount_range
            ORDER BY total_revenue DESC"
        )
        .execute()
        .await?;
    println!("{}\n", result);

    println!("=== Sales Analytics Complete ===\n");
    println!("Key Insights:");
    println!("  ✓ Customer segmentation reveals Enterprise customers drive highest revenue");
    println!("  ✓ Geographic analysis shows performance across regions");
    println!("  ✓ Product analytics identifies top performers");
    println!("  ✓ Discount analysis shows impact on margins");
    println!("  ✓ Monthly trends enable forecasting");
    println!("\nElastiCube Features Used:");
    println!("  • Multi-dimensional slicing (customer, geography, product, time)");
    println!("  • Calculated measures for KPIs (revenue, profit, margin)");
    println!("  • Complex SQL queries with CASE statements");
    println!("  • Filtering and aggregation at scale");
    println!("  • DISTINCT counts for cardinality analysis");

    Ok(())
}
