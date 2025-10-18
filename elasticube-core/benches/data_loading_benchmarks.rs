//! Data loading performance benchmarks using Criterion
//!
//! Benchmarks various data loading operations:
//! - CSV loading
//! - Parquet loading
//! - JSON loading
//! - In-memory RecordBatch loading
//! - Cube building with different schema complexities

use arrow_array::{Float64Array, Int64Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema as ArrowSchema};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use elasticube_core::{AggFunc, ElastiCubeBuilder};
use parquet::arrow::ArrowWriter;
use std::fs::File;
use std::io::Write as IoWrite;
use std::sync::Arc;
use tempfile::NamedTempFile;

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

/// Create a temporary CSV file with test data
fn create_csv_file(num_rows: usize) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "date,region,product,category,sales,quantity").unwrap();

    let regions = vec!["North", "South", "East", "West"];
    let products = vec!["Widget", "Gadget", "Doohickey", "Thingamajig"];
    let categories = vec!["Electronics", "Furniture", "Clothing", "Food"];

    for i in 0..num_rows {
        let date = format!("2024-01-{:02}", (i % 28) + 1);
        let region = regions[i % regions.len()];
        let product = products[i % products.len()];
        let category = categories[i % categories.len()];
        let sales = 100.0 + (i as f64 * 0.5);
        let quantity = 10 + (i % 50);

        writeln!(
            file,
            "{},{},{},{},{},{}",
            date, region, product, category, sales, quantity
        )
        .unwrap();
    }

    file.flush().unwrap();
    file
}

/// Create a temporary Parquet file with test data
fn create_parquet_file(num_rows: usize) -> NamedTempFile {
    let batch = generate_test_data(num_rows);
    let file = NamedTempFile::new().unwrap();
    let file_handle = File::create(file.path()).unwrap();

    let mut writer = ArrowWriter::try_new(file_handle, batch.schema(), None).unwrap();
    writer.write(&batch).unwrap();
    writer.close().unwrap();

    file
}

/// Create a temporary JSON file with test data
fn create_json_file(num_rows: usize) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();

    let regions = vec!["North", "South", "East", "West"];
    let products = vec!["Widget", "Gadget", "Doohickey", "Thingamajig"];
    let categories = vec!["Electronics", "Furniture", "Clothing", "Food"];

    for i in 0..num_rows {
        let date = format!("2024-01-{:02}", (i % 28) + 1);
        let region = regions[i % regions.len()];
        let product = products[i % products.len()];
        let category = categories[i % categories.len()];
        let sales = 100.0 + (i as f64 * 0.5);
        let quantity = 10 + (i % 50);

        writeln!(
            file,
            r#"{{"date":"{}","region":"{}","product":"{}","category":"{}","sales":{},"quantity":{}}}"#,
            date, region, product, category, sales, quantity
        )
        .unwrap();
    }

    file.flush().unwrap();
    file
}

fn bench_csv_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("csv_loading");

    for size in [100, 1000, 5000].iter() {
        let csv_file = create_csv_file(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                black_box(
                    ElastiCubeBuilder::new("test_cube")
                        .add_dimension("date", DataType::Utf8).unwrap()
                        .add_dimension("region", DataType::Utf8).unwrap()
                        .add_dimension("product", DataType::Utf8).unwrap()
                        .add_dimension("category", DataType::Utf8).unwrap()
                        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
                        .add_measure("quantity", DataType::Int64, AggFunc::Sum).unwrap()
                        .load_csv(csv_file.path())
                        .unwrap()
                        .build()
                        .unwrap(),
                )
            });
        });
    }

    group.finish();
}

fn bench_parquet_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("parquet_loading");

    for size in [100, 1000, 5000].iter() {
        let parquet_file = create_parquet_file(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                black_box(
                    ElastiCubeBuilder::new("test_cube")
                        .add_dimension("date", DataType::Utf8).unwrap()
                        .add_dimension("region", DataType::Utf8).unwrap()
                        .add_dimension("product", DataType::Utf8).unwrap()
                        .add_dimension("category", DataType::Utf8).unwrap()
                        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
                        .add_measure("quantity", DataType::Int64, AggFunc::Sum).unwrap()
                        .load_parquet(parquet_file.path())
                        .unwrap()
                        .build()
                        .unwrap(),
                )
            });
        });
    }

    group.finish();
}

fn bench_json_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_loading");

    for size in [100, 1000, 5000].iter() {
        let json_file = create_json_file(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                black_box(
                    ElastiCubeBuilder::new("test_cube")
                        .add_dimension("date", DataType::Utf8).unwrap()
                        .add_dimension("region", DataType::Utf8).unwrap()
                        .add_dimension("product", DataType::Utf8).unwrap()
                        .add_dimension("category", DataType::Utf8).unwrap()
                        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
                        .add_measure("quantity", DataType::Int64, AggFunc::Sum).unwrap()
                        .load_json(json_file.path())
                        .unwrap()
                        .build()
                        .unwrap(),
                )
            });
        });
    }

    group.finish();
}

fn bench_in_memory_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("in_memory_loading");

    for size in [100, 1000, 10000].iter() {
        let batch = generate_test_data(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            let batch_clone = batch.clone();
            b.iter(|| {
                black_box(
                    ElastiCubeBuilder::new("test_cube")
                        .add_dimension("date", DataType::Utf8).unwrap()
                        .add_dimension("region", DataType::Utf8).unwrap()
                        .add_dimension("product", DataType::Utf8).unwrap()
                        .add_dimension("category", DataType::Utf8).unwrap()
                        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
                        .add_measure("quantity", DataType::Int64, AggFunc::Sum).unwrap()
                        .with_data(vec![batch_clone.clone().unwrap()])
                        .build()
                        .unwrap(),
                )
            });
        });
    }

    group.finish();
}

fn bench_cube_building_simple_schema(c: &mut Criterion) {
    let mut group = c.benchmark_group("cube_building_simple");

    for size in [100, 1000, 10000].iter() {
        let batch = generate_test_data(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            let batch_clone = batch.clone();
            b.iter(|| {
                black_box(
                    ElastiCubeBuilder::new("test_cube")
                        .add_dimension("region", DataType::Utf8).unwrap()
                        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
                        .with_data(vec![batch_clone.clone().unwrap()])
                        .build()
                        .unwrap(),
                )
            });
        });
    }

    group.finish();
}

fn bench_cube_building_complex_schema(c: &mut Criterion) {
    let mut group = c.benchmark_group("cube_building_complex");

    for size in [100, 1000, 10000].iter() {
        let batch = generate_test_data(*size);

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            let batch_clone = batch.clone();
            b.iter(|| {
                black_box(
                    ElastiCubeBuilder::new("test_cube")
                        .add_dimension("date", DataType::Utf8).unwrap()
                        .add_dimension("region", DataType::Utf8).unwrap()
                        .add_dimension("product", DataType::Utf8).unwrap()
                        .add_dimension("category", DataType::Utf8).unwrap()
                        .add_measure("sales", DataType::Float64, AggFunc::Sum).unwrap()
                        .add_measure("quantity", DataType::Int64, AggFunc::Sum).unwrap()
                        .with_data(vec![batch_clone.clone().unwrap()])
                        .build()
                        .unwrap(),
                )
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_csv_loading,
    bench_parquet_loading,
    bench_json_loading,
    bench_in_memory_loading,
    bench_cube_building_simple_schema,
    bench_cube_building_complex_schema,
);

criterion_main!(benches);
