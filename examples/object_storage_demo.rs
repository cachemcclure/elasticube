//! Object Storage Demo
//!
//! Demonstrates loading data from cloud object storage (S3, GCS, Azure).
//!
//! This example shows how to:
//! - Load Parquet files from AWS S3
//! - Load CSV files from Google Cloud Storage
//! - Load JSON files from Azure Blob Storage
//! - Configure authentication and storage-specific options
//! - Work with different file formats from object storage
//!
//! # Usage
//!
//! Run with object-storage feature enabled:
//! ```bash
//! cargo run --features object-storage --example object_storage_demo
//! ```
//!
//! # Authentication
//!
//! ## AWS S3
//! - Set environment variables: AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, AWS_REGION
//! - Or use ~/.aws/credentials file
//! - Or provide explicit credentials in code
//!
//! ## Google Cloud Storage (GCS)
//! - Set GOOGLE_APPLICATION_CREDENTIALS environment variable
//! - Or provide service account key in code
//!
//! ## Azure Blob Storage
//! - Provide storage account key or SAS token in code
//! - Or use environment variables

use elasticube_core::{
    ElastiCubeBuilder, AggFunc, Result,
    S3Source, GcsSource, AzureSource, StorageFileFormat,
};
use arrow::datatypes::DataType;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=============================================================================");
    println!("ElastiCube Object Storage Demo");
    println!("=============================================================================\n");

    // ===========================================================================
    // Example 1: Load Parquet file from AWS S3
    // ===========================================================================
    println!("--- Example 1: AWS S3 Parquet Data ---\n");

    // Method 1: Simple S3 load (uses default AWS credentials)
    println!("Method 1: Simple S3 load using environment credentials");
    println!("Code:");
    println!("  let cube = ElastiCubeBuilder::new(\"s3_sales\")");
    println!("      .load_s3(\"my-bucket\", \"data/sales.parquet\")");
    println!("      .build()?;\n");

    // Method 2: S3 with explicit configuration
    println!("Method 2: S3 with explicit configuration");
    println!("Code:");
    println!("  let source = S3Source::new(\"my-bucket\", \"data/sales.parquet\")");
    println!("      .with_region(\"us-west-2\")");
    println!("      .with_format(StorageFileFormat::Parquet)");
    println!("      .with_batch_size(8192);");
    println!();
    println!("  let cube = ElastiCubeBuilder::new(\"s3_sales\")");
    println!("      .load_s3_with(source)");
    println!("      .build()?;\n");

    // Method 3: S3 with explicit credentials
    println!("Method 3: S3 with explicit credentials");
    println!("Code:");
    println!("  let source = S3Source::new(\"my-bucket\", \"data/sales.csv\")");
    println!("      .with_region(\"us-east-1\")");
    println!("      .with_access_key(\"AKIAIOSFODNN7EXAMPLE\", \"secret_key\")");
    println!("      .with_format(StorageFileFormat::Csv)");
    println!("      .with_batch_size(4096);");
    println!();
    println!("  let cube = ElastiCubeBuilder::new(\"s3_sales_csv\")");
    println!("      .add_dimension(\"region\", DataType::Utf8)?");
    println!("      .add_dimension(\"date\", DataType::Date32)?");
    println!("      .add_measure(\"revenue\", DataType::Float64, AggFunc::Sum)?");
    println!("      .load_s3_with(source)");
    println!("      .build()?;\n");

    // Method 4: S3-compatible storage (MinIO, etc.)
    println!("Method 4: S3-compatible storage (MinIO, Cloudflare R2, etc.)");
    println!("Code:");
    println!("  let source = S3Source::new(\"my-bucket\", \"data/logs.json\")");
    println!("      .with_endpoint(\"http://localhost:9000\")  // MinIO endpoint");
    println!("      .with_access_key(\"minioadmin\", \"minioadmin\")");
    println!("      .with_format(StorageFileFormat::Json)");
    println!("      .with_batch_size(2048);");
    println!();
    println!("  let cube = ElastiCubeBuilder::new(\"minio_logs\")");
    println!("      .load_s3_with(source)");
    println!("      .build()?;\n");

    println!("Note: These are example code snippets. Actual execution requires valid S3 credentials.\n");

    // ===========================================================================
    // Example 2: Load CSV file from Google Cloud Storage
    // ===========================================================================
    println!("\n--- Example 2: Google Cloud Storage CSV Data ---\n");

    // Method 1: Simple GCS load (uses default credentials)
    println!("Method 1: Simple GCS load using default credentials");
    println!("Code:");
    println!("  let cube = ElastiCubeBuilder::new(\"gcs_analytics\")");
    println!("      .load_gcs(\"my-gcs-bucket\", \"data/analytics.parquet\")");
    println!("      .build()?;\n");

    // Method 2: GCS with explicit configuration
    println!("Method 2: GCS with service account key");
    println!("Code:");
    println!("  let source = GcsSource::new(\"my-bucket\", \"data/metrics.json\")");
    println!("      .with_service_account_key(\"/path/to/service-account-key.json\")");
    println!("      .with_format(StorageFileFormat::Json)");
    println!("      .with_batch_size(8192);");
    println!();
    println!("  let cube = ElastiCubeBuilder::new(\"gcs_metrics\")");
    println!("      .add_dimension(\"service\", DataType::Utf8)?");
    println!("      .add_dimension(\"timestamp\", DataType::Timestamp(TimeUnit::Second, None))?");
    println!("      .add_measure(\"requests\", DataType::Int64, AggFunc::Sum)?");
    println!("      .add_measure(\"latency_ms\", DataType::Float64, AggFunc::Avg)?");
    println!("      .load_gcs_with(source)");
    println!("      .build()?;\n");

    println!("Note: These are example code snippets. Actual execution requires valid GCS credentials.\n");

    // ===========================================================================
    // Example 3: Load JSON file from Azure Blob Storage
    // ===========================================================================
    println!("\n--- Example 3: Azure Blob Storage JSON Data ---\n");

    // Method 1: Simple Azure load
    println!("Method 1: Simple Azure Blob Storage load");
    println!("Code:");
    println!("  let cube = ElastiCubeBuilder::new(\"azure_reports\")");
    println!("      .load_azure(\"mystorageaccount\", \"mycontainer\", \"data/reports.parquet\")");
    println!("      .build()?;\n");

    // Method 2: Azure with access key
    println!("Method 2: Azure with access key authentication");
    println!("Code:");
    println!("  let source = AzureSource::new(\"mystorageaccount\", \"mycontainer\", \"data/logs.csv\")");
    println!("      .with_access_key(\"your-storage-account-key\")");
    println!("      .with_format(StorageFileFormat::Csv)");
    println!("      .with_batch_size(4096);");
    println!();
    println!("  let cube = ElastiCubeBuilder::new(\"azure_logs\")");
    println!("      .add_dimension(\"app_name\", DataType::Utf8)?");
    println!("      .add_dimension(\"log_level\", DataType::Utf8)?");
    println!("      .add_measure(\"error_count\", DataType::Int32, AggFunc::Sum)?");
    println!("      .load_azure_with(source)");
    println!("      .build()?;\n");

    // Method 3: Azure with SAS token
    println!("Method 3: Azure with SAS token authentication");
    println!("Code:");
    println!("  let source = AzureSource::new(\"mystorageaccount\", \"mycontainer\", \"data/events.json\")");
    println!("      .with_sas_token(\"?sv=2020-08-04&ss=bfqt&srt=sco&sp=rwdlacuptfx...\")");
    println!("      .with_format(StorageFileFormat::Json)");
    println!("      .with_batch_size(8192);");
    println!();
    println!("  let cube = ElastiCubeBuilder::new(\"azure_events\")");
    println!("      .load_azure_with(source)");
    println!("      .build()?;\n");

    println!("Note: These are example code snippets. Actual execution requires valid Azure credentials.\n");

    // ===========================================================================
    // Example 4: Querying data loaded from object storage
    // ===========================================================================
    println!("\n--- Example 4: Querying Data from Object Storage ---\n");

    println!("Once data is loaded from object storage, querying works the same as local files:");
    println!();
    println!("Code:");
    println!("  // Load cube from S3");
    println!("  let cube = ElastiCubeBuilder::new(\"sales\")");
    println!("      .load_s3(\"analytics-bucket\", \"data/sales.parquet\")");
    println!("      .build()?;");
    println!();
    println!("  // Query the data");
    println!("  let results = cube.query()");
    println!("      .select(&[\"region\", \"sum(revenue) as total_revenue\"])");
    println!("      .filter(\"date >= '2024-01-01'\")");
    println!("      .group_by(&[\"region\"])");
    println!("      .order_by(&[(\"total_revenue\", false)])  // descending");
    println!("      .limit(10)");
    println!("      .execute()");
    println!("      .await?;");
    println!();
    println!("  results.show();");
    println!();

    // ===========================================================================
    // Practical Tips
    // ===========================================================================
    println!("\n--- Practical Tips for Object Storage ---\n");

    println!("1. File Formats:");
    println!("   - Parquet: Best for analytics (columnar, compressed)");
    println!("   - CSV: Human-readable, widely supported");
    println!("   - JSON: Flexible structure, good for nested data\n");

    println!("2. Authentication:");
    println!("   - S3: AWS credentials chain (env vars, ~/.aws/credentials, IAM roles)");
    println!("   - GCS: GOOGLE_APPLICATION_CREDENTIALS environment variable");
    println!("   - Azure: Storage account key or SAS token\n");

    println!("3. Performance:");
    println!("   - Use Parquet for large datasets (10x+ faster than CSV)");
    println!("   - Adjust batch_size based on available memory");
    println!("   - Consider data locality (same region as compute)\n");

    println!("4. Cost Optimization:");
    println!("   - Parquet reduces storage costs (compressed)");
    println!("   - Minimize data transfer across regions");
    println!("   - Use lifecycle policies for older data\n");

    println!("5. Multi-Cloud Strategy:");
    println!("   - Same ElastiCube code works across all clouds");
    println!("   - Easy to migrate between providers");
    println!("   - Test with local MinIO before deploying to cloud\n");

    // ===========================================================================
    // Summary
    // ===========================================================================
    println!("\n=============================================================================");
    println!("Summary");
    println!("=============================================================================\n");

    println!("ElastiCube supports loading data from:");
    println!("  ✓ AWS S3 (and S3-compatible: MinIO, Cloudflare R2, DigitalOcean Spaces)");
    println!("  ✓ Google Cloud Storage (GCS)");
    println!("  ✓ Azure Blob Storage");
    println!();
    println!("Supported file formats:");
    println!("  ✓ Parquet (recommended for analytics)");
    println!("  ✓ CSV (with schema inference)");
    println!("  ✓ JSON (newline-delimited)");
    println!();
    println!("Authentication methods:");
    println!("  ✓ Environment variables");
    println!("  ✓ Credential files");
    println!("  ✓ Explicit keys/tokens in code");
    println!("  ✓ IAM roles (where available)");
    println!();
    println!("For real usage examples, ensure you have:");
    println!("  1. Valid cloud credentials configured");
    println!("  2. Access to the specified buckets/containers");
    println!("  3. Network connectivity to the cloud provider");
    println!();
    println!("=============================================================================\n");

    Ok(())
}
