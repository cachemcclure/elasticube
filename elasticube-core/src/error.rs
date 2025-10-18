//! Error types for ElastiCube

use thiserror::Error;

/// Result type alias for ElastiCube operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types that can occur during ElastiCube operations
#[derive(Error, Debug)]
pub enum Error {
    /// Arrow-related errors
    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),

    /// DataFusion-related errors
    #[error("DataFusion error: {0}")]
    DataFusion(#[from] datafusion::error::DataFusionError),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Schema validation errors
    #[error("Schema error: {0}")]
    Schema(String),

    /// Dimension-related errors
    #[error("Dimension error: {0}")]
    Dimension(String),

    /// Measure-related errors
    #[error("Measure error: {0}")]
    Measure(String),

    /// Hierarchy-related errors
    #[error("Hierarchy error: {0}")]
    Hierarchy(String),

    /// Query-related errors
    #[error("Query error: {0}")]
    Query(String),

    /// Data source errors
    #[error("Data source error: {0}")]
    DataSource(String),

    /// Type conversion errors
    #[error("Type conversion error: {0}")]
    TypeConversion(String),

    /// Invalid configuration
    #[error("Configuration error: {0}")]
    Config(String),

    /// Builder-related errors
    #[error("Builder error: {0}")]
    Builder(String),

    /// Data loading errors
    #[error("Data error: {0}")]
    Data(String),

    /// Generic error with custom message
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Create a schema error
    pub fn schema(msg: impl Into<String>) -> Self {
        Error::Schema(msg.into())
    }

    /// Create a dimension error
    pub fn dimension(msg: impl Into<String>) -> Self {
        Error::Dimension(msg.into())
    }

    /// Create a measure error
    pub fn measure(msg: impl Into<String>) -> Self {
        Error::Measure(msg.into())
    }

    /// Create a hierarchy error
    pub fn hierarchy(msg: impl Into<String>) -> Self {
        Error::Hierarchy(msg.into())
    }

    /// Create a query error
    pub fn query(msg: impl Into<String>) -> Self {
        Error::Query(msg.into())
    }

    /// Create a data source error
    pub fn data_source(msg: impl Into<String>) -> Self {
        Error::DataSource(msg.into())
    }

    /// Create a configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Error::Config(msg.into())
    }

    /// Create a builder error
    pub fn builder(msg: impl Into<String>) -> Self {
        Error::Builder(msg.into())
    }

    /// Create a data error
    pub fn data(msg: impl Into<String>) -> Self {
        Error::Data(msg.into())
    }

    /// Create an arrow error
    pub fn arrow(msg: impl Into<String>) -> Self {
        Error::Arrow(arrow::error::ArrowError::ExternalError(
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, msg.into()))
        ))
    }

    /// Create an IO error
    pub fn io(msg: impl Into<String>) -> Self {
        Error::Io(std::io::Error::new(std::io::ErrorKind::Other, msg.into()))
    }
}
