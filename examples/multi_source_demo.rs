//! Multi-Source Data Loading Demo
//!
//! Demonstrates loading data from different sources:
//! - CSV files
//! - Parquet files
//! - JSON files
//! - PostgreSQL databases (with feature flag)
//! - MySQL databases (with feature flag)
//! - REST APIs (with feature flag)
//!
//! Run this example with:
//! ```bash
//! cargo run --example multi_source_demo --features all-sources
//! ```

use elasticube_core::{AggFunc, ElastiCubeBuilder, Result};
use arrow::datatypes::DataType;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== ElastiCube Multi-Source Loading Demo ===\n");

    // =========================================================================
    // Example 1: Loading from CSV
    // =========================================================================
    println!("📄 Example 1: Loading from CSV file");
    println!("-----------------------------------");

    // Note: This requires a sales.csv file to exist
    // Uncomment the following to test with your own CSV file:
    /*
    let csv_cube = ElastiCubeBuilder::new("csv_sales")
        .add_dimension("region", DataType::Utf8)?
        .add_dimension("product", DataType::Utf8)?
        .add_measure("revenue", DataType::Float64, AggFunc::Sum)?
        .add_measure("quantity", DataType::Int32, AggFunc::Sum)?
        .load_csv("sales.csv")
        .build()?;

    let results = csv_cube.query()
        .select(&["region", "SUM(revenue) as total_revenue"])
        .group_by(&["region"])
        .execute()
        .await?;

    println!("CSV Results:");
    println!("{}", results);
    */
    println!("✓ CSV loading supported via .load_csv(path)\n");

    // =========================================================================
    // Example 2: Loading from Parquet
    // =========================================================================
    println!("📦 Example 2: Loading from Parquet file");
    println!("---------------------------------------");

    // Note: This requires a sales.parquet file to exist
    // Uncomment the following to test with your own Parquet file:
    /*
    let parquet_cube = ElastiCubeBuilder::new("parquet_sales")
        .load_parquet("sales.parquet")
        .build()?;

    println!("Loaded {} rows from Parquet", parquet_cube.row_count());
    */
    println!("✓ Parquet loading supported via .load_parquet(path)\n");

    // =========================================================================
    // Example 3: Loading from JSON
    // =========================================================================
    println!("🔤 Example 3: Loading from JSON file");
    println!("------------------------------------");

    // Note: This requires a sales.json file to exist (newline-delimited JSON)
    // Uncomment the following to test with your own JSON file:
    /*
    let json_cube = ElastiCubeBuilder::new("json_sales")
        .load_json("sales.json")
        .build()?;

    println!("Loaded {} rows from JSON", json_cube.row_count());
    */
    println!("✓ JSON loading supported via .load_json(path)\n");

    // =========================================================================
    // Example 4: Loading from PostgreSQL (requires 'database' feature)
    // =========================================================================
    #[cfg(feature = "database")]
    {
        println!("🐘 Example 4: Loading from PostgreSQL");
        println!("--------------------------------------");

        // Note: This requires a PostgreSQL database to be running
        // Uncomment and configure the following to test:
        /*
        let pg_cube = ElastiCubeBuilder::new("postgres_sales")
            .load_postgres(
                "localhost",           // host
                "sales_db",            // database
                "postgres",            // username
                "password",            // password
                "SELECT * FROM sales WHERE date >= '2025-01-01'"  // query
            )
            .build()?;

        println!("Loaded {} rows from PostgreSQL", pg_cube.row_count());

        let results = pg_cube.query()
            .select(&["region", "SUM(revenue) as total"])
            .group_by(&["region"])
            .execute()
            .await?;

        println!("PostgreSQL Results:");
        println!("{}", results);
        */

        println!("✓ PostgreSQL loading supported via .load_postgres()");
        println!("  Connection string: Driver={{PostgreSQL Unicode}};Server=host;Database=db;...");
        println!();
    }

    #[cfg(not(feature = "database"))]
    {
        println!("🐘 Example 4: PostgreSQL (disabled)");
        println!("------------------------------------");
        println!("⚠  Enable with --features database");
        println!();
    }

    // =========================================================================
    // Example 5: Loading from MySQL (requires 'database' feature)
    // =========================================================================
    #[cfg(feature = "database")]
    {
        println!("🐬 Example 5: Loading from MySQL");
        println!("--------------------------------");

        // Note: This requires a MySQL database to be running
        // Uncomment and configure the following to test:
        /*
        let mysql_cube = ElastiCubeBuilder::new("mysql_sales")
            .load_mysql(
                "localhost",           // host
                "sales_db",            // database
                "root",                // username
                "password",            // password
                "SELECT * FROM orders WHERE year = 2025"  // query
            )
            .build()?;

        println!("Loaded {} rows from MySQL", mysql_cube.row_count());
        */

        println!("✓ MySQL loading supported via .load_mysql()");
        println!("  Connection string: Driver={{MySQL ODBC 8.0 Unicode Driver}};...");
        println!();
    }

    #[cfg(not(feature = "database"))]
    {
        println!("🐬 Example 5: MySQL (disabled)");
        println!("-------------------------------");
        println!("⚠  Enable with --features database");
        println!();
    }

    // =========================================================================
    // Example 6: Loading from REST API (requires 'rest-api' feature)
    // =========================================================================
    #[cfg(feature = "rest-api")]
    {
        println!("🌐 Example 6: Loading from REST API");
        println!("-----------------------------------");

        // Note: This requires a REST API that returns JSON data
        // Uncomment and configure the following to test:
        /*
        use elasticube_core::{RestApiSource, HttpMethod};

        let api_source = RestApiSource::new("https://api.example.com/sales")
            .with_method(HttpMethod::Get)
            .with_header("Authorization", "Bearer YOUR_TOKEN_HERE")
            .with_query_param("limit", "1000")
            .with_timeout_secs(60);

        let api_cube = ElastiCubeBuilder::new("api_sales")
            .load_rest_api_with(api_source)
            .build()?;

        println!("Loaded {} rows from REST API", api_cube.row_count());

        let results = api_cube.query()
            .select(&["*"])
            .limit(10)
            .execute()
            .await?;

        println!("API Results (first 10 rows):");
        println!("{}", results);
        */

        println!("✓ REST API loading supported via .load_rest_api()");
        println!("  Supports GET/POST with headers and query parameters");
        println!("  Response must be JSON (array of objects or single object)");
        println!();
    }

    #[cfg(not(feature = "rest-api"))]
    {
        println!("🌐 Example 6: REST API (disabled)");
        println!("----------------------------------");
        println!("⚠  Enable with --features rest-api");
        println!();
    }

    // =========================================================================
    // Example 7: Generic ODBC Connection (requires 'database' feature)
    // =========================================================================
    #[cfg(feature = "database")]
    {
        println!("🔌 Example 7: Generic ODBC Connection");
        println!("-------------------------------------");

        // Works with any ODBC-compatible database (SQLite, SQL Server, etc.)
        // Uncomment and configure the following to test:
        /*
        let odbc_cube = ElastiCubeBuilder::new("odbc_sales")
            .load_odbc(
                "Driver={SQL Server};Server=localhost;Database=SalesDB;Trusted_Connection=yes;",
                "SELECT * FROM transactions WHERE amount > 100"
            )
            .build()?;

        println!("Loaded {} rows via generic ODBC", odbc_cube.row_count());
        */

        println!("✓ Generic ODBC loading supported via .load_odbc()");
        println!("  Works with: SQL Server, SQLite, Oracle, DB2, etc.");
        println!();
    }

    #[cfg(not(feature = "database"))]
    {
        println!("🔌 Example 7: Generic ODBC (disabled)");
        println!("--------------------------------------");
        println!("⚠  Enable with --features database");
        println!();
    }

    // =========================================================================
    // Summary
    // =========================================================================
    println!("═══════════════════════════════════════");
    println!("Summary of Supported Data Sources:");
    println!("═══════════════════════════════════════");
    println!("✓ CSV files (built-in)");
    println!("✓ Parquet files (built-in)");
    println!("✓ JSON files (built-in)");
    println!("✓ Arrow RecordBatches (built-in)");

    #[cfg(feature = "database")]
    {
        println!("✓ PostgreSQL (via ODBC)");
        println!("✓ MySQL (via ODBC)");
        println!("✓ Generic ODBC databases");
    }
    #[cfg(not(feature = "database"))]
    {
        println!("⚠ PostgreSQL (enable with --features database)");
        println!("⚠ MySQL (enable with --features database)");
        println!("⚠ Generic ODBC databases (enable with --features database)");
    }

    #[cfg(feature = "rest-api")]
    println!("✓ REST APIs (JSON)");
    #[cfg(not(feature = "rest-api"))]
    println!("⚠ REST APIs (enable with --features rest-api)");

    println!();
    println!("Use --features all-sources to enable all optional sources!");

    Ok(())
}
