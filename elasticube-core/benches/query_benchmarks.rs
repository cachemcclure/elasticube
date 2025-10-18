//! Query performance benchmarks using Criterion
//!
//! Benchmarks various query operations to track performance over time:
//! - Simple aggregations
//! - Complex queries with filters
//! - OLAP operations
//! - Different dataset sizes

use arrow_array::{Float64Array, Int64Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema as ArrowSchema};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use elasticube_core::{AggFunc, ElastiCubeBuilder};
use std::sync::Arc;

/// Generate a test dataset with specified number of rows
fn generate_test_data(num_rows: usize) -> RecordBatch {
    let schema = Arc::new(ArrowSchema::new(vec![
        Field::new("date", DataType::Utf8, false),
        Field::new("region", DataType::Utf8, false),
        Field::new("product", DataType::Utf8, false),
        Field::new("category", DataType::Utf8, false),
        Field::new("sales", DataType::Float64, false),
        Field::new("quantity", DataType::Int64, false),
    ]));

    let regions = vec!["North", "South", "East", "West"];
    let products = vec!["Widget", "Gadget", "Doohickey", "Thingamajig"];
    let categories = vec!["Electronics", "Furniture", "Clothing", "Food"];

    let mut dates = Vec::with_capacity(num_rows);
    let mut region_vec = Vec::with_capacity(num_rows);
    let mut product_vec = Vec::with_capacity(num_rows);
    let mut category_vec = Vec::with_capacity(num_rows);
    let mut sales = Vec::with_capacity(num_rows);
    let mut quantities = Vec::with_capacity(num_rows);

    for i in 0..num_rows {
        dates.push(format!("2024-01-{:02}", (i % 28) + 1));
        region_vec.push(regions[i % regions.len()].to_string());
        product_vec.push(products[i % products.len()].to_string());
        category_vec.push(categories[i % categories.len()].to_string());
        sales.push(100.0 + (i as f64 * 0.5));
        quantities.push((10 + (i % 50)) as i64);
    }

    RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(dates)),
            Arc::new(StringArray::from(region_vec)),
            Arc::new(StringArray::from(product_vec)),
            Arc::new(StringArray::from(category_vec)),
            Arc::new(Float64Array::from(sales)),
            Arc::new(Int64Array::from(quantities)),
        ],
    )
    .unwrap()
}

fn bench_simple_aggregation(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_aggregation");

    for size in [100, 1000, 10000].iter() {
        let batch = generate_test_data(*size);
        let cube = ElastiCubeBuilder::new("test_cube")
            .add_dimension("region", DataType::Utf8).unwrap()
            .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
            .with_data(vec![batch]).unwrap()
            .build()
            .unwrap();

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| async {
                    black_box(
                        cube.query().unwrap()
                            .select(&["region", "sum(sales)"])
                            .group_by(&["region"])
                            .execute()
                            .await
                            .unwrap(),
                    )
                });
        });
    }

    group.finish();
}

fn bench_filtered_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("filtered_query");

    for size in [100, 1000, 10000].iter() {
        let batch = generate_test_data(*size);
        let cube = ElastiCubeBuilder::new("test_cube")
            .add_dimension("region", DataType::Utf8).unwrap()
            .add_dimension("product", DataType::Utf8).unwrap()
            .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
            .with_data(vec![batch]).unwrap()
            .build()
            .unwrap();

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| async {
                    black_box(
                        cube.query().unwrap()
                            .select(&["region", "product", "sum(sales)"])
                            .filter("region = 'North'")
                            .group_by(&["region", "product"])
                            .execute()
                            .await
                            .unwrap(),
                    )
                });
        });
    }

    group.finish();
}

fn bench_multi_aggregation(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_aggregation");

    for size in [100, 1000, 10000].iter() {
        let batch = generate_test_data(*size);
        let cube = ElastiCubeBuilder::new("test_cube")
            .add_dimension("region", DataType::Utf8).unwrap()
            .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
            .add_measure("quantity", DataType::Int64, AggFunc::Sum).unwrap()
            .with_data(vec![batch]).unwrap()
            .build()
            .unwrap();

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| async {
                    black_box(
                        cube.query().unwrap()
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
                            .unwrap(),
                    )
                });
        });
    }

    group.finish();
}

fn bench_complex_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_query");

    for size in [100, 1000, 10000].iter() {
        let batch = generate_test_data(*size);
        let cube = ElastiCubeBuilder::new("test_cube")
            .add_dimension("date", DataType::Utf8).unwrap()
            .add_dimension("region", DataType::Utf8).unwrap()
            .add_dimension("product", DataType::Utf8).unwrap()
            .add_dimension("category", DataType::Utf8).unwrap()
            .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
            .add_measure("quantity", DataType::Int64, AggFunc::Sum).unwrap()
            .with_data(vec![batch]).unwrap()
            .build()
            .unwrap();

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| async {
                    black_box(
                        cube.query().unwrap()
                            .select(&[
                                "region",
                                "category",
                                "sum(sales) as total_sales",
                                "sum(quantity) as total_qty",
                            ])
                            .filter("date >= '2024-01-15'")
                            .group_by(&["region", "category"])
                            .order_by(&["total_sales DESC"])
                            .limit(10)
                            .execute()
                            .await
                            .unwrap(),
                    )
                });
        });
    }

    group.finish();
}

fn bench_olap_slice(c: &mut Criterion) {
    let mut group = c.benchmark_group("olap_slice");

    for size in [100, 1000, 10000].iter() {
        let batch = generate_test_data(*size);
        let cube = ElastiCubeBuilder::new("test_cube")
            .add_dimension("region", DataType::Utf8).unwrap()
            .add_dimension("product", DataType::Utf8).unwrap()
            .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
            .with_data(vec![batch]).unwrap()
            .build()
            .unwrap();

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| async {
                    black_box(
                        cube.query().unwrap()
                            .slice("region", "North")
                            .select(&["product", "sum(sales)"])
                            .group_by(&["product"])
                            .execute()
                            .await
                            .unwrap(),
                    )
                });
        });
    }

    group.finish();
}

fn bench_olap_dice(c: &mut Criterion) {
    let mut group = c.benchmark_group("olap_dice");

    for size in [100, 1000, 10000].iter() {
        let batch = generate_test_data(*size);
        let cube = ElastiCubeBuilder::new("test_cube")
            .add_dimension("region", DataType::Utf8).unwrap()
            .add_dimension("product", DataType::Utf8).unwrap()
            .add_dimension("category", DataType::Utf8).unwrap()
            .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
            .with_data(vec![batch]).unwrap()
            .build()
            .unwrap();

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.to_async(tokio::runtime::Runtime::new().unwrap())
                .iter(|| async {
                    black_box(
                        cube.query().unwrap()
                            .dice(&[
                                ("region", vec!["North", "South"]),
                                ("product", vec!["Widget"]),
                            ])
                            .select(&["category", "sum(sales)"])
                            .group_by(&["category"])
                            .execute()
                            .await
                            .unwrap(),
                    )
                });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_aggregation,
    bench_filtered_query,
    bench_multi_aggregation,
    bench_complex_query,
    bench_olap_slice,
    bench_olap_dice,
);

criterion_main!(benches);
