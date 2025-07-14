//! Filtering and search functionality for libsql-orm
//!
//! This module provides comprehensive filtering capabilities including simple comparisons,
//! complex logical combinations, text search, and sorting. It supports building complex
//! WHERE clauses programmatically with type safety.
//!
//! # Basic Filtering
//!
//! ```rust
//! use libsql_orm::{Filter, FilterOperator, Value};
//!
//! // Simple equality filter
//! let filter = Filter::eq("status", "active");
//!
//! // Comparison filters
//! let age_filter = Filter::gt("age", 18i64);
//! let name_filter = Filter::like("name", "%john%");
//!
//! // Convert to FilterOperator for use in queries
//! let filter_op = FilterOperator::Single(age_filter);
//! ```
//!
//! # Complex Filtering
//!
//! ```rust
//! use libsql_orm::{Filter, FilterOperator};
//!
//! // AND combination
//! let complex_filter = FilterOperator::And(vec![
//!     FilterOperator::Single(Filter::eq("is_active", true)),
//!     FilterOperator::Single(Filter::gt("age", 18i64)),
//!     FilterOperator::Single(Filter::like("email", "%@company.com")),
//! ]);
//!
//! // OR combination
//! let or_filter = FilterOperator::Or(vec![
//!     FilterOperator::Single(Filter::eq("role", "admin")),
//!     FilterOperator::Single(Filter::eq("role", "moderator")),
//! ]);
//!
//! // Nested combinations
//! let nested = FilterOperator::And(vec![
//!     complex_filter,
//!     FilterOperator::Not(Box::new(or_filter)),
//! ]);
//! ```
//!
//! # Text Search
//!
//! ```rust
//! use libsql_orm::SearchFilter;
//!
//! // Search across multiple fields
//! let search = SearchFilter::new("john", vec!["name", "email"])
//!     .case_sensitive(false)
//!     .exact_match(false);
//!
//! let filter_op = search.to_filter_operator();
//! ```
//!
//! # Sorting
//!
//! ```rust
//! use libsql_orm::{Sort, SortOrder};
//!
//! let sorts = vec![
//!     Sort::new("created_at", SortOrder::Desc),
//!     Sort::new("name", SortOrder::Asc),
//! ];
//! ```

use crate::{Operator, Value};
use serde::{Deserialize, Serialize};

/// Filter operator for building complex queries
///
/// Represents a complete filter expression that can be a single condition,
/// a logical combination of multiple conditions, or a custom SQL condition.
///
/// # Examples
///
/// ```rust
/// use libsql_orm::{FilterOperator, Filter};
///
/// // Single condition
/// let single = FilterOperator::Single(Filter::eq("status", "active"));
///
/// // Multiple conditions with AND
/// let and_filter = FilterOperator::And(vec![
///     FilterOperator::Single(Filter::eq("is_active", true)),
///     FilterOperator::Single(Filter::gt("age", 18i64)),
/// ]);
///
/// // Multiple conditions with OR
/// let or_filter = FilterOperator::Or(vec![
///     FilterOperator::Single(Filter::eq("role", "admin")),
///     FilterOperator::Single(Filter::eq("role", "user")),
/// ]);
///
/// // Negation
/// let not_filter = FilterOperator::Not(Box::new(single));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    /// Single condition
    Single(Filter),
    /// AND combination of filters
    And(Vec<FilterOperator>),
    /// OR combination of filters
    Or(Vec<FilterOperator>),
    /// NOT filter
    Not(Box<FilterOperator>),
    /// Custom SQL condition
    Custom(String),
}

/// Individual filter condition
///
/// Represents a single comparison operation between a column and a value.
/// Provides convenient constructor methods for common comparison operations.
///
/// # Examples
///
/// ```rust
/// use libsql_orm::{Filter, Value};
///
/// // Basic comparisons
/// let eq_filter = Filter::eq("status", "active");
/// let gt_filter = Filter::gt("age", 18i64);
/// let like_filter = Filter::like("name", "%john%");
///
/// // Range and list operations
/// let between_filter = Filter::between("score", 80, 100);
/// let in_filter = Filter::in_values("role", vec!["admin", "user"]);
/// let null_filter = Filter::is_null("deleted_at");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    /// Column name
    pub column: String,
    /// Operator
    pub operator: Operator,
    /// Value(s) to compare against
    pub value: FilterValue,
}

/// Filter value that can be a single value or multiple values
///
/// Supports different value types for various SQL operations:
/// - Single values for basic comparisons (=, >, <, etc.)
/// - Multiple values for IN/NOT IN operations  
/// - Range values for BETWEEN/NOT BETWEEN operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterValue {
    /// Single value
    Single(Value),
    /// Multiple values (for IN, NOT IN operators)
    Multiple(Vec<Value>),
    /// Range values (for BETWEEN, NOT BETWEEN operators)
    Range(Value, Value),
}

