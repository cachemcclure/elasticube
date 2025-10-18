//! Python bindings for ElastiCube
//!
//! This module provides Python access to the high-performance ElastiCube library
//! built in Rust using Apache Arrow and DataFusion.

use pyo3::prelude::*;
use pyo3::types::PyBytes;

use elasticube_core::{AggFunc, ElastiCube, ElastiCubeBuilder};
use arrow::datatypes::DataType;
use arrow::ipc::writer::StreamWriter;
use arrow::ipc::reader::StreamReader;
use std::sync::{Arc, Mutex};

/// Python wrapper for ElastiCubeBuilder
#[pyclass]
struct PyElastiCubeBuilder {
    builder: Option<ElastiCubeBuilder>,
}

#[pymethods]
impl PyElastiCubeBuilder {
    /// Create a new cube builder
    #[new]
    fn new(name: String) -> Self {
        Self {
            builder: Some(ElastiCubeBuilder::new(name)),
        }
    }

    /// Add a dimension to the cube
    fn add_dimension(&mut self, name: String, data_type: String) -> PyResult<()> {
        let dt = parse_datatype(&data_type)?;
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        self.builder = Some(builder.add_dimension(name, dt)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?);
        Ok(())
    }

    /// Add a measure to the cube
    fn add_measure(
        &mut self,
        name: String,
        data_type: String,
        agg_func: String,
    ) -> PyResult<()> {
        let dt = parse_datatype(&data_type)?;
        let agg = parse_agg_func(&agg_func)?;
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        self.builder = Some(builder.add_measure(name, dt, agg)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?);
        Ok(())
    }

    /// Load data from a CSV file
    fn load_csv(&mut self, path: String) -> PyResult<()> {
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        self.builder = Some(builder.load_csv(path));
        Ok(())
    }

    /// Load data from a Parquet file
    fn load_parquet(&mut self, path: String) -> PyResult<()> {
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        self.builder = Some(builder.load_parquet(path));
        Ok(())
    }

    /// Load data from a JSON file
    fn load_json(&mut self, path: String) -> PyResult<()> {
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        self.builder = Some(builder.load_json(path));
        Ok(())
    }

    /// Build the cube
    fn build(&mut self) -> PyResult<Py<PyElastiCube>> {
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Builder already consumed. Create a new PyElastiCubeBuilder to build another cube."
            )
        })?;

        // Build the cube (consumes the builder)
        let cube = builder.build()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        // Wrap in Arc<Mutex<>> and PyElastiCube for update support
        Python::attach(|py| {
            Py::new(py, PyElastiCube {
                cube: Arc::new(Mutex::new(cube)),
            })
        })
    }
}

/// Python wrapper for ElastiCube
///
/// Uses Mutex for interior mutability to support update operations
#[pyclass]
struct PyElastiCube {
    cube: Arc<Mutex<ElastiCube>>,
}

#[pymethods]
impl PyElastiCube {
    /// Create a query builder
    fn query(&self) -> PyResult<PyQueryBuilder> {
        let cube = self.cube.lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock error: {}", e)))?;

        // Clone the cube to create an Arc for the query
        let cube_arc = Arc::new((*cube).clone());

        let query_builder = cube_arc.query()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(PyQueryBuilder {
            builder: Some(query_builder),
        })
    }

    /// Get cube name
    fn name(&self) -> PyResult<String> {
        let cube = self.cube.lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock error: {}", e)))?;
        Ok(cube.schema().name().to_string())
    }

    /// Get number of rows
    fn row_count(&self) -> PyResult<usize> {
        let cube = self.cube.lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock error: {}", e)))?;
        Ok(cube.row_count())
    }

    /// Get number of batches in the cube
    fn batch_count(&self) -> PyResult<usize> {
        let cube = self.cube.lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock error: {}", e)))?;
        Ok(cube.batch_count())
    }

