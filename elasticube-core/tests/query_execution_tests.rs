//! Integration tests for query execution
//!
//! These tests verify the end-to-end query execution pipeline, including:
//! - Data loading from various sources
//! - Query building and execution
//! - Result correctness
//! - OLAP operations (slice, dice, drill-down, roll-up)

use arrow_array::{Float64Array, Int64Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema as ArrowSchema};
use elasticube_core::{AggFunc, ElastiCubeBuilder};
use std::sync::Arc;
use tempfile::NamedTempFile;
use std::io::Write;

/// Helper function to create a test sales dataset
fn create_test_sales_data() -> RecordBatch {
    let schema = Arc::new(ArrowSchema::new(vec![
        Field::new("date", DataType::Utf8, false),
        Field::new("region", DataType::Utf8, false),
        Field::new("product", DataType::Utf8, false),
        Field::new("sales", DataType::Float64, false),
        Field::new("quantity", DataType::Int64, false),
    ]));

    let dates = StringArray::from(vec![
        "2024-01-01",
        "2024-01-01",
        "2024-01-02",
        "2024-01-02",
        "2024-01-03",
        "2024-01-03",
    ]);

    let regions = StringArray::from(vec![
        "North",
        "South",
        "North",
        "South",
        "North",
        "South",
    ]);

    let products = StringArray::from(vec![
        "Widget",
        "Gadget",
        "Widget",
        "Gadget",
        "Widget",
        "Gadget",
    ]);

    let sales = Float64Array::from(vec![100.0, 150.0, 200.0, 175.0, 120.0, 160.0]);
    let quantity = Int64Array::from(vec![10, 15, 20, 17, 12, 16]);

    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(dates),
            Arc::new(regions),
            Arc::new(products),
            Arc::new(sales),
            Arc::new(quantity),
        ],
    )
    .unwrap()
}

/// Helper function to write test data to a CSV file
fn create_test_csv() -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "date,region,product,sales,quantity").unwrap();
    writeln!(file, "2024-01-01,North,Widget,100.0,10").unwrap();
    writeln!(file, "2024-01-01,South,Gadget,150.0,15").unwrap();
    writeln!(file, "2024-01-02,North,Widget,200.0,20").unwrap();
    writeln!(file, "2024-01-02,South,Gadget,175.0,17").unwrap();
    writeln!(file, "2024-01-03,North,Widget,120.0,12").unwrap();
    writeln!(file, "2024-01-03,South,Gadget,160.0,16").unwrap();
    file.flush().unwrap();
    file
}

#[tokio::test]
async fn test_basic_query_execution() {
    let batch = create_test_sales_data();

    let cube = Arc::new(ElastiCubeBuilder::new("test_cube")
        .add_dimension("date", DataType::Utf8).unwrap()
        .add_dimension("region", DataType::Utf8).unwrap()
        .add_dimension("product", DataType::Utf8).unwrap()
        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
        .add_measure("quantity", DataType::Int64, AggFunc::Sum).unwrap()
        .with_data(vec![batch]).unwrap()
        .build()
        .unwrap());

    // Basic SELECT query
    let result = cube.clone()
        .query().unwrap()
        .select(&["region", "sum(sales)"])
        .group_by(&["region"])
        .execute()
        .await
        .unwrap();

    assert!(result.row_count() > 0);
    assert!(result.batches().first().map(|b| b.num_columns()).unwrap_or(0) >= 2);
}

#[tokio::test]
async fn test_filter_query() {
    let batch = create_test_sales_data();

    let cube = Arc::new(ElastiCubeBuilder::new("test_cube")
        .add_dimension("date", DataType::Utf8).unwrap()
        .add_dimension("region", DataType::Utf8).unwrap()
        .add_dimension("product", DataType::Utf8).unwrap()
        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
        .add_measure("quantity", DataType::Int64, AggFunc::Sum).unwrap()
        .with_data(vec![batch]).unwrap()
        .build()
        .unwrap());

    // Query with filter
    let result = cube.clone()
        .query().unwrap()
        .select(&["region", "sum(sales)"])
        .filter("region = 'North'")
        .group_by(&["region"])
        .execute()
        .await
        .unwrap();

    assert_eq!(result.row_count(), 1);
}