impl Filter {
    /// Create a new filter
    pub fn new(column: impl Into<String>, operator: Operator, value: FilterValue) -> Self {
        Self {
            column: column.into(),
            operator,
            value,
        }
    }

    /// Create a new filter with a simple value
    pub fn new_simple(
        column: impl Into<String>,
        operator: Operator,
        value: impl Into<Value>,
    ) -> Self {
        Self {
            column: column.into(),
            operator,
            value: FilterValue::Single(value.into()),
        }
    }

    /// Create an equality filter
    pub fn eq(column: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::new(column, Operator::Eq, FilterValue::Single(value.into()))
    }

    /// Create a not-equal filter
    pub fn ne(column: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::new(column, Operator::Ne, FilterValue::Single(value.into()))
    }

    /// Create a less-than filter
    pub fn lt(column: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::new(column, Operator::Lt, FilterValue::Single(value.into()))
    }

    /// Create a less-than-or-equal filter
    pub fn le(column: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::new(column, Operator::Le, FilterValue::Single(value.into()))
    }

    /// Create a greater-than filter
    pub fn gt(column: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::new(column, Operator::Gt, FilterValue::Single(value.into()))
    }

    /// Create a greater-than-or-equal filter
    pub fn ge(column: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::new(column, Operator::Ge, FilterValue::Single(value.into()))
    }

    /// Create a LIKE filter
    pub fn like(column: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self::new(
            column,
            Operator::Like,
            FilterValue::Single(Value::Text(pattern.into())),
        )
    }

    /// Create a NOT LIKE filter
    pub fn not_like(column: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self::new(
            column,
            Operator::NotLike,
            FilterValue::Single(Value::Text(pattern.into())),
        )
    }

    /// Create an IN filter
    pub fn in_values(column: impl Into<String>, values: Vec<impl Into<Value>>) -> Self {
        let values = values.into_iter().map(|v| v.into()).collect();
        Self::new(column, Operator::In, FilterValue::Multiple(values))
    }

    /// Create a NOT IN filter
    pub fn not_in_values(column: impl Into<String>, values: Vec<impl Into<Value>>) -> Self {
        let values = values.into_iter().map(|v| v.into()).collect();
        Self::new(column, Operator::NotIn, FilterValue::Multiple(values))
    }

    /// Create an IS NULL filter
    pub fn is_null(column: impl Into<String>) -> Self {
        Self::new(column, Operator::IsNull, FilterValue::Single(Value::Null))
    }

    /// Create an IS NOT NULL filter
    pub fn is_not_null(column: impl Into<String>) -> Self {
        Self::new(
            column,
            Operator::IsNotNull,
            FilterValue::Single(Value::Null),
        )
    }

    /// Create a BETWEEN filter
    pub fn between(
        column: impl Into<String>,
        min: impl Into<Value>,
        max: impl Into<Value>,
    ) -> Self {
        Self::new(
            column,
            Operator::Between,
            FilterValue::Range(min.into(), max.into()),
        )
    }

    /// Create a NOT BETWEEN filter
    pub fn not_between(
        column: impl Into<String>,
        min: impl Into<Value>,
        max: impl Into<Value>,
    ) -> Self {
        Self::new(
            column,
            Operator::NotBetween,
            FilterValue::Range(min.into(), max.into()),
        )
    }
}

impl FilterOperator {
    /// Create an AND filter
    pub fn and(filters: Vec<FilterOperator>) -> Self {
        FilterOperator::And(filters)
    }

    /// Create an OR filter
    pub fn or(filters: Vec<FilterOperator>) -> Self {
        FilterOperator::Or(filters)
    }

    /// Create a NOT filter
    pub fn negate(filter: FilterOperator) -> Self {
        FilterOperator::Not(Box::new(filter))
    }

    /// Add a filter to an AND group
    pub fn and_with(self, other: FilterOperator) -> Self {
        match self {
            FilterOperator::And(mut filters) => {
                filters.push(other);
                FilterOperator::And(filters)
            }
            _ => FilterOperator::And(vec![self, other]),
        }
    }

    /// Add a filter to an OR group
    pub fn or_with(self, other: FilterOperator) -> Self {
        match self {
            FilterOperator::Or(mut filters) => {
                filters.push(other);
                FilterOperator::Or(filters)
            }
            _ => FilterOperator::Or(vec![self, other]),
        }
    }
}

impl std::ops::Not for FilterOperator {
    type Output = Self;

