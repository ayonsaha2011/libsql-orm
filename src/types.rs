//! Type definitions for libsql-orm
//!
//! This module contains core type definitions used throughout the library,
//! including database values, operators, sort orders, and aggregate functions.
//!
//! # Core Types
//!
//! - [`Value`] - Represents any database value with automatic type conversion
//! - [`Row`] - Type alias for a database row (HashMap of column names to values)
//! - [`SortOrder`] - Ascending or descending sort order
//! - [`Aggregate`] - SQL aggregate functions (COUNT, SUM, AVG, etc.)
//! - [`JoinType`] - SQL join types (INNER, LEFT, RIGHT, FULL)
//! - [`Operator`] - SQL comparison operators
//!
//! # Examples
//!
//! ```rust
//! use libsql_orm::{Value, SortOrder, Aggregate};
//!
//! // Creating values
//! let text_value = Value::from("hello");
//! let int_value = Value::from(42i64);
//! let bool_value = Value::from(true);
//!
//! // Using in queries
//! let sort = SortOrder::Desc;
//! let agg = Aggregate::Count;
//! ```

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// Represents a database row as a map of column names to values
///
/// This type alias provides a convenient way to work with database rows
/// as key-value pairs where keys are column names and values are database values.
pub type Row = HashMap<String, Value>;

/// Represents a database value that can be serialized/deserialized
///
/// The `Value` enum covers all possible SQLite/libsql data types and provides
/// automatic conversion from common Rust types. It supports JSON serialization
/// for easy data exchange.
///
/// # Examples
///
/// ```rust
/// use libsql_orm::Value;
///
/// let null_val = Value::Null;
/// let int_val = Value::Integer(42);
/// let text_val = Value::Text("hello".to_string());
/// let bool_val = Value::Boolean(true);
///
/// // Automatic conversion
/// let converted: Value = "hello".into();
/// let converted: Value = 42i64.into();
/// let converted: Value = true.into();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
    Boolean(bool),
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Integer(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Real(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::Text(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::Text(v.to_string())
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Boolean(v)
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Self {
        Value::Blob(v)
    }
}

impl From<Option<String>> for Value {
    fn from(v: Option<String>) -> Self {
        match v {
            Some(s) => Value::Text(s),
            None => Value::Null,
        }
    }
}

impl From<Option<i64>> for Value {
    fn from(v: Option<i64>) -> Self {
        match v {
            Some(i) => Value::Integer(i),
            None => Value::Null,
        }
    }
}

impl From<Option<f64>> for Value {
    fn from(v: Option<f64>) -> Self {
        match v {
            Some(f) => Value::Real(f),
            None => Value::Null,
        }
    }
}

impl From<Option<bool>> for Value {
    fn from(v: Option<bool>) -> Self {
        match v {
            Some(b) => Value::Boolean(b),
            None => Value::Null,
        }
    }
}

impl From<Option<Vec<u8>>> for Value {
    fn from(v: Option<Vec<u8>>) -> Self {
        match v {
            Some(b) => Value::Blob(b),
            None => Value::Null,
        }
    }
}

impl From<serde_json::Value> for Value {
    fn from(v: serde_json::Value) -> Self {
        match v {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Boolean(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Real(f)
                } else {
                    Value::Text(n.to_string())
                }
            }
            serde_json::Value::String(s) => Value::Text(s),
            serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                Value::Text(v.to_string())
            }
        }
    }
}

/// Sort order for queries
///
/// Specifies whether query results should be sorted in ascending or descending order.
///
/// # Examples
///
/// ```rust
/// use libsql_orm::{SortOrder, Sort};
///
/// let asc_sort = Sort::new("name", SortOrder::Asc);
/// let desc_sort = Sort::new("created_at", SortOrder::Desc);
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}

impl std::fmt::Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortOrder::Asc => write!(f, "ASC"),
            SortOrder::Desc => write!(f, "DESC"),
        }
    }
}

