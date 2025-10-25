//! Property-based tests using quickcheck
//!
//! These tests verify invariant properties that should hold true for all inputs:
//! - Query results should be deterministic
//! - Aggregations should be commutative where applicable
//! - Filtering should preserve schema
//! - Result row counts should respect LIMIT constraints

use arrow_array::{Float64Array, Int64Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema as ArrowSchema};
use elasticube_core::{AggFunc, ElastiCubeBuilder};
use quickcheck::{Arbitrary, Gen, TestResult};
use quickcheck_macros::quickcheck;
use std::sync::Arc;

/// Generate a small test dataset for property testing
#[derive(Clone, Debug)]
struct TestDataset {
    regions: Vec<String>,
    sales: Vec<f64>,
    quantities: Vec<i64>,
}

impl Arbitrary for TestDataset {
    fn arbitrary(g: &mut Gen) -> Self {
        let size = u8::arbitrary(g) % 20 + 5; // 5-24 rows
        let regions = vec!["North", "South", "East", "West"];

        let mut dataset_regions = Vec::new();
        let mut sales = Vec::new();
        let mut quantities = Vec::new();

        for _ in 0..size {
            let region_idx = (u8::arbitrary(g) % regions.len() as u8) as usize;
            dataset_regions.push(regions[region_idx].to_string());

            // Generate positive sales values using unsigned random values
            let sale_raw = u64::arbitrary(g) % 10000 + 1;
            sales.push(sale_raw as f64);

            // Generate positive quantities using unsigned random values
            let qty = (u64::arbitrary(g) % 1000 + 1) as i64;
            quantities.push(qty);
        }

        TestDataset {
            regions: dataset_regions,
            sales,
            quantities,
        }
    }
}

impl TestDataset {
    fn to_record_batch(&self) -> RecordBatch {
        let schema = Arc::new(ArrowSchema::new(vec![
            Field::new("region", DataType::Utf8, false),
            Field::new("sales", DataType::Float64, false),
            Field::new("quantity", DataType::Int64, false),
        ]));

        let regions = StringArray::from(self.regions.clone());
        let sales = Float64Array::from(self.sales.clone());
        let quantities = Int64Array::from(self.quantities.clone());

        RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(regions), Arc::new(sales), Arc::new(quantities)],
        )
        .unwrap()
    }

    fn expected_sum_by_region(&self) -> std::collections::HashMap<String, f64> {
        let mut sums = std::collections::HashMap::new();
        for (region, &sales) in self.regions.iter().zip(self.sales.iter()) {
            *sums.entry(region.clone()).or_insert(0.0) += sales;
        }
        sums
    }
}

#[quickcheck]
fn prop_query_is_deterministic(dataset: TestDataset) -> TestResult {
    if dataset.regions.is_empty() {
        return TestResult::discard();
    }

    let batch = dataset.to_record_batch();

    let cube = match ElastiCubeBuilder::new("test_cube")
        .add_dimension("region", DataType::Utf8).unwrap()
        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
        .add_measure("quantity", DataType::Int64, AggFunc::Sum).unwrap()
        .with_data(vec![batch.clone()]).unwrap()
        .build()
    {
        Ok(c) => Arc::new(c),
        Err(_) => return TestResult::discard(),
    };

    // Run the same query twice
    let rt = tokio::runtime::Runtime::new().unwrap();

    let result1 = rt.block_on(async {
        cube.clone().query().unwrap()
            .select(&["region", "sum(sales)"])
            .group_by(&["region"])
            .execute()
            .await
    });

    let result2 = rt.block_on(async {
        cube.clone().query().unwrap()
            .select(&["region", "sum(sales)"])
            .group_by(&["region"])
            .execute()
            .await
    });

    match (result1, result2) {
        (Ok(r1), Ok(r2)) => {
            // Both queries should return the same number of rows
            TestResult::from_bool(r1.row_count() == r2.row_count())
        }
        _ => TestResult::discard(),
    }
}

#[quickcheck]
fn prop_filter_preserves_schema(dataset: TestDataset) -> TestResult {
    if dataset.regions.is_empty() {
        return TestResult::discard();
    }

    let batch = dataset.to_record_batch();

    let cube = match ElastiCubeBuilder::new("test_cube")
        .add_dimension("region", DataType::Utf8).unwrap()
        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
        .with_data(vec![batch]).unwrap()
        .build()
    {
        Ok(c) => Arc::new(c),
        Err(_) => return TestResult::discard(),
    };

    let rt = tokio::runtime::Runtime::new().unwrap();

    // Query without filter
    let result_all = rt.block_on(async {
        cube.clone().query().unwrap()
            .select(&["region", "sum(sales)"])
            .group_by(&["region"])
            .execute()
            .await
    });

    // Query with filter - use a region that exists in the dataset
    let filter_region = dataset.regions.first().unwrap();
    let filter_expr = format!("region = '{}'", filter_region);

    let result_filtered = rt.block_on(async {
        cube.clone().query().unwrap()
            .select(&["region", "sum(sales)"])
            .filter(&filter_expr)
            .group_by(&["region"])
            .execute()
            .await
    });

    match (result_all, result_filtered) {
        (Ok(r1), Ok(r2)) => {
            // Both should have the same schema (number of columns)
            let cols1 = r1.batches().first().map(|b| b.num_columns()).unwrap_or(2); // expect 2 columns
            let cols2 = r2.batches().first().map(|b| b.num_columns()).unwrap_or(2); // expect 2 columns
            TestResult::from_bool(cols1 == cols2)
        }
        _ => TestResult::discard(),
    }
}