    /// Append rows from PyArrow Table/RecordBatch
    ///
    /// Args:
    ///     data: PyArrow Table or RecordBatch containing rows to append
    ///
    /// Returns:
    ///     Number of rows added
    fn append_rows<'py>(&self, py: Python<'py>, data: Bound<'py, PyAny>) -> PyResult<usize> {
        // Convert PyArrow Table/RecordBatch to Arrow RecordBatch via IPC
        let batches = pyarrow_to_recordbatches(py, data)?;

        if batches.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "No data to append"
            ));
        }

        let mut cube = self.cube.lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock error: {}", e)))?;

        // Append each batch
        let mut total_added = 0;
        for batch in batches {
            let added = cube.append_rows(batch)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            total_added += added;
        }

        Ok(total_added)
    }

    /// Append multiple batches
    ///
    /// Args:
    ///     batches: List of PyArrow Tables/RecordBatches
    ///
    /// Returns:
    ///     Total number of rows added
    fn append_batches<'py>(&self, py: Python<'py>, batches_list: Vec<Bound<'py, PyAny>>) -> PyResult<usize> {
        let mut all_batches = Vec::new();

        for py_data in batches_list {
            let batches = pyarrow_to_recordbatches(py, py_data)?;
            all_batches.extend(batches);
        }

        let mut cube = self.cube.lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock error: {}", e)))?;

        cube.append_batches(all_batches)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Delete rows matching a filter expression
    ///
    /// Args:
    ///     filter_expr: SQL WHERE clause (e.g., "sales < 100" or "region = 'North'")
    ///
    /// Returns:
    ///     Number of rows deleted
    fn delete_rows<'py>(&self, py: Python<'py>, filter_expr: String) -> PyResult<usize> {
        // Clone the cube data to avoid holding lock across thread boundary
        let cube_arc = {
            let cube = self.cube.lock()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock error: {}", e)))?;
            Arc::new((*cube).clone())
        };

        // Execute async delete in blocking context
        let result = Python::detach(py, || {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async {
                    // We need a mutable cube for deletion, so unwrap the Arc
                    let mut cube_mut = Arc::try_unwrap(cube_arc)
                        .unwrap_or_else(|arc| (*arc).clone());

                    cube_mut.delete_rows(&filter_expr).await
                        .map(|deleted| (deleted, cube_mut))
                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
                })
        })?;

        // Update the original cube with the modified version
        let (deleted, updated_cube) = result;
        {
            let mut cube = self.cube.lock()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock error: {}", e)))?;
            *cube = updated_cube;
        }

        Ok(deleted)
    }

    /// Update rows matching a filter with replacement data
    ///
    /// Args:
    ///     filter_expr: SQL WHERE clause to identify rows to update
    ///     replacement_data: PyArrow Table/RecordBatch with updated rows
    ///
    /// Returns:
    ///     Tuple of (rows_deleted, rows_added)
    fn update_rows<'py>(
        &self,
        py: Python<'py>,
        filter_expr: String,
        replacement_data: Bound<'py, PyAny>
    ) -> PyResult<(usize, usize)> {
        // Convert PyArrow data to RecordBatches
        let batches = pyarrow_to_recordbatches(py, replacement_data)?;

        if batches.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "No replacement data provided"
            ));
        }

        // For simplicity, concatenate all batches into one
        // In practice, update_rows expects a single batch
        let schema = batches[0].schema();
        let replacement_batch = if batches.len() == 1 {
            batches.into_iter().next().unwrap()
        } else {
            arrow::compute::concat_batches(&schema, &batches)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?
        };

        // Clone the cube to avoid holding lock across thread boundary
        let cube_arc = {
            let cube = self.cube.lock()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock error: {}", e)))?;
            Arc::new((*cube).clone())
        };

        // Execute async update in blocking context
        let result = Python::detach(py, || {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async {
                    let mut cube_mut = Arc::try_unwrap(cube_arc)
                        .unwrap_or_else(|arc| (*arc).clone());

                    cube_mut.update_rows(&filter_expr, replacement_batch).await
                        .map(|counts| (counts, cube_mut))
                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
                })
        })?;

        // Update the original cube with the modified version
        let (counts, updated_cube) = result;
        {
            let mut cube = self.cube.lock()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock error: {}", e)))?;
            *cube = updated_cube;
        }

        Ok(counts)
    }

    /// Consolidate all batches into a single batch
    ///
    /// Returns:
    ///     Number of batches before consolidation
    fn consolidate_batches(&self) -> PyResult<usize> {
        let mut cube = self.cube.lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock error: {}", e)))?;

        cube.consolidate_batches()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}

