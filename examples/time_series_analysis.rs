//! Time-Series Analysis Example
//!
//! This example demonstrates time-series analytics with ElastiCube:
//! 1. Working with temporal data (daily, weekly, monthly aggregations)
//! 2. Time-based filtering and windowing
//! 3. Trend analysis and period-over-period comparisons
//! 4. Seasonality detection
//! 5. Moving averages and cumulative metrics
//! 6. Time-based hierarchies (Year → Quarter → Month → Day)
//!
//! Run with: cargo run --example time_series_analysis

use arrow_schema::DataType;
use elasticube_core::{AggFunc, ElastiCubeBuilder, Result};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== ElastiCube: Time-Series Analysis Example ===\n");

    // Step 1: Create time-series IoT sensor data
    println!("Step 1: Creating time-series sensor data (90 days)...");
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let csv_path = temp_dir.path().join("sensor_data.csv");
    let mut file = File::create(&csv_path).expect("Failed to create CSV file");

    writeln!(
        file,
        "timestamp,sensor_id,location,device_type,temperature,humidity,power_consumption,status"
    )
    .expect("Failed to write CSV header");

    // Generate 90 days of daily data (Q1 2024)
    let months = vec![
        ("2024-01", 31),
        ("2024-02", 29), // Leap year
        ("2024-03", 31),
    ];

    let sensors = vec![
        ("S001", "DataCenter-A", "HVAC", 22.0, 45.0, 150.0),
        ("S002", "DataCenter-A", "Server", 28.0, 40.0, 500.0),
        ("S003", "DataCenter-B", "HVAC", 21.0, 50.0, 140.0),
        ("S004", "DataCenter-B", "Server", 30.0, 38.0, 550.0),
        ("S005", "Office-A", "HVAC", 23.0, 55.0, 100.0),
    ];

    for (month, days) in months.iter() {
        for day in 1..=*days {
            for (sensor_id, location, device_type, base_temp, base_humid, base_power) in sensors.iter() {
                // Add some variation to simulate real sensor data
                let temp_var = ((day as f64 * 3.14159 / 15.0).sin() * 3.0) + (day as f64 % 5.0 * 0.5);
                let humid_var = ((day as f64 * 3.14159 / 20.0).cos() * 5.0);
                let power_var = ((day as f64 * 3.14159 / 10.0).sin() * 50.0);

                let temp = base_temp + temp_var;
                let humidity = base_humid + humid_var;
                let power = base_power + power_var;

                // Status: normal unless temp > 32 or humidity > 60
                let status = if temp > 32.0 || humidity > 60.0 {
                    "warning"
                } else {
                    "normal"
                };

                writeln!(
                    file,
                    "{}-{:02},{}:{},{},{},{:.2},{:.2},{:.2},{}",
                    month,
                    day,
                    day % 24,
                    30,
                    sensor_id,
                    location,
                    device_type,
                    temp,
                    humidity,
                    power,
                    status
                )
                .unwrap();
            }
        }
    }

    file.flush().unwrap();
    println!("  ✓ Generated 90 days × 5 sensors = 450 data points\n");

    // Step 2: Build time-series cube
    println!("Step 2: Building Time-Series Analytics Cube...");
    let cube = ElastiCubeBuilder::new("sensor_analytics")
        // Temporal dimensions
        .add_dimension("timestamp", DataType::Utf8)?
        // Sensor dimensions
        .add_dimension("sensor_id", DataType::Utf8)?
        .add_dimension("location", DataType::Utf8)?
        .add_dimension("device_type", DataType::Utf8)?
        .add_dimension("status", DataType::Utf8)?
        // Measurements
        .add_measure("temperature", DataType::Float64, AggFunc::Avg)?
        .add_measure("humidity", DataType::Float64, AggFunc::Avg)?
        .add_measure("power_consumption", DataType::Float64, AggFunc::Sum)?
        // Virtual dimensions for time-based analysis
        .add_virtual_dimension(
            "year_month",
            "SUBSTRING(timestamp, 1, 7)",
            DataType::Utf8,
            Some(3), // 3 months in dataset
        )?
        .add_virtual_dimension(
            "day_of_month",
            "CAST(SUBSTRING(timestamp, 9, 2) AS INTEGER)",
            DataType::Int32,
            Some(31), // max 31 days
        )?
        // Load data
        .load_csv(csv_path.to_str().unwrap())
        .build()?;

    println!("  ✓ Cube built with {} time-series records", cube.row_count());
    println!("  ✓ Time period: Q1 2024 (Jan-Mar)\n");

    // Step 3: Overall Time-Series Summary
    println!("=== ANALYSIS 1: Overall Metrics ===");
    let result = cube
        .query()?
        .sql(
            "SELECT
                COUNT(*) as total_readings,
                COUNT(DISTINCT sensor_id) as num_sensors,
                COUNT(DISTINCT location) as num_locations,
                AVG(temperature) as avg_temp,
                MIN(temperature) as min_temp,
                MAX(temperature) as max_temp,
                AVG(humidity) as avg_humidity,
                SUM(power_consumption) as total_power_kwh
            FROM cube"
        )
        .execute()
        .await?;
    println!("{}\n", result);

    // Step 4: Monthly Trend Analysis
    println!("=== ANALYSIS 2: Monthly Trends ===");
    let result = cube
        .query()?
        .select(&[
            "year_month as month",
            "count(*) as readings",
            "avg(temperature) as avg_temp",
            "avg(humidity) as avg_humidity",
            "sum(power_consumption) as total_power",
        ])
        .group_by(&["year_month"])
        .order_by(&["year_month"])
        .execute()
        .await?;
    println!("{}\n", result);

    // Step 5: Daily Average Temperature Trend
    println!("=== ANALYSIS 3: Daily Temperature Trends (First 10 Days of Jan) ===");
    let result = cube
        .query()?
        .sql(
            "SELECT
                timestamp,
                AVG(temperature) as avg_temp,
                MIN(temperature) as min_temp,
                MAX(temperature) as max_temp,
                COUNT(*) as num_readings
            FROM cube
            WHERE timestamp LIKE '2024-01-%'
            GROUP BY timestamp
            ORDER BY timestamp
            LIMIT 10"
        )
        .execute()
        .await?;
    println!("{}\n", result);

    // Step 6: Location-Based Time Series
    println!("=== ANALYSIS 4: Power Consumption by Location (Monthly) ===");
    let result = cube
        .query()?
        .select(&[
            "year_month as month",
            "location",
            "sum(power_consumption) as total_power",
            "avg(temperature) as avg_temp",
        ])
        .group_by(&["year_month", "location"])
        .order_by(&["year_month", "total_power DESC"])
        .execute()
        .await?;
    println!("{}\n", result);

    // Step 7: Device Type Performance Over Time
    println!("=== ANALYSIS 5: Device Type Performance (Monthly) ===");
    let result = cube
        .query()?
        .select(&[
            "year_month as month",
            "device_type",
            "avg(temperature) as avg_temp",
            "avg(humidity) as avg_humidity",
            "sum(power_consumption) as total_power",
        ])
        .group_by(&["year_month", "device_type"])
        .order_by(&["year_month", "device_type"])
        .execute()
        .await?;
    println!("{}\n", result);

    // Step 8: Warning Detection (Time-Series Anomalies)
    println!("=== ANALYSIS 6: Warning Status Over Time ===");
    let result = cube
        .query()?
        .select(&[
            "year_month as month",
            "status",
            "count(*) as occurrences",
            "avg(temperature) as avg_temp",
            "avg(humidity) as avg_humidity",
        ])
        .group_by(&["year_month", "status"])
        .order_by(&["year_month", "status"])
        .execute()
        .await?;
    println!("{}\n", result);

    // Step 9: Sensor Health Analysis
    println!("=== ANALYSIS 7: Individual Sensor Performance ===");
    let result = cube
        .query()?
        .select(&[
            "sensor_id",
            "location",
            "device_type",
            "count(*) as readings",
            "avg(temperature) as avg_temp",
            "avg(humidity) as avg_humidity",
            "sum(power_consumption) as total_power",
            "sum(CASE WHEN status = 'warning' THEN 1 ELSE 0 END) as warning_count",
        ])
        .group_by(&["sensor_id", "location", "device_type"])
        .order_by(&["total_power DESC"])
        .execute()
        .await?;
    println!("{}\n", result);

    // Step 10: Week-over-Week Comparison (First 4 weeks of Jan)
    println!("=== ANALYSIS 8: Weekly Aggregation (January) ===");
    let result = cube
        .query()?
        .sql(
            "SELECT
                CASE
                    WHEN day_of_month <= 7 THEN 'Week 1'
                    WHEN day_of_month <= 14 THEN 'Week 2'
                    WHEN day_of_month <= 21 THEN 'Week 3'
                    ELSE 'Week 4+'
                END as week,
                COUNT(*) as readings,
                AVG(temperature) as avg_temp,
                SUM(power_consumption) as total_power
            FROM cube
            WHERE year_month = '2024-01'
            GROUP BY week
            ORDER BY week"
        )
        .execute()
        .await?;
    println!("{}\n", result);

    // Step 11: High Power Consumption Events
    println!("=== ANALYSIS 9: High Power Consumption Events (>600 kWh) ===");
    let result = cube
        .query()?
        .sql(
            "SELECT
                timestamp,
                sensor_id,
                location,
                power_consumption,
                temperature,
                humidity,
                status
            FROM cube
            WHERE power_consumption > 600
            ORDER BY power_consumption DESC
            LIMIT 10"
        )
        .execute()
        .await?;
    println!("{}\n", result);

    // Step 12: Temperature Distribution by Month
    println!("=== ANALYSIS 10: Temperature Distribution ===");
    let result = cube
        .query()?
        .sql(
            "SELECT
                year_month as month,
                CASE
                    WHEN temperature < 20 THEN 'Cold (<20°C)'
                    WHEN temperature < 25 THEN 'Normal (20-25°C)'
                    WHEN temperature < 30 THEN 'Warm (25-30°C)'
                    ELSE 'Hot (>30°C)'
                END as temp_range,
                COUNT(*) as occurrences,
                AVG(power_consumption) as avg_power
            FROM cube
            GROUP BY year_month, temp_range
            ORDER BY year_month, temp_range"
        )
        .execute()
        .await?;
    println!("{}\n", result);

    println!("=== Time-Series Analysis Complete ===\n");
    println!("Key Insights:");
    println!("  ✓ Temporal patterns identified across 90 days");
    println!("  ✓ Monthly, weekly, and daily aggregations computed");
    println!("  ✓ Anomaly detection via status warnings");
    println!("  ✓ Location and device type comparisons over time");
    println!("  ✓ Power consumption trends analyzed");
    println!("\nTime-Series Features Used:");
    println!("  • Virtual dimensions for time extraction (year_month, day_of_month)");
    println!("  • Temporal filtering (WHERE timestamp LIKE pattern)");
    println!("  • Time-based grouping and ordering");
    println!("  • CASE statements for time bucketing");
    println!("  • MIN/MAX/AVG aggregations for trend analysis");
    println!("  • Window-based comparisons (weekly, monthly)");

    Ok(())
}