#[tokio::test]
async fn test_aggregation_functions() {
    let batch = create_test_sales_data();

    let cube = Arc::new(ElastiCubeBuilder::new("test_cube")
        .add_dimension("region", DataType::Utf8).unwrap()
        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
        .add_measure("quantity", DataType::Int64, AggFunc::Sum).unwrap()
        .with_data(vec![batch]).unwrap()
        .build()
        .unwrap());

    // Test multiple aggregation functions
    let result = cube.clone()
        .query().unwrap()
        .select(&[
            "region",
            "sum(sales) as total_sales",
            "avg(sales) as avg_sales",
            "min(sales) as min_sales",
            "max(sales) as max_sales",
            "count(sales) as count_sales",
        ])
        .group_by(&["region"])
        .execute()
        .await
        .unwrap();

    assert!(result.row_count() > 0);
    assert_eq!(result.batches().first().map(|b| b.num_columns()).unwrap_or(0), 6); // region + 5 aggregations
}

#[tokio::test]
async fn test_order_by_query() {
    let batch = create_test_sales_data();

    let cube = Arc::new(ElastiCubeBuilder::new("test_cube")
        .add_dimension("region", DataType::Utf8).unwrap()
        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
        .with_data(vec![batch]).unwrap()
        .build()
        .unwrap());

    // Query with ORDER BY
    let result = cube.clone()
        .query().unwrap()
        .select(&["region", "sum(sales) as total"])
        .group_by(&["region"])
        .order_by(&["total DESC"])
        .execute()
        .await
        .unwrap();

    assert!(result.row_count() > 0);
}

#[tokio::test]
async fn test_limit_and_offset() {
    let batch = create_test_sales_data();

    let cube = Arc::new(ElastiCubeBuilder::new("test_cube")
        .add_dimension("date", DataType::Utf8).unwrap()
        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
        .with_data(vec![batch]).unwrap()
        .build()
        .unwrap());

    // Query with LIMIT
    let result = cube.clone()
        .query().unwrap()
        .select(&["date", "sum(sales)"])
        .group_by(&["date"])
        .limit(2)
        .execute()
        .await
        .unwrap();

    assert!(result.row_count() <= 2);

    // Query with LIMIT and OFFSET
    let result_offset = cube.clone()
        .query().unwrap()
        .select(&["date", "sum(sales)"])
        .group_by(&["date"])
        .limit(1)
        .offset(1)
        .execute()
        .await
        .unwrap();

    assert!(result_offset.row_count() <= 1);
}

#[tokio::test]
async fn test_olap_slice() {
    let batch = create_test_sales_data();

    let cube = Arc::new(ElastiCubeBuilder::new("test_cube")
        .add_dimension("date", DataType::Utf8).unwrap()
        .add_dimension("region", DataType::Utf8).unwrap()
        .add_dimension("product", DataType::Utf8).unwrap()
        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
        .with_data(vec![batch]).unwrap()
        .build()
        .unwrap());

    // Slice by region
    let result = cube.clone()
        .query().unwrap()
        .slice("region", "North")
        .select(&["date", "product", "sum(sales)"])
        .group_by(&["date", "product"])
        .execute()
        .await
        .unwrap();

    assert!(result.row_count() > 0);
}

#[tokio::test]
async fn test_olap_dice() {
    let batch = create_test_sales_data();

    let cube = Arc::new(ElastiCubeBuilder::new("test_cube")
        .add_dimension("date", DataType::Utf8).unwrap()
        .add_dimension("region", DataType::Utf8).unwrap()
        .add_dimension("product", DataType::Utf8).unwrap()
        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
        .with_data(vec![batch]).unwrap()
        .build()
        .unwrap());

    // Dice by multiple dimensions
    let result = cube.clone()
        .query().unwrap()
        .dice(&[
            ("region", "North"),
            ("product", "Widget"),
        ])
        .select(&["date", "sum(sales)"])
        .group_by(&["date"])
        .execute()
        .await
        .unwrap();

    assert!(result.row_count() > 0);
}