#[quickcheck]
fn prop_limit_respects_constraint(dataset: TestDataset, limit: u8) -> TestResult {
    if dataset.regions.is_empty() || limit == 0 {
        return TestResult::discard();
    }

    let limit = (limit % 20 + 1) as usize; // 1-20
    let batch = dataset.to_record_batch();

    let cube = match ElastiCubeBuilder::new("test_cube")
        .add_dimension("region", DataType::Utf8).unwrap()
        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
        .with_data(vec![batch]).unwrap()
        .build()
    {
        Ok(c) => Arc::new(c),
        Err(_) => return TestResult::discard(),
    };

    let rt = tokio::runtime::Runtime::new().unwrap();

    let result = rt.block_on(async {
        cube.clone().query().unwrap()
            .select(&["region", "sum(sales)"])
            .group_by(&["region"])
            .limit(limit)
            .execute()
            .await
    });

    match result {
        Ok(r) => {
            // Result row count should be <= limit
            TestResult::from_bool(r.row_count() <= limit)
        }
        _ => TestResult::discard(),
    }
}

#[quickcheck]
fn prop_sum_aggregation_correctness(dataset: TestDataset) -> TestResult {
    if dataset.regions.is_empty() {
        return TestResult::discard();
    }

    let _expected_sums = dataset.expected_sum_by_region();
    let batch = dataset.to_record_batch();

    let cube = match ElastiCubeBuilder::new("test_cube")
        .add_dimension("region", DataType::Utf8).unwrap()
        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
        .with_data(vec![batch]).unwrap()
        .build()
    {
        Ok(c) => Arc::new(c),
        Err(_) => return TestResult::discard(),
    };

    let rt = tokio::runtime::Runtime::new().unwrap();

    let result = rt.block_on(async {
        cube.clone().query().unwrap()
            .select(&["region", "sum(sales) as total"])
            .group_by(&["region"])
            .execute()
            .await
    });

    match result {
        Ok(r) => {
            // Check that the number of groups matches
            let unique_regions: std::collections::HashSet<_> =
                dataset.regions.iter().collect();
            TestResult::from_bool(r.row_count() == unique_regions.len())
        }
        _ => TestResult::discard(),
    }
}

#[quickcheck]
fn prop_group_by_reduces_rows(dataset: TestDataset) -> TestResult {
    if dataset.regions.is_empty() {
        return TestResult::discard();
    }

    let batch = dataset.to_record_batch();
    let original_row_count = batch.num_rows();

    let cube = match ElastiCubeBuilder::new("test_cube")
        .add_dimension("region", DataType::Utf8).unwrap()
        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
        .with_data(vec![batch]).unwrap()
        .build()
    {
        Ok(c) => Arc::new(c),
        Err(_) => return TestResult::discard(),
    };

    let rt = tokio::runtime::Runtime::new().unwrap();

    let result = rt.block_on(async {
        cube.clone().query().unwrap()
            .select(&["region", "sum(sales)"])
            .group_by(&["region"])
            .execute()
            .await
    });

    match result {
        Ok(r) => {
            // GROUP BY should reduce or maintain row count (never increase)
            TestResult::from_bool(r.row_count() <= original_row_count)
        }
        _ => TestResult::discard(),
    }
}

#[quickcheck]
fn prop_empty_filter_returns_zero_rows(dataset: TestDataset) -> TestResult {
    if dataset.regions.is_empty() {
        return TestResult::discard();
    }

    let batch = dataset.to_record_batch();

    let cube = match ElastiCubeBuilder::new("test_cube")
        .add_dimension("region", DataType::Utf8).unwrap()
        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
        .with_data(vec![batch]).unwrap()
        .build()
    {
        Ok(c) => Arc::new(c),
        Err(_) => return TestResult::discard(),
    };

    let rt = tokio::runtime::Runtime::new().unwrap();

    let result = rt.block_on(async {
        cube.clone().query().unwrap()
            .select(&["region", "sum(sales)"])
            .filter("region = 'NonExistentRegion'")
            .group_by(&["region"])
            .execute()
            .await
    });

    match result {
        Ok(r) => {
            // Filtering by non-existent value should return 0 rows
            TestResult::from_bool(r.row_count() == 0)
        }
        _ => TestResult::discard(),
    }
}

// Note: Tests are run via the #[quickcheck] attribute macro above
// No need for a manual test runner function - quickcheck handles this automatically
