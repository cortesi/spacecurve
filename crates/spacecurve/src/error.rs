//! Error types for the `spacecurve` crate.

use std::result::Result as StdResult;

use thiserror::Error;

/// Error variants for operations in the `spacecurve` crate.
#[derive(Debug, Error)]
pub enum Error {
    /// Errors related to dimensionality or dimensional constraints.
    #[error("Shape error: {0}")]
    Shape(String),
    /// Errors where size exceeds limits or constraints.
    #[error("Size error: {0}")]
    Size(String),
    /// Unknown pattern or identifier error.
    #[error("Unknown: {0}")]
    Unknown(String),
    /// Other miscellaneous error.
    #[error("{0}")]
    Other(String),
}

/// Convenient result type used throughout the crate.
pub type Result<T> = StdResult<T, Error>;
