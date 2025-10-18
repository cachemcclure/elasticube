//! Common test utilities and example datasets
//!
//! This module provides shared test data and helper functions that can be used
//! across multiple integration tests.

use arrow_array::{Float64Array, Int32Array, Int64Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema as ArrowSchema};
use std::sync::Arc;

/// Sales dataset with typical OLAP dimensions and measures
pub struct SalesDataset {
    pub batch: RecordBatch,
    pub row_count: usize,
}

impl SalesDataset {
    /// Create a small sales dataset for quick tests
    pub fn small() -> Self {
        Self::with_size(50)
    }

    /// Create a medium sales dataset
    pub fn medium() -> Self {
        Self::with_size(500)
    }

    /// Create a large sales dataset
    pub fn large() -> Self {
        Self::with_size(5000)
    }

    /// Create a sales dataset with specified number of rows
    pub fn with_size(num_rows: usize) -> Self {
        let schema = Arc::new(ArrowSchema::new(vec![
            Field::new("date", DataType::Utf8, false),
            Field::new("year", DataType::Int32, false),
            Field::new("month", DataType::Int32, false),
            Field::new("region", DataType::Utf8, false),
            Field::new("product", DataType::Utf8, false),
            Field::new("category", DataType::Utf8, false),
            Field::new("sales", DataType::Float64, false),
            Field::new("quantity", DataType::Int64, false),
            Field::new("profit", DataType::Float64, false),
        ]));

        let regions = vec!["North", "South", "East", "West"];
        let products = vec!["Widget", "Gadget", "Doohickey", "Thingamajig"];
        let categories = vec!["Electronics", "Furniture", "Clothing", "Food"];

        let mut dates = Vec::with_capacity(num_rows);
        let mut years = Vec::with_capacity(num_rows);
        let mut months = Vec::with_capacity(num_rows);
        let mut region_vec = Vec::with_capacity(num_rows);
        let mut product_vec = Vec::with_capacity(num_rows);
        let mut category_vec = Vec::with_capacity(num_rows);
        let mut sales = Vec::with_capacity(num_rows);
        let mut quantities = Vec::with_capacity(num_rows);
        let mut profits = Vec::with_capacity(num_rows);

        for i in 0..num_rows {
            let year = 2024;
            let month = (i % 12) + 1;
            let day = (i % 28) + 1;

            dates.push(format!("{}-{:02}-{:02}", year, month, day));
            years.push(year);
            months.push(month as i32);
            region_vec.push(regions[i % regions.len()].to_string());
            product_vec.push(products[i % products.len()].to_string());
            category_vec.push(categories[i % categories.len()].to_string());

            let base_sales = 100.0 + (i as f64 * 0.5);
            sales.push(base_sales);
            quantities.push((10 + (i % 50)) as i64);
            profits.push(base_sales * 0.3); // 30% profit margin
        }

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(dates)),
                Arc::new(Int32Array::from(years)),
                Arc::new(Int32Array::from(months)),
                Arc::new(StringArray::from(region_vec)),
                Arc::new(StringArray::from(product_vec)),
                Arc::new(StringArray::from(category_vec)),
                Arc::new(Float64Array::from(sales)),
                Arc::new(Int64Array::from(quantities)),
                Arc::new(Float64Array::from(profits)),
            ],
        )
        .unwrap();

        Self {
            batch,
            row_count: num_rows,
        }
    }
}

/// E-commerce dataset with customer and order information
pub struct EcommerceDataset {
    pub batch: RecordBatch,
    pub row_count: usize,
}

