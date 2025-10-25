//! Python bindings for ElastiCube
//!
//! This module provides Python access to the high-performance ElastiCube library
//! built in Rust using Apache Arrow and DataFusion.

use pyo3::prelude::*;
use pyo3::types::{PyBytes, IntoPyDict};

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

    /// Add a hierarchy to the cube
    ///
    /// # Arguments
    /// * `name` - Name of the hierarchy (e.g., "time_hierarchy")
    /// * `levels` - List of dimension names forming the hierarchy from coarse to fine
    ///              (e.g., ["year", "quarter", "month"])
    ///
    /// # Example
    /// ```python
    /// builder.add_hierarchy("time", ["year", "quarter", "month"])
    /// ```
    fn add_hierarchy(&mut self, name: String, levels: Vec<String>) -> PyResult<()> {
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        self.builder = Some(builder.add_hierarchy(name, levels)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?);
        Ok(())
    }

    /// Add a calculated measure (derived from expression)
    ///
    /// # Arguments
    /// * `name` - Name for the calculated measure
    /// * `expression` - SQL expression (e.g., "revenue - cost")
    /// * `data_type` - Expected result data type
    /// * `agg_func` - Aggregation function
    ///
    /// # Example
    /// ```python
    /// builder.add_calculated_measure("profit", "revenue - cost", "float64", "sum")
    /// ```
    fn add_calculated_measure(
        &mut self,
        name: String,
        expression: String,
        data_type: String,
        agg_func: String,
    ) -> PyResult<()> {
        let dt = parse_datatype(&data_type)?;
        let agg = parse_agg_func(&agg_func)?;
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        self.builder = Some(builder.add_calculated_measure(name, expression, dt, agg)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?);
        Ok(())
    }

    /// Add a virtual dimension (computed dimension)
    ///
    /// # Arguments
    /// * `name` - Name for the virtual dimension
    /// * `expression` - SQL expression (e.g., "EXTRACT(YEAR FROM sale_date)")
    /// * `data_type` - Expected result data type
    ///
    /// # Example
    /// ```python
    /// builder.add_virtual_dimension("year", "EXTRACT(YEAR FROM sale_date)", "int32")
    /// ```
    fn add_virtual_dimension(
        &mut self,
        name: String,
        expression: String,
        data_type: String,
    ) -> PyResult<()> {
        let dt = parse_datatype(&data_type)?;
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        self.builder = Some(builder.add_virtual_dimension(name, expression, dt)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?);
        Ok(())
    }

    /// Set the cube description
    ///
    /// # Arguments
    /// * `description` - Human-readable description of the cube
    ///
    /// # Example
    /// ```python
    /// builder.with_description("Sales data cube for 2024")
    /// ```
    fn with_description(&mut self, description: String) -> PyResult<()> {
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        self.builder = Some(builder.with_description(description));
        Ok(())
    }

    /// Load data from a Polars DataFrame
    ///
    /// # Arguments
    /// * `df` - Polars DataFrame containing the data
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Raises
    /// * `ImportError` - If polars is not installed
    /// * `TypeError` - If df is not a polars.DataFrame
    /// * `ValueError` - If DataFrame is empty or has incompatible schema
    ///
    /// # Example
    /// ```python
    /// import polars as pl
    /// df = pl.DataFrame({
    ///     "region": ["North", "South"],
    ///     "sales": [100.0, 200.0]
    /// })
    /// cube = ElastiCubeBuilder("sales") \
    ///     .add_dimension("region", "utf8") \
    ///     .add_measure("sales", "float64", "sum") \
    ///     .load_from_polars(df) \
    ///     .build()
    /// ```
    fn load_from_polars(&mut self, df: Bound<'_, PyAny>) -> PyResult<()> {
        let py = df.py();

        // Convert to Arrow Table first (like Pandas does with pyarrow.Table.from_pandas)
        let arrow_table = df.call_method0("to_arrow")
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to convert Polars DataFrame to Arrow: {}", e)
            ))?;

        // Normalize schema to handle type mismatches
        let normalized_table = normalize_arrow_schema(py, arrow_table)?;

        // Convert to RecordBatches
        let batches = pyarrow_to_recordbatches(py, normalized_table)?;

        if batches.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "No data batches found"
            ));
        }

        // Extract schema from first batch
        let schema = batches[0].schema();

        // Take the builder and add the batches
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        self.builder = Some(builder.load_record_batches(schema, batches)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?);
        Ok(())
    }

    /// Load data from a Pandas DataFrame
    ///
    /// # Arguments
    /// * `df` - Pandas DataFrame containing the data
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Raises
    /// * `ImportError` - If pandas is not installed
    /// * `TypeError` - If df is not a pandas.DataFrame
    /// * `ValueError` - If DataFrame is empty or has incompatible schema
    ///
    /// # Example
    /// ```python
    /// import pandas as pd
    /// df = pd.DataFrame({
    ///     "date": pd.date_range("2024-01-01", periods=3),
    ///     "revenue": [100.0, 200.0, 150.0]
    /// })
    /// cube = ElastiCubeBuilder("revenue") \
    ///     .load_from_pandas(df) \
    ///     .build()
    /// ```
    fn load_from_pandas(&mut self, df: Bound<'_, PyAny>) -> PyResult<()> {
        let py = df.py();
        // Try to import pandas with helpful error message
        let pandas = py.import("pandas")
            .map_err(|_| PyErr::new::<pyo3::exceptions::PyImportError, _>(
                "Failed to import pandas. Please install it: pip install pandas"
            ))?;

        // Type check
        let dataframe_class = pandas.getattr("DataFrame")?;
        let is_dataframe = df.is_instance(&dataframe_class)?;
        if !is_dataframe {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Expected pandas.DataFrame, got different type"
            ));
        }

        // Check for empty DataFrame
        let row_count: usize = df.call_method0("__len__")?.extract()?;
        if row_count == 0 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Cannot load empty DataFrame. Provide data with at least one row."
            ));
        }

        // Import pyarrow
        let pyarrow = py.import("pyarrow")
            .map_err(|_| PyErr::new::<pyo3::exceptions::PyImportError, _>(
                "Failed to import pyarrow. Please install it: pip install pyarrow"
            ))?;

        // Convert to Arrow Table
        let table_class = pyarrow.getattr("Table")?;
        let arrow_table = table_class.call_method1("from_pandas", (&df,))?;

        // Normalize schema to handle type mismatches
        let normalized_table = normalize_arrow_schema(py, arrow_table)?;

        // Convert to RecordBatches
        let batches = pyarrow_to_recordbatches(py, normalized_table)?;

        if batches.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "No data batches found"
            ));
        }

        // Extract schema from first batch
        let schema = batches[0].schema();

        // Take the builder and add the batches
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        self.builder = Some(builder.load_record_batches(schema, batches)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?);
        Ok(())
    }

    /// Load data from a PyArrow Table directly (zero-copy when possible)
    ///
    /// # Arguments
    /// * `table` - PyArrow Table containing the data
    ///
    /// # Returns
    /// Self for method chaining
    ///
    /// # Raises
    /// * `TypeError` - If table is not a pyarrow.Table
    /// * `ValueError` - If table is empty or has incompatible schema
    ///
    /// # Example
    /// ```python
    /// import pyarrow as pa
    /// table = pa.table({
    ///     "product": ["A", "B", "C"],
    ///     "quantity": [10, 20, 15]
    /// })
    /// cube = ElastiCubeBuilder("inventory") \
    ///     .load_from_arrow(table) \
    ///     .build()
    /// ```
    fn load_from_arrow(&mut self, table: Bound<'_, PyAny>) -> PyResult<()> {
        let py = table.py();

        // Normalize schema to handle type mismatches
        let normalized_table = normalize_arrow_schema(py, table)?;

        // Convert to RecordBatches
        let batches = pyarrow_to_recordbatches(py, normalized_table)?;

        if batches.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "No data batches found"
            ));
        }

        // Extract schema from first batch
        let schema = batches[0].schema();

        // Take the builder and add the batches
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        self.builder = Some(builder.load_record_batches(schema, batches)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?);
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

    /// Append rows from a Polars DataFrame
    ///
    /// This method provides a convenient way to incrementally load data from Polars
    /// DataFrames without recreating the cube. Useful for streaming large datasets
    /// or updating cubes with new data.
    ///
    /// Args:
    ///     df: Polars DataFrame containing rows to append
    ///
    /// Returns:
    ///     Number of rows added
    ///
    /// Example:
    ///     >>> import polars as pl
    ///     >>> # Load initial data
    ///     >>> cube = ElastiCubeBuilder("sales") \\
    ///     ...     .load_from_polars(initial_df) \\
    ///     ...     .build()
    ///     >>> # Append new data later
    ///     >>> new_data = pl.DataFrame({"region": ["West"], "sales": [500.0]})
    ///     >>> rows_added = cube.append_from_polars(new_data)
    fn append_from_polars<'py>(&self, py: Python<'py>, df: Bound<'py, PyAny>) -> PyResult<usize> {
        // Convert to Arrow Table
        let arrow_table = df.call_method0("to_arrow")
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to convert Polars DataFrame to Arrow: {}", e)
            ))?;

        // Normalize schema to handle type mismatches
        let normalized_table = normalize_arrow_schema(py, arrow_table)?;

        // Use existing append_rows method
        self.append_rows(py, normalized_table)
    }

    /// Append rows from a Pandas DataFrame
    ///
    /// This method provides a convenient way to incrementally load data from Pandas
    /// DataFrames without recreating the cube. Useful for streaming large datasets
    /// or updating cubes with new data.
    ///
    /// Args:
    ///     df: Pandas DataFrame containing rows to append
    ///
    /// Returns:
    ///     Number of rows added
    ///
    /// Example:
    ///     >>> import pandas as pd
    ///     >>> # Load initial data
    ///     >>> cube = ElastiCubeBuilder("sales") \\
    ///     ...     .load_from_pandas(initial_df) \\
    ///     ...     .build()
    ///     >>> # Append new data later
    ///     >>> new_data = pd.DataFrame({"date": ["2024-02-01"], "revenue": [300.0]})
    ///     >>> rows_added = cube.append_from_pandas(new_data)
    fn append_from_pandas<'py>(&self, py: Python<'py>, df: Bound<'py, PyAny>) -> PyResult<usize> {
        // Import pyarrow
        let pyarrow = py.import("pyarrow")
            .map_err(|_| PyErr::new::<pyo3::exceptions::PyImportError, _>(
                "Failed to import pyarrow. Please install it: pip install pyarrow"
            ))?;

        // Convert to Arrow Table
        let table_class = pyarrow.getattr("Table")?;
        let arrow_table = table_class.call_method1("from_pandas", (&df,))?;

        // Normalize schema to handle type mismatches
        let normalized_table = normalize_arrow_schema(py, arrow_table)?;

        // Use existing append_rows method
        self.append_rows(py, normalized_table)
    }

    /// Append rows from a PyArrow Table
    ///
    /// This method provides a convenient way to incrementally load data from PyArrow
    /// Tables with automatic type normalization. While similar to append_rows(),
    /// this method includes the same normalization logic as load_from_arrow().
    ///
    /// Args:
    ///     table: PyArrow Table containing rows to append
    ///
    /// Returns:
    ///     Number of rows added
    ///
    /// Example:
    ///     >>> import pyarrow as pa
    ///     >>> # Load initial data
    ///     >>> cube = ElastiCubeBuilder("inventory") \\
    ///     ...     .load_from_arrow(initial_table) \\
    ///     ...     .build()
    ///     >>> # Append new data later
    ///     >>> new_table = pa.table({"product": ["D"], "quantity": [25]})
    ///     >>> rows_added = cube.append_from_arrow(new_table)
    fn append_from_arrow<'py>(&self, py: Python<'py>, table: Bound<'py, PyAny>) -> PyResult<usize> {
        // Normalize schema to handle type mismatches
        let normalized_table = normalize_arrow_schema(py, table)?;

        // Use existing append_rows method
        self.append_rows(py, normalized_table)
    }

    /// Get all dimensions
    ///
    /// Returns:
    ///     List of dimension dictionaries with keys: name, data_type, cardinality
    fn dimensions<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyList>> {
        let cube = self.cube.lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock error: {}", e)))?;

        let dims = cube.dimensions();
        let py_list = pyo3::types::PyList::empty(py);

        for dim in dims {
            let dict = pyo3::types::PyDict::new(py);
            dict.set_item("name", dim.name())?;
            dict.set_item("data_type", format!("{:?}", dim.data_type()))?;
            dict.set_item("cardinality", dim.cardinality())?;
            py_list.append(dict)?;
        }

        Ok(py_list)
    }

    /// Get all measures
    ///
    /// Returns:
    ///     List of measure dictionaries with keys: name, data_type, agg_func
    fn measures<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyList>> {
        let cube = self.cube.lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock error: {}", e)))?;

        let measures = cube.measures();
        let py_list = pyo3::types::PyList::empty(py);

        for measure in measures {
            let dict = pyo3::types::PyDict::new(py);
            dict.set_item("name", measure.name())?;
            dict.set_item("data_type", format!("{:?}", measure.data_type()))?;
            dict.set_item("agg_func", format!("{:?}", measure.default_agg()))?;
            py_list.append(dict)?;
        }

        Ok(py_list)
    }

    /// Get all hierarchies
    ///
    /// Returns:
    ///     List of hierarchy dictionaries with keys: name, levels
    fn hierarchies<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyList>> {
        let cube = self.cube.lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock error: {}", e)))?;

        let hierarchies = cube.hierarchies();
        let py_list = pyo3::types::PyList::empty(py);

        for hierarchy in hierarchies {
            let dict = pyo3::types::PyDict::new(py);
            dict.set_item("name", hierarchy.name())?;
            dict.set_item("levels", hierarchy.levels())?;
            py_list.append(dict)?;
        }

        Ok(py_list)
    }

    /// Get a specific dimension by name
    ///
    /// Args:
    ///     name: Name of the dimension to retrieve
    ///
    /// Returns:
    ///     Dictionary with dimension metadata or None if not found
    fn get_dimension<'py>(&self, py: Python<'py>, name: String) -> PyResult<Option<Bound<'py, pyo3::types::PyDict>>> {
        let cube = self.cube.lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock error: {}", e)))?;

        if let Some(dim) = cube.get_dimension(&name) {
            let dict = pyo3::types::PyDict::new(py);
            dict.set_item("name", dim.name())?;
            dict.set_item("data_type", format!("{:?}", dim.data_type()))?;
            dict.set_item("cardinality", dim.cardinality())?;
            Ok(Some(dict))
        } else {
            Ok(None)
        }
    }

    /// Get a specific measure by name
    ///
    /// Args:
    ///     name: Name of the measure to retrieve
    ///
    /// Returns:
    ///     Dictionary with measure metadata or None if not found
    fn get_measure<'py>(&self, py: Python<'py>, name: String) -> PyResult<Option<Bound<'py, pyo3::types::PyDict>>> {
        let cube = self.cube.lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock error: {}", e)))?;

        if let Some(measure) = cube.get_measure(&name) {
            let dict = pyo3::types::PyDict::new(py);
            dict.set_item("name", measure.name())?;
            dict.set_item("data_type", format!("{:?}", measure.data_type()))?;
            dict.set_item("agg_func", format!("{:?}", measure.default_agg()))?;
            Ok(Some(dict))
        } else {
            Ok(None)
        }
    }

    /// Get a specific hierarchy by name
    ///
    /// Args:
    ///     name: Name of the hierarchy to retrieve
    ///
    /// Returns:
    ///     Dictionary with hierarchy metadata or None if not found
    fn get_hierarchy<'py>(&self, py: Python<'py>, name: String) -> PyResult<Option<Bound<'py, pyo3::types::PyDict>>> {
        let cube = self.cube.lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock error: {}", e)))?;

        if let Some(hierarchy) = cube.get_hierarchy(&name) {
            let dict = pyo3::types::PyDict::new(py);
            dict.set_item("name", hierarchy.name())?;
            dict.set_item("levels", hierarchy.levels())?;
            Ok(Some(dict))
        } else {
            Ok(None)
        }
    }

    /// Get the cube description
    ///
    /// Returns:
    ///     Description string or None if not set
    fn description(&self) -> PyResult<Option<String>> {
        let cube = self.cube.lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock error: {}", e)))?;
        Ok(cube.schema().description().map(|s| s.to_string()))
    }

    /// Get cube statistics
    ///
    /// Returns:
    ///     Dictionary with statistics including row_count, partition_count,
    ///     memory_bytes, and column_stats
    fn statistics<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let cube = self.cube.lock()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Lock error: {}", e)))?;

        let stats = cube.statistics();
        let dict = pyo3::types::PyDict::new(py);

        dict.set_item("row_count", stats.row_count)?;
        dict.set_item("partition_count", stats.partition_count)?;
        dict.set_item("avg_rows_per_partition", stats.avg_rows_per_partition)?;
        dict.set_item("memory_bytes", stats.memory_bytes)?;
        dict.set_item("memory_mb", stats.memory_bytes as f64 / 1_048_576.0)?;

        // Add column statistics
        let col_stats_list = pyo3::types::PyList::empty(py);
        for col_stat in &stats.column_stats {
            let col_dict = pyo3::types::PyDict::new(py);
            col_dict.set_item("column_index", col_stat.column_index)?;
            col_dict.set_item("column_name", &col_stat.column_name)?;
            col_dict.set_item("null_count", col_stat.null_count)?;
            col_dict.set_item("null_percentage", col_stat.null_percentage)?;
            col_dict.set_item("distinct_count", col_stat.distinct_count)?;
            col_stats_list.append(col_dict)?;
        }
        dict.set_item("column_stats", col_stats_list)?;

        Ok(dict)
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

    /// Skip a number of rows (offset)
    ///
    /// # Arguments
    /// * `count` - Number of rows to skip
    ///
    /// # Example
    /// ```python
    /// query.offset(50)  # Skip first 50 rows
    /// ```
    fn offset(&mut self, count: usize) -> PyResult<()> {
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        self.builder = Some(builder.offset(count));
        Ok(())
    }

    /// OLAP Operation: Slice - filter on a single dimension
    ///
    /// # Arguments
    /// * `dimension` - Dimension name to filter on
    /// * `value` - Value to filter for
    ///
    /// # Example
    /// ```python
    /// query.slice("region", "North")
    /// ```
    fn slice(&mut self, dimension: String, value: String) -> PyResult<()> {
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        self.builder = Some(builder.slice(dimension, value));
        Ok(())
    }

    /// OLAP Operation: Dice - filter on multiple dimensions
    ///
    /// # Arguments
    /// * `filters` - List of (dimension, value) tuples to filter on
    ///
    /// # Example
    /// ```python
    /// query.dice([("region", "North"), ("product", "Widget")])
    /// ```
    fn dice(&mut self, filters: Vec<(String, String)>) -> PyResult<()> {
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        // Convert Vec<(String, String)> to &[(impl AsRef<str>, impl AsRef<str>)]
        let filter_refs: Vec<(&str, &str)> = filters
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        self.builder = Some(builder.dice(&filter_refs));
        Ok(())
    }

    /// OLAP Operation: Drill-down - navigate down a hierarchy
    ///
    /// # Arguments
    /// * `parent_level` - Parent level name (for reference)
    /// * `child_levels` - List of child level names to drill down to
    ///
    /// # Example
    /// ```python
    /// query.drill_down("year", ["year", "quarter", "month"])
    /// ```
    fn drill_down(&mut self, parent_level: String, child_levels: Vec<String>) -> PyResult<()> {
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        let level_refs: Vec<&str> = child_levels.iter().map(|s| s.as_str()).collect();
        self.builder = Some(builder.drill_down(parent_level, &level_refs));
        Ok(())
    }

    /// OLAP Operation: Roll-up - aggregate across dimensions
    ///
    /// # Arguments
    /// * `dimensions_to_remove` - List of dimension names to remove from grouping
    ///
    /// # Example
    /// ```python
    /// query.roll_up(["region"])  # Aggregate across all regions
    /// ```
    fn roll_up(&mut self, dimensions_to_remove: Vec<String>) -> PyResult<()> {
        let builder = self.builder.take().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Builder already consumed")
        })?;

        let dim_refs: Vec<&str> = dimensions_to_remove.iter().map(|s| s.as_str()).collect();
        self.builder = Some(builder.roll_up(&dim_refs));
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