    fn not(self) -> Self::Output {
        FilterOperator::Not(Box::new(self))
    }
}

/// Search filter for text-based searches
///
/// Provides flexible text search capabilities across one or more columns
/// with options for case sensitivity and exact matching.
///
/// # Examples
///
/// ```rust
/// use libsql_orm::SearchFilter;
///
/// // Basic search across multiple fields
/// let search = SearchFilter::new("john", vec!["name", "email"]);
///
/// // Case-sensitive exact match
/// let exact_search = SearchFilter::new("John Doe", vec!["full_name"])
///     .case_sensitive(true)
///     .exact_match(true);
///
/// // Convert to filter for use in queries
/// let filter_op = search.to_filter_operator();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilter {
    /// Search query
    pub query: String,
    /// Columns to search in
    pub columns: Vec<String>,
    /// Whether to use case-sensitive search
    pub case_sensitive: bool,
    /// Whether to use exact match
    pub exact_match: bool,
}

impl SearchFilter {
    /// Create a new search filter
    pub fn new(query: impl Into<String>, columns: Vec<impl Into<String>>) -> Self {
        Self {
            query: query.into(),
            columns: columns.into_iter().map(|c| c.into()).collect(),
            case_sensitive: false,
            exact_match: false,
        }
    }

    /// Set case sensitivity
    pub fn case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }

    /// Set exact match
    pub fn exact_match(mut self, exact_match: bool) -> Self {
        self.exact_match = exact_match;
        self
    }

    /// Convert to FilterOperator
    pub fn to_filter_operator(&self) -> FilterOperator {
        let mut filters = Vec::new();

        for column in &self.columns {
            let filter = if self.exact_match {
                Filter::eq(column, &*self.query)
            } else {
                Filter::like(column, format!("%{}%", self.query))
            };
            filters.push(FilterOperator::Single(filter));
        }

        if filters.len() == 1 {
            filters.pop().unwrap()
        } else {
            FilterOperator::Or(filters)
        }
    }

    /// Create a new search filter for a single field
    pub fn new_single_field(field: impl Into<String>, query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            columns: vec![field.into()],
            case_sensitive: false,
            exact_match: false,
        }
    }

    /// Create a new search filter for multiple fields
    pub fn new_multiple_fields(fields: Vec<impl Into<String>>, query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            columns: fields.into_iter().map(|f| f.into()).collect(),
            case_sensitive: false,
            exact_match: false,
        }
    }

    /// Convert to FilterOperator with improved search logic
    pub fn to_filter_operator_improved(&self) -> FilterOperator {
        let mut filters = Vec::new();

        for column in &self.columns {
            let pattern = if self.exact_match {
                self.query.clone()
            } else {
                format!("%{}%", self.query)
            };

            let filter = if self.case_sensitive {
                Filter::like(column.clone(), pattern)
            } else {
                // For case-insensitive search, we'll use LOWER() function
                // This will be handled in the query builder
                Filter::like(column.clone(), pattern)
            };

            filters.push(FilterOperator::Single(filter));
        }

        if filters.len() == 1 {
            filters.pop().unwrap()
        } else {
            FilterOperator::Or(filters)
        }
    }
}

/// Sort specification
///
/// Defines how query results should be sorted by column and order.
/// Multiple sort specifications can be combined to create complex sorting.
///
/// # Examples
///
/// ```rust
/// use libsql_orm::{Sort, SortOrder};
///
/// // Single column sorts
/// let name_asc = Sort::asc("name");
/// let date_desc = Sort::desc("created_at");
///
/// // Multiple column sorting
/// let sorts = vec![
///     Sort::new("priority", SortOrder::Desc),
///     Sort::new("name", SortOrder::Asc),
/// ];
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sort {
    /// Column name
    pub column: String,
    /// Sort order
    pub order: crate::SortOrder,
}

impl Sort {
    /// Create a new sort specification
    pub fn new(column: impl Into<String>, order: crate::SortOrder) -> Self {
        Self {
            column: column.into(),
            order,
        }
    }

    /// Create a new sort with boolean flag for ascending
    pub fn new_bool(column: impl Into<String>, ascending: bool) -> Self {
        Self {
            column: column.into(),
            order: if ascending {
                crate::SortOrder::Asc
            } else {
                crate::SortOrder::Desc
            },
        }
    }

    /// Create an ascending sort
    pub fn asc(column: impl Into<String>) -> Self {
        Self::new(column, crate::SortOrder::Asc)
    }

    /// Create a descending sort
    pub fn desc(column: impl Into<String>) -> Self {
        Self::new(column, crate::SortOrder::Desc)
    }
}