impl EcommerceDataset {
    /// Create an e-commerce dataset
    pub fn new(num_rows: usize) -> Self {
        let schema = Arc::new(ArrowSchema::new(vec![
            Field::new("order_id", DataType::Int64, false),
            Field::new("customer_id", DataType::Int64, false),
            Field::new("date", DataType::Utf8, false),
            Field::new("product", DataType::Utf8, false),
            Field::new("price", DataType::Float64, false),
            Field::new("quantity", DataType::Int64, false),
            Field::new("shipping_cost", DataType::Float64, false),
        ]));

        let products = vec!["Laptop", "Phone", "Tablet", "Headphones", "Monitor"];

        let mut order_ids = Vec::with_capacity(num_rows);
        let mut customer_ids = Vec::with_capacity(num_rows);
        let mut dates = Vec::with_capacity(num_rows);
        let mut product_vec = Vec::with_capacity(num_rows);
        let mut prices = Vec::with_capacity(num_rows);
        let mut quantities = Vec::with_capacity(num_rows);
        let mut shipping_costs = Vec::with_capacity(num_rows);

        for i in 0..num_rows {
            order_ids.push(i as i64 + 1000);
            customer_ids.push((i % 100) as i64 + 1);
            dates.push(format!("2024-{:02}-{:02}", (i % 12) + 1, (i % 28) + 1));
            product_vec.push(products[i % products.len()].to_string());
            prices.push(99.99 + (i as f64 * 0.1));
            quantities.push((i % 5 + 1) as i64);
            shipping_costs.push(5.0 + (i % 10) as f64);
        }

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int64Array::from(order_ids)),
                Arc::new(Int64Array::from(customer_ids)),
                Arc::new(StringArray::from(dates)),
                Arc::new(StringArray::from(product_vec)),
                Arc::new(Float64Array::from(prices)),
                Arc::new(Int64Array::from(quantities)),
                Arc::new(Float64Array::from(shipping_costs)),
            ],
        )
        .unwrap();

        Self {
            batch,
            row_count: num_rows,
        }
    }
}

/// Time series dataset for testing temporal queries
pub struct TimeSeriesDataset {
    pub batch: RecordBatch,
    pub row_count: usize,
}

impl TimeSeriesDataset {
    /// Create a time series dataset
    pub fn new(num_rows: usize) -> Self {
        let schema = Arc::new(ArrowSchema::new(vec![
            Field::new("timestamp", DataType::Utf8, false),
            Field::new("sensor_id", DataType::Utf8, false),
            Field::new("temperature", DataType::Float64, false),
            Field::new("humidity", DataType::Float64, false),
            Field::new("pressure", DataType::Float64, false),
        ]));

        let sensors = vec!["sensor_1", "sensor_2", "sensor_3", "sensor_4"];

        let mut timestamps = Vec::with_capacity(num_rows);
        let mut sensor_ids = Vec::with_capacity(num_rows);
        let mut temperatures = Vec::with_capacity(num_rows);
        let mut humidities = Vec::with_capacity(num_rows);
        let mut pressures = Vec::with_capacity(num_rows);

        for i in 0..num_rows {
            let hour = i % 24;
            let minute = (i * 5) % 60;
            timestamps.push(format!("2024-01-01T{:02}:{:02}:00", hour, minute));
            sensor_ids.push(sensors[i % sensors.len()].to_string());
            temperatures.push(20.0 + ((i as f64) * 0.1).sin() * 5.0);
            humidities.push(50.0 + ((i as f64) * 0.2).cos() * 10.0);
            pressures.push(1013.0 + ((i as f64) * 0.15).sin() * 3.0);
        }

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(timestamps)),
                Arc::new(StringArray::from(sensor_ids)),
                Arc::new(Float64Array::from(temperatures)),
                Arc::new(Float64Array::from(humidities)),
                Arc::new(Float64Array::from(pressures)),
            ],
        )
        .unwrap();

        Self {
            batch,
            row_count: num_rows,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sales_dataset_small() {
        let dataset = SalesDataset::small();
        assert_eq!(dataset.row_count, 50);
        assert_eq!(dataset.batch.num_rows(), 50);
        assert_eq!(dataset.batch.num_columns(), 9);
    }

    #[test]
    fn test_sales_dataset_medium() {
        let dataset = SalesDataset::medium();
        assert_eq!(dataset.row_count, 500);
        assert_eq!(dataset.batch.num_rows(), 500);
    }

    #[test]
    fn test_ecommerce_dataset() {
        let dataset = EcommerceDataset::new(100);
        assert_eq!(dataset.row_count, 100);
        assert_eq!(dataset.batch.num_rows(), 100);
        assert_eq!(dataset.batch.num_columns(), 7);
    }

    #[test]
    fn test_time_series_dataset() {
        let dataset = TimeSeriesDataset::new(200);
        assert_eq!(dataset.row_count, 200);
        assert_eq!(dataset.batch.num_rows(), 200);
        assert_eq!(dataset.batch.num_columns(), 5);
    }
}
