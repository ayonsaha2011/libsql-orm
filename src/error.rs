//! Error handling for libsql-orm
//!
//! This module provides comprehensive error types and handling for all operations
//! within the libsql-orm library. All errors implement the standard `Error` trait
//! and provide detailed error messages for debugging.
//!
//! # Error Categories
//!
//! - **Connection Errors**: Database connection failures
//! - **SQL Errors**: Query execution problems  
//! - **Serialization Errors**: Data conversion issues
//! - **Validation Errors**: Data validation failures
//! - **Not Found Errors**: Resource not found
//! - **Pagination Errors**: Pagination parameter issues
//! - **Query Errors**: Query building problems
//!
//! # Examples
//!
//! ```rust
//! use libsql_orm::{Error, Result};
//!
//! fn handle_error(result: Result<String>) {
//!     match result {
//!         Ok(value) => println!("Success: {}", value),
//!         Err(Error::NotFound(msg)) => println!("Resource not found: {}", msg),
//!         Err(Error::Validation(msg)) => println!("Validation failed: {}", msg),
//!         Err(e) => println!("Other error: {}", e),
//!     }
//! }
//! ```

use std::fmt;

/// Custom error type for the libsql-orm crate
///
/// Provides comprehensive error handling for all database and ORM operations.
/// All variants include descriptive messages to aid in debugging and error handling.
#[derive(Debug)]
pub enum Error {
    /// Database connection error
    Connection(String),
    /// SQL execution error
    Sql(String),
    /// Serialization/deserialization error
    Serialization(String),
    /// Validation error
    Validation(String),
    /// Not found error
    NotFound(String),
    /// Pagination error
    Pagination(String),
    /// Query building error
    Query(String),
    /// Worker environment error
    AnyhowError(String),
    /// Database error
    DatabaseError(String),
    /// Generic error
    Generic(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Connection(msg) => write!(f, "Connection error: {msg}"),
            Error::Sql(msg) => write!(f, "SQL error: {msg}"),
            Error::Serialization(msg) => write!(f, "Serialization error: {msg}"),
            Error::Validation(msg) => write!(f, "Validation error: {msg}"),
            Error::NotFound(msg) => write!(f, "Not found: {msg}"),
            Error::Pagination(msg) => write!(f, "Pagination error: {msg}"),
            Error::Query(msg) => write!(f, "Query error: {msg}"),
            Error::AnyhowError(msg) => write!(f, "Anyhow error: {msg}"),
            Error::DatabaseError(msg) => write!(f, "Database error: {msg}"),
            Error::Generic(msg) => write!(f, "Error: {msg}"),
        }
    }
}

impl From<libsql::Error> for Error {
    fn from(err: libsql::Error) -> Self {
        Error::Sql(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Generic(err.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Error::Generic(err.to_string())
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::AnyhowError(err.to_string())
    }
}

/// Result type alias for the crate
pub type Result<T> = std::result::Result<T, Error>;