/// Aggregate functions
///
/// SQL aggregate functions for performing calculations on sets of values.
///
/// # Examples
///
/// ```rust
/// use libsql_orm::{Model, Aggregate};
///
/// // Count all users
/// let count = User::aggregate(Aggregate::Count, "*", None, &db).await?;
///
/// // Average age
/// let avg_age = User::aggregate(Aggregate::Avg, "age", None, &db).await?;
///
/// // Maximum salary
/// let max_salary = User::aggregate(Aggregate::Max, "salary", None, &db).await?;
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Aggregate {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

impl std::fmt::Display for Aggregate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Aggregate::Count => write!(f, "COUNT"),
            Aggregate::Sum => write!(f, "SUM"),
            Aggregate::Avg => write!(f, "AVG"),
            Aggregate::Min => write!(f, "MIN"),
            Aggregate::Max => write!(f, "MAX"),
        }
    }
}

/// Join types for queries
///
/// SQL join types for combining data from multiple tables.
///
/// # Examples
///
/// ```rust
/// use libsql_orm::JoinType;
///
/// let inner = JoinType::Inner; // INNER JOIN
/// let left = JoinType::Left;   // LEFT JOIN
/// let right = JoinType::Right; // RIGHT JOIN
/// let full = JoinType::Full;   // FULL JOIN
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

impl std::fmt::Display for JoinType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JoinType::Inner => write!(f, "INNER JOIN"),
            JoinType::Left => write!(f, "LEFT JOIN"),
            JoinType::Right => write!(f, "RIGHT JOIN"),
            JoinType::Full => write!(f, "FULL JOIN"),
        }
    }
}

/// SQL operator types
///
/// Comparison and logical operators for use in WHERE clauses and filters.
///
/// # Examples
///
/// ```rust
/// use libsql_orm::{Operator, FilterOperator, Value};
///
/// // Equal comparison
/// let filter = FilterOperator::Eq("status".to_string(), Value::Text("active".to_string()));
///
/// // Greater than
/// let filter = FilterOperator::Gt("age".to_string(), Value::Integer(18));
///
/// // LIKE pattern matching
/// let filter = FilterOperator::Like("name".to_string(), Value::Text("%john%".to_string()));
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Operator {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Like,
    NotLike,
    In,
    NotIn,
    IsNull,
    IsNotNull,
    Between,
    NotBetween,
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::Eq => write!(f, "="),
            Operator::Ne => write!(f, "!="),
            Operator::Lt => write!(f, "<"),
            Operator::Le => write!(f, "<="),
            Operator::Gt => write!(f, ">"),
            Operator::Ge => write!(f, ">="),
            Operator::Like => write!(f, "LIKE"),
            Operator::NotLike => write!(f, "NOT LIKE"),
            Operator::In => write!(f, "IN"),
            Operator::NotIn => write!(f, "NOT IN"),
            Operator::IsNull => write!(f, "IS NULL"),
            Operator::IsNotNull => write!(f, "IS NOT NULL"),
            Operator::Between => write!(f, "BETWEEN"),
            Operator::NotBetween => write!(f, "NOT BETWEEN"),
        }
    }
}

/// Custom deserializer for boolean fields that handles SQLite integer conversion
///
/// SQLite stores boolean values as integers (0 for false, 1 for true).
/// This deserializer automatically converts these integers to proper Rust boolean types.
///
/// # Usage
///
/// ```rust
/// use libsql_orm::deserialize_bool;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct User {
///     pub id: Option<i64>,
///     pub name: String,
///     #[serde(deserialize_with = "deserialize_bool")]
///     pub is_active: bool,
/// }
/// ```
pub fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Bool(b) => Ok(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i != 0)
            } else if let Some(f) = n.as_f64() {
                Ok(f != 0.0)
            } else {
                Err(Error::custom("Invalid number format for boolean"))
            }
        }
        serde_json::Value::String(s) => match s.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Ok(true),
            "false" | "0" | "no" | "off" => Ok(false),
            _ => Err(Error::custom(format!(
                "Invalid string value for boolean: {s}"
            ))),
        },
        _ => Err(Error::custom("Expected boolean, integer, or string")),
    }
}