/// Python wrapper for QueryBuilder
#[pyclass]
struct PyQueryBuilder {
    builder: Option<elasticube_core::QueryBuilder>,
}

#[pymethods]
impl PyQueryBuilder {
    /// Select columns
    fn select(&mut self, columns: Vec<String>) -> PyResult<()> {
        let col_refs: Vec<&str> = columns.iter().map(|s| s.as_str()).collect();
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        self.builder = Some(builder.select(&col_refs));
        Ok(())
    }

    /// Add a filter condition
    fn filter(&mut self, condition: String) -> PyResult<()> {
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        self.builder = Some(builder.filter(&condition));
        Ok(())
    }

    /// Group by columns
    fn group_by(&mut self, columns: Vec<String>) -> PyResult<()> {
        let col_refs: Vec<&str> = columns.iter().map(|s| s.as_str()).collect();
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        self.builder = Some(builder.group_by(&col_refs));
        Ok(())
    }

    /// Order by columns
    fn order_by(&mut self, columns: Vec<String>) -> PyResult<()> {
        let col_refs: Vec<&str> = columns.iter().map(|s| s.as_str()).collect();
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        self.builder = Some(builder.order_by(&col_refs));
        Ok(())
    }

    /// Limit results
    fn limit(&mut self, n: usize) -> PyResult<()> {
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        self.builder = Some(builder.limit(n));
        Ok(())
    }

    /// Execute the query and return results as PyArrow Table
    fn execute<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Query builder already executed")
        })?;

        // Execute query in a blocking context using Python's detach API
        let result = Python::detach(py, || {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async {
                    builder.execute().await
                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
                })
        })?;

        // Convert QueryResult to PyArrow RecordBatch using Arrow IPC
        let batches = result.batches();

        if batches.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "No results returned",
            ));
        }

        // Serialize to Arrow IPC format
        let mut buffer = Vec::new();
        {
            let mut writer = StreamWriter::try_new(&mut buffer, &batches[0].schema())
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            for batch in batches {
                writer.write(batch)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            }

            writer.finish()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        }

        // Import pyarrow
        let pyarrow = py.import("pyarrow")
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyImportError, _>(
                format!("Failed to import pyarrow: {}. Please install pyarrow: pip install pyarrow", e)
            ))?;
        let ipc = pyarrow.getattr("ipc")?;

        // Create a PyBytes object from the buffer
        let py_bytes = PyBytes::new(py, &buffer);

        // Use PyArrow to read the IPC data
        let reader = ipc.call_method1("open_stream", (py_bytes,))?;
        let table = reader.call_method0("read_all")?;

        Ok(table)
    }

    /// Execute query and return as Pandas DataFrame
    fn to_pandas<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let arrow_table = self.execute(py)?;

        // Convert PyArrow Table to Pandas using to_pandas()
        let pandas_df = arrow_table.call_method0("to_pandas")?;

        Ok(pandas_df)
    }

    /// Execute query and return as Polars DataFrame (high-performance alternative to Pandas)
    fn to_polars<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let arrow_table = self.execute(py)?;

        // Import polars
        let polars = py.import("polars")
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyImportError, _>(
                format!("Failed to import polars: {}. Please install polars: pip install polars", e)
            ))?;

        // Create Polars DataFrame from PyArrow Table (zero-copy!)
        let polars_df = polars.call_method1("from_arrow", (arrow_table,))?;

        Ok(polars_df)
    }
}