/// Normalize PyArrow table schema to handle common type mismatches
///
/// Handles:
/// - large_utf8 → utf8
/// - large_binary → binary
/// - timezone-aware timestamps → timezone-naive (with warning)
/// - large_list → list (recursively)
///
/// Returns the normalized table or the original if no normalization needed.
fn normalize_arrow_schema<'py>(
    py: Python<'py>,
    arrow_table: Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyAny>> {
    // Import pyarrow
    let pyarrow = py.import("pyarrow")
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyImportError, _>(
            format!("Failed to import pyarrow: {}", e)
        ))?;

    // Get the schema
    let schema = arrow_table.getattr("schema")?;
    let fields = schema.getattr("names")?;
    let field_names: Vec<String> = fields.extract()?;

    // Check if normalization is needed
    let mut needs_normalization = false;
    let mut new_fields = Vec::new();

    for field_name in &field_names {
        let field = schema.call_method1("field", (field_name,))?;
        let field_type = field.getattr("type")?;
        let type_str = field_type.call_method0("__str__")?.extract::<String>()?;

        // Check if this field needs normalization
        let normalized_type = if type_str == "large_string" || type_str == "large_utf8" {
            needs_normalization = true;
            Some(pyarrow.call_method0("utf8")?)
        } else if type_str == "large_binary" {
            needs_normalization = true;
            Some(pyarrow.call_method0("binary")?)
        } else if type_str.starts_with("timestamp[") && type_str.contains("tz=") {
            // Handle timezone-aware timestamps
            needs_normalization = true;
            // Warn about lossy conversion
            let warnings = py.import("warnings")?;
            warnings.call_method1(
                "warn",
                (
                    "Converting timezone-aware timestamp to naive UTC. This may cause data loss. \
                     Recommended: convert to UTC before loading.",

                )
            )?;
            Some(pyarrow.call_method1("timestamp", ("us",))?)
        } else if type_str.starts_with("large_list<") {
            needs_normalization = true;
            // For large_list, we need to extract the value type and create a standard list
            let value_type = field_type.call_method0("value_type")?;
            Some(pyarrow.call_method1("list_", (value_type,))?)
        } else {
            None
        };

        if let Some(norm_type) = normalized_type {
            let nullable = field.getattr("nullable")?.extract::<bool>()?;
            let kwargs = [("nullable", nullable)].into_py_dict(py)?;
            let new_field = pyarrow.call_method(
                "field",
                (field_name, norm_type),
                Some(&kwargs)
            )?;
            new_fields.push(new_field);
        } else {
            new_fields.push(field);
        }
    }

    // If no normalization needed, return original table
    if !needs_normalization {
        return Ok(arrow_table);
    }

    // Create new schema
    let new_schema = pyarrow.call_method1("schema", (new_fields,))?;

    // Cast the table to the new schema
    let normalized_table = arrow_table.call_method1("cast", (new_schema,))?;

    Ok(normalized_table)
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
