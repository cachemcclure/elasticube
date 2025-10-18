# Object Storage Quick Start Guide

ElastiCube now supports loading data directly from cloud object storage! This guide shows you how to get started in minutes.

## Installation

Add ElastiCube to your `Cargo.toml` with the object-storage feature:

```toml
[dependencies]
elasticube-core = { path = "./elasticube-core", features = ["object-storage"] }

# Or enable all data sources
elasticube-core = { path = "./elasticube-core", features = ["all-sources"] }
```

## Quick Examples

### AWS S3

```rust
use elasticube_core::{ElastiCubeBuilder, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Load Parquet file from S3 (uses environment credentials)
    let cube = ElastiCubeBuilder::new("sales")
        .load_s3("my-bucket", "data/sales.parquet")
        .build()?;

    // Query the data
    let results = cube.query()
        .select(&["region", "sum(revenue)"])
        .group_by(&["region"])
        .execute()
        .await?;

    results.show();
    Ok(())
}
```

### Google Cloud Storage

```rust
use elasticube_core::{ElastiCubeBuilder, GcsSource, StorageFileFormat, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Load CSV from GCS with explicit configuration
    let source = GcsSource::new("my-bucket", "data/sales.csv")
        .with_format(StorageFileFormat::Csv)
        .with_service_account_key("/path/to/key.json");

    let cube = ElastiCubeBuilder::new("sales")
        .load_gcs_with(source)
        .build()?;

    Ok(())
}
```

### Azure Blob Storage

```rust
use elasticube_core::{ElastiCubeBuilder, AzureSource, StorageFileFormat, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Load JSON from Azure with access key
    let source = AzureSource::new("myaccount", "mycontainer", "data/metrics.json")
        .with_access_key("your-storage-account-key")
        .with_format(StorageFileFormat::Json);

    let cube = ElastiCubeBuilder::new("metrics")
        .load_azure_with(source)
        .build()?;

    Ok(())
}
```

## Authentication

### AWS S3

Set environment variables:
```bash
export AWS_ACCESS_KEY_ID=your_access_key
export AWS_SECRET_ACCESS_KEY=your_secret_key
export AWS_REGION=us-west-2
```

Or use `~/.aws/credentials`:
```ini
[default]
aws_access_key_id = your_access_key
aws_secret_access_key = your_secret_key
```

### Google Cloud Storage

Set environment variable:
```bash
export GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account-key.json
```

### Azure Blob Storage

Provide credentials in code:
```rust
let source = AzureSource::new("account", "container", "path")
    .with_access_key("your-key")
    // or
    .with_sas_token("?sv=2020-08-04&...");
```

## File Formats

All cloud providers support three formats:

```rust
use elasticube_core::StorageFileFormat;

// Parquet (recommended for analytics)
.with_format(StorageFileFormat::Parquet)

// CSV (human-readable)
.with_format(StorageFileFormat::Csv)

// JSON (flexible structure)
.with_format(StorageFileFormat::Json)
```

## Complete Example

```rust
use elasticube_core::{
    ElastiCubeBuilder, S3Source, StorageFileFormat,
    AggFunc, Result
};
use arrow::datatypes::DataType;

#[tokio::main]
async fn main() -> Result<()> {
    // Configure S3 source
    let source = S3Source::new("analytics-bucket", "sales/2024/sales.parquet")
        .with_region("us-west-2")
        .with_format(StorageFileFormat::Parquet)
        .with_batch_size(8192);

    // Build cube with schema
    let cube = ElastiCubeBuilder::new("sales_2024")
        .add_dimension("region", DataType::Utf8)?
        .add_dimension("product", DataType::Utf8)?
        .add_dimension("date", DataType::Date32)?
        .add_measure("revenue", DataType::Float64, AggFunc::Sum)?
        .add_measure("quantity", DataType::Int32, AggFunc::Sum)?
        .load_s3_with(source)
        .build()?;

    // Query with OLAP operations
    let results = cube.query()
        .select(&[
            "region",
            "product",
            "sum(revenue) as total_revenue",
            "sum(quantity) as total_quantity"
        ])
        .filter("date >= '2024-01-01' AND date < '2024-04-01'")
        .group_by(&["region", "product"])
        .order_by(&[("total_revenue", false)])  // descending
        .limit(10)
        .execute()
        .await?;

    // Display results
    results.show();

    Ok(())
}
```

## S3-Compatible Storage

ElastiCube works with any S3-compatible service:

```rust
// MinIO
let source = S3Source::new("my-bucket", "data/file.parquet")
    .with_endpoint("http://localhost:9000")
    .with_access_key("minioadmin", "minioadmin");

// Cloudflare R2
let source = S3Source::new("my-bucket", "data/file.parquet")
    .with_endpoint("https://account-id.r2.cloudflarestorage.com")
    .with_access_key("access_key", "secret_key");

// DigitalOcean Spaces
let source = S3Source::new("my-bucket", "data/file.parquet")
    .with_endpoint("https://nyc3.digitaloceanspaces.com")
    .with_region("nyc3")
    .with_access_key("access_key", "secret_key");
```

## Performance Tips

1. **Use Parquet for large datasets**: 10-100x faster than CSV, better compression
2. **Adjust batch size**: Larger batches (16K-32K) for throughput, smaller (2K-4K) for memory
3. **Same-region deployment**: Deploy compute in same region as storage to minimize latency
4. **Schema inference**: Provide explicit schema when possible to skip inference step

## Testing Locally

Use MinIO for local testing without cloud costs:

```bash
# Run MinIO with Docker
docker run -p 9000:9000 -p 9001:9001 \
  -e MINIO_ROOT_USER=minioadmin \
  -e MINIO_ROOT_PASSWORD=minioadmin \
  minio/minio server /data --console-address ":9001"

# Access at http://localhost:9001
# Create bucket and upload test files
```

Then in your code:
```rust
let source = S3Source::new("test-bucket", "data/test.parquet")
    .with_endpoint("http://localhost:9000")
    .with_access_key("minioadmin", "minioadmin");
```

## More Information

- **Full Example**: See `examples/object_storage_demo.rs`
- **API Documentation**: Run `cargo doc --features all-sources --open`
- **Detailed Guide**: See `PHASE_6_OBJECT_STORAGE_COMPLETION.md`
- **Project Status**: See `PROJECT_CHECKLIST.md`

## Supported Platforms

- ✅ AWS S3
- ✅ S3-compatible (MinIO, Cloudflare R2, DigitalOcean Spaces, Backblaze B2, Wasabi)
- ✅ Google Cloud Storage (GCS)
- ✅ Azure Blob Storage
- ✅ Local filesystem (via object_store)

## Support

File issues at: https://github.com/cachemcclure/elasticube/issues

---

Happy analyzing! 🚀
