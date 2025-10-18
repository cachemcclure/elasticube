//! Data update operations for ElastiCube
//!
//! This module provides functionality for modifying cube data:
//! - Append new rows incrementally
//! - Delete rows based on filter predicates
//! - Update existing rows
//!
//! Since Apache Arrow RecordBatch is immutable, all operations create new batches
//! and reconstruct the cube's internal data structure.

use crate::error::{Error, Result};
use arrow::array::BooleanArray;
use arrow::compute;
use arrow::datatypes::Schema as ArrowSchema;
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

/// Helper to concatenate multiple RecordBatches into one
///
/// This is used internally for append operations and batch consolidation.
///
/// # Arguments
/// * `schema` - The Arrow schema for all batches
/// * `batches` - Slice of RecordBatches to concatenate
///
/// # Returns
/// A single RecordBatch containing all rows from the input batches
pub(crate) fn concat_record_batches(
    schema: &Arc<ArrowSchema>,
    batches: &[RecordBatch],
) -> Result<RecordBatch> {
    if batches.is_empty() {
        return Err(Error::data("Cannot concatenate empty batch list"));
    }

    if batches.len() == 1 {
        return Ok(batches[0].clone());
    }

    compute::concat_batches(schema, batches).map_err(|e| {
        Error::arrow(format!("Failed to concatenate batches: {}", e))
    })
}

/// Filter a RecordBatch based on a boolean array predicate
///
/// # Arguments
/// * `batch` - The RecordBatch to filter
/// * `predicate` - A BooleanArray where true means keep the row
///
/// # Returns
/// A new RecordBatch containing only rows where predicate is true
#[allow(dead_code)]
pub(crate) fn filter_record_batch(
    batch: &RecordBatch,
    predicate: &BooleanArray,
) -> Result<RecordBatch> {
    compute::filter_record_batch(batch, predicate).map_err(|e| {
        Error::arrow(format!("Failed to filter record batch: {}", e))
    })
}

/// Validates that a RecordBatch schema matches the expected schema
///
/// # Arguments
/// * `expected` - The expected Arrow schema
/// * `actual` - The schema of the batch being validated
///
/// # Returns
/// Ok if schemas match, Err otherwise
pub(crate) fn validate_batch_schema(
    expected: &Arc<ArrowSchema>,
    actual: &Arc<ArrowSchema>,
) -> Result<()> {
    // Check field count
    if expected.fields().len() != actual.fields().len() {
        return Err(Error::schema(format!(
            "Schema field count mismatch: expected {}, got {}",
            expected.fields().len(),
            actual.fields().len()
        )));
    }

    // Check each field
    for (expected_field, actual_field) in expected.fields().iter().zip(actual.fields().iter()) {
        if expected_field.name() != actual_field.name() {
            return Err(Error::schema(format!(
                "Schema field name mismatch: expected '{}', got '{}'",
                expected_field.name(),
                actual_field.name()
            )));
        }

        if expected_field.data_type() != actual_field.data_type() {
            return Err(Error::schema(format!(
                "Schema field '{}' type mismatch: expected {:?}, got {:?}",
                expected_field.name(),
                expected_field.data_type(),
                actual_field.data_type()
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Float64Array, Int32Array};
    use arrow::datatypes::{DataType, Field};

    #[test]
    fn test_concat_single_batch() {
        let schema = Arc::new(ArrowSchema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("value", DataType::Float64, false),
        ]));

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int32Array::from(vec![1, 2, 3])),
                Arc::new(Float64Array::from(vec![1.0, 2.0, 3.0])),
            ],
        )
        .unwrap();

        let result = concat_record_batches(&schema, &[batch.clone()]).unwrap();
        assert_eq!(result.num_rows(), 3);
    }

    #[test]
    fn test_concat_multiple_batches() {
        let schema = Arc::new(ArrowSchema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("value", DataType::Float64, false),
        ]));

        let batch1 = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int32Array::from(vec![1, 2])),
                Arc::new(Float64Array::from(vec![1.0, 2.0])),
            ],
        )
        .unwrap();

        let batch2 = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int32Array::from(vec![3, 4])),
                Arc::new(Float64Array::from(vec![3.0, 4.0])),
            ],
        )
        .unwrap();

        let result = concat_record_batches(&schema, &[batch1, batch2]).unwrap();
        assert_eq!(result.num_rows(), 4);
    }

    #[test]
    fn test_concat_empty_fails() {
        let schema = Arc::new(ArrowSchema::new(vec![
            Field::new("id", DataType::Int32, false),
        ]));

        let result = concat_record_batches(&schema, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_record_batch_keeps_true_rows() {
        let schema = Arc::new(ArrowSchema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("value", DataType::Float64, false),
        ]));

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int32Array::from(vec![1, 2, 3, 4])),
                Arc::new(Float64Array::from(vec![1.0, 2.0, 3.0, 4.0])),
            ],
        )
        .unwrap();

        // Keep only rows where id is even (2, 4)
        let predicate = BooleanArray::from(vec![false, true, false, true]);
        let result = filter_record_batch(&batch, &predicate).unwrap();

        assert_eq!(result.num_rows(), 2);
        let id_array = result
            .column(0)
            .as_any()
            .downcast_ref::<Int32Array>()
            .unwrap();
        assert_eq!(id_array.value(0), 2);
        assert_eq!(id_array.value(1), 4);
    }

    #[test]
    fn test_validate_batch_schema_success() {
        let schema1 = Arc::new(ArrowSchema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, false),
        ]));

        let schema2 = Arc::new(ArrowSchema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, false),
        ]));

        assert!(validate_batch_schema(&schema1, &schema2).is_ok());
    }

    #[test]
    fn test_validate_batch_schema_field_count_mismatch() {
        let schema1 = Arc::new(ArrowSchema::new(vec![
            Field::new("id", DataType::Int32, false),
        ]));

        let schema2 = Arc::new(ArrowSchema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, false),
        ]));

        let result = validate_batch_schema(&schema1, &schema2);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("field count mismatch"));
    }

    #[test]
    fn test_validate_batch_schema_type_mismatch() {
        let schema1 = Arc::new(ArrowSchema::new(vec![
            Field::new("id", DataType::Int32, false),
        ]));

        let schema2 = Arc::new(ArrowSchema::new(vec![
            Field::new("id", DataType::Int64, false),
        ]));

        let result = validate_batch_schema(&schema1, &schema2);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("type mismatch"));
    }

    #[test]
    fn test_validate_batch_schema_name_mismatch() {
        let schema1 = Arc::new(ArrowSchema::new(vec![
            Field::new("id", DataType::Int32, false),
        ]));

        let schema2 = Arc::new(ArrowSchema::new(vec![
            Field::new("identifier", DataType::Int32, false),
        ]));

        let result = validate_batch_schema(&schema1, &schema2);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name mismatch"));
    }
}
