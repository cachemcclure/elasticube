//! Tests for query materialization of calculated fields
//!
//! These tests verify that calculated measures and virtual dimensions
//! are properly expanded in SQL queries.

#[cfg(test)]
mod tests {
    use crate::{AggFunc, ElastiCubeBuilder};
    use arrow::array::{Float64Array, Int32Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use std::sync::Arc;

    /// Helper to create test data with sales information
    fn create_test_data() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("region", DataType::Utf8, false),
            Field::new("revenue", DataType::Float64, false),
            Field::new("cost", DataType::Float64, false),
            Field::new("quantity", DataType::Int32, false),
        ]));

        let region = Arc::new(StringArray::from(vec!["North", "South", "East", "West"]));
        let revenue = Arc::new(Float64Array::from(vec![1000.0, 1500.0, 1200.0, 1800.0]));
        let cost = Arc::new(Float64Array::from(vec![600.0, 900.0, 700.0, 1100.0]));
        let quantity = Arc::new(Int32Array::from(vec![10, 15, 12, 18]));

        RecordBatch::try_new(
            schema,
            vec![region, revenue, cost, quantity],
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_calculated_measure_in_select() {
        // Build cube with calculated measure
        let batch = create_test_data();
        let cube = Arc::new(
            ElastiCubeBuilder::new("sales")
                .add_dimension("region", DataType::Utf8)
                .unwrap()
                .add_measure("revenue", DataType::Float64, AggFunc::Sum)
                .unwrap()
                .add_measure("cost", DataType::Float64, AggFunc::Sum)
                .unwrap()
                .add_calculated_measure(
                    "profit",
                    "revenue - cost",
                    DataType::Float64,
                    AggFunc::Sum,
                )
                .unwrap()
                .with_data(vec![batch])
                .unwrap()
                .build()
                .unwrap(),
        );

        // Query using the calculated measure
        let query_builder = cube.clone().query().unwrap();
        let sql = query_builder.select(&["region", "SUM(profit) as total_profit"])
            .group_by(&["region"]);

        // Get the built SQL (we need to access this via a test method)
        // For now, we'll just execute and verify it works
        let result = sql.execute().await.unwrap();

        assert!(result.row_count() > 0, "Should have results");
        // Verify the calculated measure was expanded correctly
        // profit = revenue - cost should work in the query
    }

    #[tokio::test]
    async fn test_virtual_dimension_in_select() {
        // Create test data with dates
        use arrow::array::Date32Array;

        let schema = Arc::new(Schema::new(vec![
            Field::new("sale_date", DataType::Date32, false),
            Field::new("amount", DataType::Float64, false),
        ]));

        // Create some date values (days since epoch)
        let dates = Arc::new(Date32Array::from(vec![
            19000, // Year 2022
            19100, // Year 2022
            19365, // Year 2023
            19500, // Year 2023
        ]));
        let amounts = Arc::new(Float64Array::from(vec![100.0, 150.0, 200.0, 250.0]));

        let batch =
            RecordBatch::try_new(schema, vec![dates, amounts]).unwrap();

        let cube = Arc::new(
            ElastiCubeBuilder::new("sales")
                .add_dimension("sale_date", DataType::Date32)
                .unwrap()
                .add_measure("amount", DataType::Float64, AggFunc::Sum)
                .unwrap()
                .add_virtual_dimension(
                    "year",
                    "EXTRACT(YEAR FROM sale_date)",
                    DataType::Int32,
                )
                .unwrap()
                .with_data(vec![batch])
                .unwrap()
                .build()
                .unwrap(),
        );

        // Query using the virtual dimension
        let result = cube
            .clone()
            .query()
            .unwrap()
            .select(&["year", "SUM(amount) as total"])
            .group_by(&["year"])
            .execute()
            .await
            .unwrap();

        assert!(result.row_count() > 0, "Should have results");
    }

    #[tokio::test]
    async fn test_calculated_measure_in_filter() {
        let batch = create_test_data();
        let cube = Arc::new(
            ElastiCubeBuilder::new("sales")
                .add_dimension("region", DataType::Utf8)
                .unwrap()
                .add_measure("revenue", DataType::Float64, AggFunc::Sum)
                .unwrap()
                .add_measure("cost", DataType::Float64, AggFunc::Sum)
                .unwrap()
                .add_calculated_measure(
                    "profit",
                    "revenue - cost",
                    DataType::Float64,
                    AggFunc::Sum,
                )
                .unwrap()
                .with_data(vec![batch])
                .unwrap()
                .build()
                .unwrap(),
        );

        // Filter using calculated measure
        // Test data: North(400), South(600), East(500), West(700)
        let result = cube
            .clone()
            .query()
            .unwrap()
            .select(&["region", "profit"])
            .filter("profit > 550")  // Should expand to: (revenue - cost) > 550
            .execute()
            .await
            .unwrap();

        // Should filter to regions where profit > 550 (South=600, West=700)
        assert!(result.row_count() > 0, "Should have results");
        assert!(result.row_count() == 2, "Should have exactly 2 rows (South and West)");
    }

    #[tokio::test]
    async fn test_nested_calculated_measures() {
        let batch = create_test_data();
        let cube = Arc::new(
            ElastiCubeBuilder::new("sales")
                .add_dimension("region", DataType::Utf8)
                .unwrap()
                .add_measure("revenue", DataType::Float64, AggFunc::Sum)
                .unwrap()
                .add_measure("cost", DataType::Float64, AggFunc::Sum)
                .unwrap()
                .add_calculated_measure(
                    "profit",
                    "revenue - cost",
                    DataType::Float64,
                    AggFunc::Sum,
                )
                .unwrap()
                .add_calculated_measure(
                    "margin",
                    "(profit / revenue) * 100",
                    DataType::Float64,
                    AggFunc::Avg,
                )
                .unwrap()
                .with_data(vec![batch])
                .unwrap()
                .build()
                .unwrap(),
        );

        // Query using nested calculated measure
        // margin references profit, which references revenue and cost
        let result = cube
            .clone()
            .query()
            .unwrap()
            .select(&["region", "margin"])
            .execute()
            .await
            .unwrap();

        assert_eq!(result.row_count(), 4, "Should have all rows");
    }

    #[tokio::test]
    async fn test_multiple_calculated_fields_in_query() {
        let batch = create_test_data();
        let cube = Arc::new(
            ElastiCubeBuilder::new("sales")
                .add_dimension("region", DataType::Utf8)
                .unwrap()
                .add_measure("revenue", DataType::Float64, AggFunc::Sum)
                .unwrap()
                .add_measure("cost", DataType::Float64, AggFunc::Sum)
                .unwrap()
                .add_measure("quantity", DataType::Int32, AggFunc::Sum)
                .unwrap()
                .add_calculated_measure(
                    "profit",
                    "revenue - cost",
                    DataType::Float64,
                    AggFunc::Sum,
                )
                .unwrap()
                .add_calculated_measure(
                    "avg_unit_price",
                    "revenue / quantity",
                    DataType::Float64,
                    AggFunc::Avg,
                )
                .unwrap()
                .with_data(vec![batch])
                .unwrap()
                .build()
                .unwrap(),
        );

        // Use multiple calculated measures in one query
        let result = cube
            .clone()
            .query()
            .unwrap()
            .select(&[
                "region",
                "SUM(profit) as total_profit",
                "AVG(avg_unit_price) as avg_price",
            ])
            .group_by(&["region"])
            .order_by(&["total_profit DESC"])
            .execute()
            .await
            .unwrap();

        assert_eq!(result.row_count(), 4, "Should have all regions");
    }
}
