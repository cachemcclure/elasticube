# ElastiCube Integration Tests

This directory contains comprehensive integration tests for the ElastiCube library.

## Test Organization

### Integration Tests (`tests/`)
- **`query_execution_tests.rs`**: End-to-end tests for query execution
  - Basic queries (SELECT, WHERE, GROUP BY, ORDER BY, LIMIT)
  - Aggregation functions (SUM, AVG, MIN, MAX, COUNT, COUNT DISTINCT)
  - OLAP operations (slice, dice, roll-up)
  - CSV data loading
  - SQL query execution
  - Edge cases (empty results, complex queries)

- **`property_tests.rs`**: Property-based tests using quickcheck
  - Query determinism (same query = same results)
  - Schema preservation under filters
  - LIMIT constraint enforcement
  - Aggregation correctness
  - GROUP BY row reduction
  - Empty filter behavior

- **`common/mod.rs`**: Shared test utilities
  - `SalesDataset`: Realistic sales data (small, medium, large)
  - `EcommerceDataset`: E-commerce orders and customers
  - `TimeSeriesDataset`: Sensor time-series data

### Benchmarks (`benches/`)
- **`query_benchmarks.rs`**: Query performance benchmarks
  - Simple aggregations
  - Filtered queries
  - Multi-aggregation queries
  - Complex queries with multiple operations
  - OLAP operations (slice, dice)

- **`data_loading_benchmarks.rs`**: Data loading benchmarks
  - CSV loading
  - Parquet loading
  - JSON loading
  - In-memory RecordBatch loading
  - Cube building with simple/complex schemas

## Running Tests

### Run All Tests
```bash
# Run unit tests and integration tests
cargo test

# Run with all optional features enabled
cargo test --features all-sources

# Run only integration tests
cargo test --test '*'
```

### Run Specific Test Suites
```bash
# Query execution tests
cargo test --test query_execution_tests

# Property-based tests
cargo test --test property_tests

# Common module tests
cargo test --test common
```

### Run Individual Tests
```bash
# Run a specific test by name
cargo test test_basic_query_execution

# Run tests matching a pattern
cargo test olap
```

### Run with Output
```bash
# Show println! output
cargo test -- --nocapture

# Show test execution output
cargo test -- --show-output
```

## Running Benchmarks

### Run All Benchmarks
```bash
# Run all benchmarks
cargo bench

# Run with all features
cargo bench --features all-sources
```

### Run Specific Benchmarks
```bash
# Query benchmarks only
cargo bench --bench query_benchmarks

# Data loading benchmarks only
cargo bench --bench data_loading_benchmarks

# Specific benchmark group
cargo bench simple_aggregation
```

### Benchmark Output
Benchmarks generate HTML reports in `target/criterion/`. Open `target/criterion/report/index.html` in a browser to view detailed performance analysis.

## Running Property Tests

Property tests use quickcheck to generate random test cases. By default, they run 100 test cases per property.

```bash
# Run property tests
cargo test --test property_tests

# Run with more test cases (via environment variable)
QUICKCHECK_TESTS=1000 cargo test --test property_tests
```

## Test Coverage

### Unit Tests (in `src/`)
- 90 unit tests covering core functionality
- Located in module test submodules (e.g., `cube::schema::tests`)

### Integration Tests (in `tests/`)
- 17+ integration tests for end-to-end workflows
- 6+ property-based tests for invariant verification
- 4+ common module tests for test utilities

### Benchmarks (in `benches/`)
- 12 benchmark groups covering queries and data loading
- Multiple dataset sizes (100, 1000, 5000, 10000 rows)

## CI/CD Integration

### GitHub Actions Example
```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --features all-sources
      - name: Run benchmarks
        run: cargo bench --features all-sources --no-run

  property-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run property tests
        run: cargo test --test property_tests
        env:
          QUICKCHECK_TESTS: 500
```

## Performance Monitoring

### Baseline Benchmarks
Create a baseline for comparison:
```bash
cargo bench --features all-sources -- --save-baseline main
```

### Compare Performance
After making changes:
```bash
cargo bench --features all-sources -- --baseline main
```

This will show performance differences compared to the baseline.

## Test Data

Test datasets are generated programmatically in `tests/common/mod.rs`:

- **SalesDataset**: Realistic sales data with dimensions (date, region, product, category) and measures (sales, quantity, profit)
- **EcommerceDataset**: E-commerce orders with customer and product information
- **TimeSeriesDataset**: Sensor readings with timestamps and measurements

## Adding New Tests

### Integration Test
1. Create a new file in `tests/` (e.g., `my_feature_tests.rs`)
2. Import common utilities: `use common::{SalesDataset};`
3. Write test functions with `#[tokio::test]` attribute
4. Run with `cargo test --test my_feature_tests`

### Property Test
1. Add to `tests/property_tests.rs`
2. Define an `Arbitrary` type for input generation
3. Write property function returning `TestResult`
4. Add `#[quickcheck]` attribute
5. Add to `run_property_tests()` function

### Benchmark
1. Add to `benches/query_benchmarks.rs` or `benches/data_loading_benchmarks.rs`
2. Define benchmark function with `fn bench_name(c: &mut Criterion)`
3. Use `c.bench_function()` or `c.benchmark_group()`
4. Add to `criterion_group!()` macro
5. Run with `cargo bench --bench my_benchmarks`

## Troubleshooting

### Tests Timing Out
Increase timeout for async tests:
```rust
#[tokio::test(flavor = "multi_thread")]
async fn my_test() {
    // test code
}
```

### Property Tests Discarding Too Many Cases
Adjust input generation to create more valid cases, or increase `max_tests`:
```rust
QuickCheck::new()
    .tests(100)
    .max_tests(500)  // Allow more discards
    .quickcheck(my_property as fn(Input) -> TestResult);
```

### Benchmarks Not Stable
Ensure system is quiet during benchmarking:
- Close other applications
- Disable CPU frequency scaling
- Run multiple times and compare

## Resources

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Quickcheck Documentation](https://docs.rs/quickcheck/)
- [Integration Testing in Rust](https://doc.rust-lang.org/rust-by-example/testing/integration_testing.html)
