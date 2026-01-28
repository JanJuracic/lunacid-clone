//! Error types for world data loading.

use thiserror::Error;

/// Errors that can occur when loading level or palette data.
#[derive(Debug, Error)]
pub enum DataLoadError {
    /// File could not be found.
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// File could not be read.
    #[error("Failed to read file '{path}': {details}")]
    ReadError { path: String, details: String },

    /// RON parsing failed.
    #[error("Parse error in '{path}': {details}")]
    ParseError { path: String, details: String },

    /// Grid dimensions don't match between layers.
    #[error("Grid mismatch: expected {expected_width}x{expected_height}, got {actual_width}x{actual_height}")]
    GridMismatch {
        expected_width: usize,
        expected_height: usize,
        actual_width: usize,
        actual_height: usize,
    },

    /// Invalid palette reference.
    #[error("Unknown palette entry '{character}' at position ({x}, {z})")]
    UnknownPaletteEntry { character: char, x: usize, z: usize },
}
