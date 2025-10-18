//! Integration tests for ElastiCube data update operations
//!
//! Tests append, delete, and update functionality

#[cfg(test)]
mod tests {
    use crate::{AggFunc, ElastiCubeBuilder};
    use arrow::array::{Float64Array, Int32Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use std::sync::Arc;

    /// Helper to create test cube with sales data
    fn create_test_cube() -> Arc<crate::ElastiCube> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("region", DataType::Utf8, false),
            Field::new("product", DataType::Utf8, false),
            Field::new("sales", DataType::Float64, false),
            Field::new("quantity", DataType::Int32, false),
        ]));

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(vec!["North", "South", "East", "West"])),
                Arc::new(StringArray::from(vec!["A", "B", "A", "C"])),
                Arc::new(Float64Array::from(vec![100.0, 200.0, 150.0, 300.0])),
                Arc::new(Int32Array::from(vec![10, 20, 15, 30])),
            ],
        )
        .unwrap();

        let cube = ElastiCubeBuilder::new("test_sales")
            .add_dimension("region", DataType::Utf8)
            .unwrap()
            .add_dimension("product", DataType::Utf8)
            .unwrap()
            .add_measure("sales", DataType::Float64, AggFunc::Sum)
            .unwrap()
            .add_measure("quantity", DataType::Int32, AggFunc::Sum)
            .unwrap()
            .with_data(vec![batch])
            .unwrap()
            .build()
            .unwrap();

        Arc::new(cube)
    }

    #[test]
    fn test_append_rows() {
        let mut cube = (*create_test_cube()).clone();
        assert_eq!(cube.row_count(), 4);

        // Create new data to append
        let schema = cube.arrow_schema().clone();
        let new_batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(vec!["Central", "North"])),
                Arc::new(StringArray::from(vec!["D", "B"])),
                Arc::new(Float64Array::from(vec![250.0, 180.0])),
                Arc::new(Int32Array::from(vec![25, 18])),
            ],
        )
        .unwrap();

        let rows_added = cube.append_rows(new_batch).unwrap();
        assert_eq!(rows_added, 2);
        assert_eq!(cube.row_count(), 6);
    }

    #[test]
    fn test_append_batches() {
        let mut cube = (*create_test_cube()).clone();
        assert_eq!(cube.row_count(), 4);

        let schema = cube.arrow_schema().clone();

        let batch1 = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(vec!["Central"])),
                Arc::new(StringArray::from(vec!["D"])),
                Arc::new(Float64Array::from(vec![250.0])),
                Arc::new(Int32Array::from(vec![25])),
            ],
        )
        .unwrap();

        let batch2 = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(vec!["Northeast", "Southwest"])),
                Arc::new(StringArray::from(vec!["E", "F"])),
                Arc::new(Float64Array::from(vec![175.0, 225.0])),
                Arc::new(Int32Array::from(vec![17, 22])),
            ],
        )
        .unwrap();

        let total_rows = cube.append_batches(vec![batch1, batch2]).unwrap();
        assert_eq!(total_rows, 3);
        assert_eq!(cube.row_count(), 7);
    }

    #[test]
    fn test_append_empty_batches_returns_zero() {
        let mut cube = (*create_test_cube()).clone();
        let original_count = cube.row_count();

        let result = cube.append_batches(vec![]).unwrap();
        assert_eq!(result, 0);
        assert_eq!(cube.row_count(), original_count);
    }

    #[test]
    fn test_append_with_incompatible_schema_fails() {
        let mut cube = (*create_test_cube()).clone();

        // Create batch with wrong schema
        let wrong_schema = Arc::new(Schema::new(vec![
            Field::new("wrong_field", DataType::Int32, false),
        ]));

        let bad_batch = RecordBatch::try_new(
            wrong_schema,
            vec![Arc::new(Int32Array::from(vec![1, 2]))],
        )
        .unwrap();

        let result = cube.append_rows(bad_batch);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("mismatch"));
    }

    #[tokio::test]
    async fn test_delete_rows() {
        let mut cube = (*create_test_cube()).clone();
        assert_eq!(cube.row_count(), 4);

        // Delete rows where sales < 200
        let deleted = cube.delete_rows("sales < 200").await.unwrap();
        assert_eq!(deleted, 2); // North (100) and East (150) should be deleted
        assert_eq!(cube.row_count(), 2); // South (200) and West (300) remain
    }

    #[tokio::test]
    async fn test_delete_rows_with_string_filter() {
        let mut cube = (*create_test_cube()).clone();
        assert_eq!(cube.row_count(), 4);

        // Delete rows where region = 'North'
        let deleted = cube.delete_rows("region = 'North'").await.unwrap();
        assert_eq!(deleted, 1);
        assert_eq!(cube.row_count(), 3);
    }

    #[tokio::test]
    async fn test_delete_rows_no_matches() {
        let mut cube = (*create_test_cube()).clone();
        let original_count = cube.row_count();

        // Delete with filter that matches nothing
        let deleted = cube.delete_rows("sales > 1000").await.unwrap();
        assert_eq!(deleted, 0);
        assert_eq!(cube.row_count(), original_count);
    }

    #[tokio::test]
    async fn test_delete_all_rows() {
        let mut cube = (*create_test_cube()).clone();

        // Delete all rows
        let deleted = cube.delete_rows("sales >= 0").await.unwrap();
        assert_eq!(deleted, 4);
        assert_eq!(cube.row_count(), 0);
    }

    #[tokio::test]
    async fn test_update_rows() {
        let mut cube = (*create_test_cube()).clone();
        assert_eq!(cube.row_count(), 4);

        // Create replacement data for North region
        let schema = cube.arrow_schema().clone();
        let replacement = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(vec!["North"])),
                Arc::new(StringArray::from(vec!["A"])),
                Arc::new(Float64Array::from(vec![500.0])), // Updated sales
                Arc::new(Int32Array::from(vec![50])),      // Updated quantity
            ],
        )
        .unwrap();

        let (deleted, added) = cube.update_rows("region = 'North'", replacement).await.unwrap();
        assert_eq!(deleted, 1);
        assert_eq!(added, 1);
        assert_eq!(cube.row_count(), 4); // Same count, but data updated

        // Verify the update by querying
        let result = Arc::new(cube.clone())
            .query()
            .unwrap()
            .select(&["region", "sales"])
            .filter("region = 'North'")
            .execute()
            .await
            .unwrap();

        let batches = result.batches();
        assert_eq!(batches.len(), 1);

        let sales_col = batches[0]
            .column_by_name("sales")
            .unwrap()
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap();

        assert_eq!(sales_col.value(0), 500.0); // Verify updated value
    }

    #[tokio::test]
    async fn test_update_multiple_rows() {
        let mut cube = (*create_test_cube()).clone();

        // Create replacement data for product 'A' (2 rows: North and East)
        let schema = cube.arrow_schema().clone();
        let replacement = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(vec!["North", "East"])),
                Arc::new(StringArray::from(vec!["A", "A"])),
                Arc::new(Float64Array::from(vec![600.0, 700.0])),
                Arc::new(Int32Array::from(vec![60, 70])),
            ],
        )
        .unwrap();

        let (deleted, added) = cube.update_rows("product = 'A'", replacement).await.unwrap();
        assert_eq!(deleted, 2); // North and East both have product A
        assert_eq!(added, 2);
        assert_eq!(cube.row_count(), 4);
    }

    #[test]
    fn test_consolidate_batches() {
        let mut cube = (*create_test_cube()).clone();

        // Append more batches to fragment the data
        let schema = cube.arrow_schema().clone();
        let batch1 = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(vec!["Central"])),
                Arc::new(StringArray::from(vec!["D"])),
                Arc::new(Float64Array::from(vec![250.0])),
                Arc::new(Int32Array::from(vec![25])),
            ],
        )
        .unwrap();

        let batch2 = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(vec!["Northeast"])),
                Arc::new(StringArray::from(vec!["E"])),
                Arc::new(Float64Array::from(vec![175.0])),
                Arc::new(Int32Array::from(vec![17])),
            ],
        )
        .unwrap();

        cube.append_batches(vec![batch1, batch2]).unwrap();

        // We should have 3 batches now (1 original + 2 appended)
        assert_eq!(cube.batch_count(), 3);
        assert_eq!(cube.row_count(), 6);

        // Consolidate
        let old_count = cube.consolidate_batches().unwrap();
        assert_eq!(old_count, 3);
        assert_eq!(cube.batch_count(), 1);
        assert_eq!(cube.row_count(), 6); // Row count unchanged
    }

    #[test]
    fn test_consolidate_single_batch_no_op() {
        let mut cube = (*create_test_cube()).clone();

        // Initial cube has 1 batch
        assert_eq!(cube.batch_count(), 1);

        let old_count = cube.consolidate_batches().unwrap();
        assert_eq!(old_count, 1);
        assert_eq!(cube.batch_count(), 1);
    }

    #[test]
    fn test_batch_count() {
        let cube = (*create_test_cube()).clone();
        assert_eq!(cube.batch_count(), 1);
    }

    #[tokio::test]
    async fn test_sequential_operations() {
        let mut cube = (*create_test_cube()).clone();
        let schema = cube.arrow_schema().clone();

        // 1. Append new rows
        let new_batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(vec!["Central"])),
                Arc::new(StringArray::from(vec!["D"])),
                Arc::new(Float64Array::from(vec![250.0])),
                Arc::new(Int32Array::from(vec![25])),
            ],
        )
        .unwrap();
        cube.append_rows(new_batch).unwrap();
        assert_eq!(cube.row_count(), 5);

        // 2. Delete some rows
        let deleted = cube.delete_rows("sales < 150").await.unwrap();
        assert_eq!(deleted, 1); // Only North (100) is < 150
        assert_eq!(cube.row_count(), 4);

        // 3. Update a row
        let update_batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(vec!["South"])),
                Arc::new(StringArray::from(vec!["B"])),
                Arc::new(Float64Array::from(vec![999.0])),
                Arc::new(Int32Array::from(vec![99])),
            ],
        )
        .unwrap();
        let (deleted, added) = cube.update_rows("region = 'South'", update_batch).await.unwrap();
        assert_eq!(deleted, 1);
        assert_eq!(added, 1);
        assert_eq!(cube.row_count(), 4);

        // 4. Consolidate batches
        cube.consolidate_batches().unwrap();
        assert_eq!(cube.batch_count(), 1);
        assert_eq!(cube.row_count(), 4);
    }
}