#[tokio::test]
async fn test_olap_roll_up() {
    let batch = create_test_sales_data();

    let cube = Arc::new(ElastiCubeBuilder::new("test_cube")
        .add_dimension("date", DataType::Utf8).unwrap()
        .add_dimension("region", DataType::Utf8).unwrap()
        .add_dimension("product", DataType::Utf8).unwrap()
        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
        .with_data(vec![batch]).unwrap()
        .build()
        .unwrap());

    // Roll-up to region level (removing product dimension)
    let result = cube.clone()
        .query().unwrap()
        .roll_up(&["region"])
        .select(&["region", "sum(sales)"])
        .group_by(&["region"])
        .execute()
        .await
        .unwrap();

    assert!(result.row_count() > 0);
}

#[tokio::test]
async fn test_complex_query_with_multiple_operations() {
    let batch = create_test_sales_data();

    let cube = Arc::new(ElastiCubeBuilder::new("test_cube")
        .add_dimension("date", DataType::Utf8).unwrap()
        .add_dimension("region", DataType::Utf8).unwrap()
        .add_dimension("product", DataType::Utf8).unwrap()
        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
        .add_measure("quantity", DataType::Int64, AggFunc::Sum).unwrap()
        .with_data(vec![batch]).unwrap()
        .build()
        .unwrap());

    // Complex query: filter, group, order, limit
    let result = cube.clone()
        .query().unwrap()
        .select(&[
            "region",
            "product",
            "sum(sales) as total_sales",
            "sum(quantity) as total_qty",
        ])
        .filter("date >= '2024-01-02'")
        .group_by(&["region", "product"])
        .order_by(&["total_sales DESC"])
        .limit(3)
        .execute()
        .await
        .unwrap();

    assert!(result.row_count() > 0);
    assert!(result.row_count() <= 3);
}

#[tokio::test]
async fn test_csv_data_loading_and_query() {
    let csv_file = create_test_csv();

    // Note: CSV loader infers date as Date32, so we don't pre-define schema
    let cube = Arc::new(ElastiCubeBuilder::new("test_cube")
        .load_csv(csv_file.path().to_str().unwrap())
        .build()
        .unwrap());

    let result = cube.clone()
        .query().unwrap()
        .select(&["region", "sum(sales)"])
        .group_by(&["region"])
        .execute()
        .await
        .unwrap();

    assert!(result.row_count() > 0);
}

#[tokio::test]
async fn test_sql_query_execution() {
    let batch = create_test_sales_data();

    let cube = Arc::new(ElastiCubeBuilder::new("test_cube")
        .add_dimension("region", DataType::Utf8).unwrap()
        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
        .with_data(vec![batch]).unwrap()
        .build()
        .unwrap());

    // Direct SQL query
    let result = cube.clone()
        .query().unwrap()
        .sql("SELECT region, SUM(sales) as total FROM cube GROUP BY region")
        .execute()
        .await
        .unwrap();

    assert!(result.row_count() > 0);
}

#[tokio::test]
async fn test_empty_result_set() {
    let batch = create_test_sales_data();

    let cube = Arc::new(ElastiCubeBuilder::new("test_cube")
        .add_dimension("region", DataType::Utf8).unwrap()
        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
        .with_data(vec![batch]).unwrap()
        .build()
        .unwrap());

    // Query that returns no results
    let result = cube.clone()
        .query().unwrap()
        .select(&["region", "sum(sales)"])
        .filter("region = 'NonExistent'")
        .group_by(&["region"])
        .execute()
        .await
        .unwrap();

    assert_eq!(result.row_count(), 0);
}

#[tokio::test]
async fn test_count_distinct() {
    let batch = create_test_sales_data();

    let cube = Arc::new(ElastiCubeBuilder::new("test_cube")
        .add_dimension("region", DataType::Utf8).unwrap()
        .add_dimension("product", DataType::Utf8).unwrap()
        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
        .with_data(vec![batch]).unwrap()
        .build()
        .unwrap());

    // COUNT DISTINCT query
    let result = cube.clone()
        .query().unwrap()
        .sql("SELECT region, COUNT(DISTINCT product) as unique_products FROM cube GROUP BY region")
        .execute()
        .await
        .unwrap();

    assert!(result.row_count() > 0);
}