/// Convert PyArrow Table or RecordBatch to Rust Arrow RecordBatches
///
/// Handles both PyArrow Table and RecordBatch objects by serializing via IPC
fn pyarrow_to_recordbatches<'py>(
    py: Python<'py>,
    data: Bound<'py, PyAny>,
) -> PyResult<Vec<arrow::record_batch::RecordBatch>> {
    // Import pyarrow
    let pyarrow = py.import("pyarrow")
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyImportError, _>(
            format!("Failed to import pyarrow: {}. Please install pyarrow: pip install pyarrow", e)
        ))?;
    let ipc = pyarrow.getattr("ipc")?;

    // Serialize to Arrow IPC format
    let mut buffer = Vec::new();
    {
        // Get the schema
        let schema_obj = data.getattr("schema")?;

        // Create a writer
        let py_buffer = py.import("io")?.call_method0("BytesIO")?;
        let writer = ipc.call_method1("new_stream", (&py_buffer, &schema_obj))?;

        // Write the data (works for both Table and RecordBatch)
        writer.call_method1("write", (&data,))?;
        writer.call_method0("close")?;

        // Get the bytes
        py_buffer.call_method1("seek", (0,))?;
        let bytes_obj = py_buffer.call_method0("read")?;
        let bytes: &[u8] = bytes_obj.extract()?;
        buffer.extend_from_slice(bytes);
    }

    // Deserialize using Rust Arrow IPC reader
    let cursor = std::io::Cursor::new(buffer);
    let reader = StreamReader::try_new(cursor, None)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            format!("Failed to create Arrow IPC reader: {}", e)
        ))?;

    // Collect all batches
    let mut batches = Vec::new();
    for batch_result in reader {
        let batch = batch_result
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to read Arrow batch: {}", e)
            ))?;
        batches.push(batch);
    }

    Ok(batches)
}

/// Helper function to parse DataType from string
fn parse_datatype(s: &str) -> PyResult<DataType> {
    match s.to_lowercase().as_str() {
        "int32" | "int" => Ok(DataType::Int32),
        "int64" | "long" => Ok(DataType::Int64),
        "float32" | "float" => Ok(DataType::Float32),
        "float64" | "double" => Ok(DataType::Float64),
        "utf8" | "string" | "str" => Ok(DataType::Utf8),
        "bool" | "boolean" => Ok(DataType::Boolean),
        "date32" | "date" => Ok(DataType::Date32),
        "date64" => Ok(DataType::Date64),
        "timestamp" => Ok(DataType::Timestamp(
            arrow::datatypes::TimeUnit::Microsecond,
            None,
        )),
        _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Unknown data type: {}", s),
        )),
    }
}

/// Helper function to parse AggFunc from string
fn parse_agg_func(s: &str) -> PyResult<AggFunc> {
    match s.to_lowercase().as_str() {
        "sum" => Ok(AggFunc::Sum),
        "avg" | "average" | "mean" => Ok(AggFunc::Avg),
        "min" => Ok(AggFunc::Min),
        "max" => Ok(AggFunc::Max),
        "count" => Ok(AggFunc::Count),
        "count_distinct" | "countdistinct" => Ok(AggFunc::CountDistinct),
        "median" => Ok(AggFunc::Median),
        "stddev" | "std" => Ok(AggFunc::StdDev),
        "variance" | "var" => Ok(AggFunc::Variance),
        "first" => Ok(AggFunc::First),
        "last" => Ok(AggFunc::Last),
        _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Unknown aggregation function: {}", s),
        )),
    }
}

/// Python module definition
#[pymodule]
fn _elasticube(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyElastiCubeBuilder>()?;
    m.add_class::<PyElastiCube>()?;
    m.add_class::<PyQueryBuilder>()?;
    Ok(())
}
