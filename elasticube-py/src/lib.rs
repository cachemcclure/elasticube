//! Python bindings for ElastiCube
//!
//! This module provides Python access to the high-performance ElastiCube library
//! built in Rust using Apache Arrow and DataFusion.

use pyo3::prelude::*;
use pyo3::types::PyBytes;

use elasticube_core::{AggFunc, ElastiCube, ElastiCubeBuilder};
use arrow::datatypes::DataType;
use arrow::ipc::writer::StreamWriter;
use std::sync::Arc;

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

        // Wrap in Arc and PyElastiCube
        Python::attach(|py| {
            Py::new(py, PyElastiCube {
                cube: Arc::new(cube),
            })
        })
    }
}

/// Python wrapper for ElastiCube
#[pyclass]
struct PyElastiCube {
    cube: Arc<ElastiCube>,
}

#[pymethods]
impl PyElastiCube {
    /// Create a query builder
    fn query(&self) -> PyResult<PyQueryBuilder> {
        let query_builder = self.cube.clone().query()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(PyQueryBuilder {
            builder: Some(query_builder),
        })
    }

    /// Get cube name
    fn name(&self) -> String {
        self.cube.schema().name().to_string()
    }

    /// Get number of rows
    fn row_count(&self) -> usize {
        self.cube.row_count()
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
