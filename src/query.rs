//! Query building and execution for libsql-orm
//!
//! This module provides a fluent query builder for constructing complex SQL queries
//! with type safety and parameter binding. It supports SELECT, INSERT, UPDATE, DELETE
//! operations with joins, filtering, sorting, grouping, and aggregation.
//!
//! # Basic Usage
//!
//! ```rust
//! use libsql_orm::{QueryBuilder, FilterOperator, Sort, SortOrder};
//!
//! let query = QueryBuilder::new("users")
//!     .select(vec!["id", "name", "email"])
//!     .r#where(FilterOperator::Eq("is_active".to_string(), Value::Boolean(true)))
//!     .order_by(Sort::new("name", SortOrder::Asc))
//!     .limit(10);
//!
//! let (sql, params) = query.build()?;
//! ```
//!
//! # Complex Queries
//!
//! ```rust
//! use libsql_orm::{QueryBuilder, JoinType, FilterOperator, Aggregate};
//!
//! let complex_query = QueryBuilder::new("orders")
//!     .select(vec!["orders.id", "users.name", "products.title"])
//!     .join(JoinType::Inner, "users", "users.id = orders.user_id")
//!     .join(JoinType::Inner, "products", "products.id = orders.product_id")
//!     .r#where(FilterOperator::Gte("orders.created_at".to_string(), Value::Text("2024-01-01".to_string())))
//!     .group_by(vec!["users.id"])
//!     .aggregate(Aggregate::Count, "orders.id", Some("order_count"))
//!     .order_by(Sort::desc("order_count"));
//!
//! let results = complex_query.execute::<OrderWithUser>(&db).await?;
//! ```

use crate::filters::FilterValue;
use crate::{
    Aggregate, Database, FilterOperator, Operator, PaginatedResult, Pagination, Result, Sort, Value,
};
use std::collections::HashMap;

/// Query result wrapper
///
/// Contains query results with optional total count for pagination support.
///
/// # Examples
///
/// ```rust
/// use libsql_orm::QueryResult;
///
/// let result = QueryResult::new(vec!["item1", "item2"]);
/// let result_with_total = QueryResult::with_total(vec!["item1", "item2"], 100);
/// ```
pub struct QueryResult<T> {
    pub data: Vec<T>,
    pub total: Option<u64>,
}

impl<T> QueryResult<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self { data, total: None }
    }

    pub fn with_total(data: Vec<T>, total: u64) -> Self {
        Self {
            data,
            total: Some(total),
        }
    }
}

/// SQL query builder for complex queries
///
/// Provides a fluent interface for building SQL queries with support for:
/// - Column selection and table joins
/// - WHERE clauses with complex filtering
/// - GROUP BY and HAVING clauses
/// - ORDER BY with multiple sort criteria
/// - LIMIT and OFFSET for pagination
/// - Aggregate functions (COUNT, SUM, AVG, etc.)
/// - DISTINCT queries
///
/// # Examples
///
/// ```rust
/// use libsql_orm::{QueryBuilder, FilterOperator, Sort, SortOrder, JoinType};
///
/// // Basic query
/// let query = QueryBuilder::new("users")
///     .select(vec!["id", "name", "email"])
///     .r#where(FilterOperator::Eq("is_active".to_string(), Value::Boolean(true)))
///     .order_by(Sort::new("name", SortOrder::Asc))
///     .limit(10);
///
/// // Query with joins
/// let joined_query = QueryBuilder::new("posts")
///     .select(vec!["posts.title", "users.name"])
///     .join(JoinType::Inner, "users", "users.id = posts.user_id")
///     .r#where(FilterOperator::Eq("posts.published".to_string(), Value::Boolean(true)));
///
/// // Aggregate query
/// let agg_query = QueryBuilder::new("orders")
///     .aggregate(Aggregate::Sum, "amount", Some("total_amount"))
///     .group_by(vec!["user_id"])
///     .having(FilterOperator::Gt("total_amount".to_string(), Value::Real(1000.0)));
/// ```
pub struct QueryBuilder {
    table: String,
    select_columns: Vec<String>,
    joins: Vec<JoinClause>,
    where_clauses: Vec<FilterOperator>,
    group_by: Vec<String>,
    having: Vec<FilterOperator>,
    order_by: Vec<Sort>,
    limit: Option<u32>,
    offset: Option<u32>,
    distinct: bool,
    aggregate: Option<AggregateClause>,
}

/// Join clause for complex queries
struct JoinClause {
    join_type: crate::JoinType,
    table: String,
    alias: Option<String>,
    condition: String,
}

/// Aggregate clause for aggregation queries
struct AggregateClause {
    function: Aggregate,
    column: String,
    alias: Option<String>,
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            select_columns: vec!["*".to_string()],
            joins: Vec::new(),
            where_clauses: Vec::new(),
            group_by: Vec::new(),
            having: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            distinct: false,
            aggregate: None,
        }
    }

    /// Select specific columns
    pub fn select(mut self, columns: Vec<impl Into<String>>) -> Self {
        self.select_columns = columns.into_iter().map(|c| c.into()).collect();
        self
    }

    /// Add a join clause
    pub fn join(
        mut self,
        join_type: crate::JoinType,
        table: impl Into<String>,
        condition: impl Into<String>,
    ) -> Self {
        self.joins.push(JoinClause {
            join_type,
            table: table.into(),
            alias: None,
            condition: condition.into(),
        });
        self
    }

    /// Add a join clause with alias
    pub fn join_as(
        mut self,
        join_type: crate::JoinType,
        table: impl Into<String>,
        alias: impl Into<String>,
        condition: impl Into<String>,
    ) -> Self {
        self.joins.push(JoinClause {
            join_type,
            table: table.into(),
            alias: Some(alias.into()),
            condition: condition.into(),
        });
        self
    }

    /// Add a where clause
    pub fn r#where(mut self, filter: FilterOperator) -> Self {
        self.where_clauses.push(filter);
        self
    }

    /// Add a group by clause
    pub fn group_by(mut self, columns: Vec<impl Into<String>>) -> Self {
        self.group_by = columns.into_iter().map(|c| c.into()).collect();
        self
    }

    /// Add a having clause
    pub fn having(mut self, filter: FilterOperator) -> Self {
        self.having.push(filter);
        self
    }

    /// Add an order by clause
    pub fn order_by(mut self, sort: Sort) -> Self {
        self.order_by.push(sort);
        self
    }

    /// Add multiple order by clauses
    pub fn order_by_multiple(mut self, sorts: Vec<Sort>) -> Self {
        self.order_by.extend(sorts);
        self
    }

    /// Set limit
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set distinct
    pub fn distinct(mut self, distinct: bool) -> Self {
        self.distinct = distinct;
        self
    }

    /// Set aggregate function
    pub fn aggregate(
        mut self,
        function: Aggregate,
        column: impl Into<String>,
        alias: Option<impl Into<String>>,
    ) -> Self {
        self.aggregate = Some(AggregateClause {
            function,
            column: column.into(),
            alias: alias.map(|a| a.into()),
        });
        self
    }

    /// Select all columns
    pub fn select_all(mut self) -> Self {
        self.select_columns = vec!["*".to_string()];
        self
    }

    /// Select specific columns
    pub fn select_columns(mut self, columns: &[&str]) -> Self {
        self.select_columns = columns.iter().map(|&c| c.to_string()).collect();
        self
    }

    /// Select a single column
    pub fn select_column(mut self, column: &str) -> Self {
        self.select_columns = vec![column.to_string()];
        self
    }

    /// Select count
    pub fn select_count(mut self) -> Self {
        self.select_columns = vec!["COUNT(*)".to_string()];
        self
    }

    /// Select aggregate
    pub fn select_aggregate(mut self, aggregate: &str) -> Self {
        self.select_columns = vec![aggregate.to_string()];
        self
    }

    /// Select distinct
    pub fn select_distinct(mut self, column: &str) -> Self {
        self.select_columns = vec![column.to_string()];
        self.distinct = true;
        self
    }

    /// Add where condition
    pub fn where_condition(
        mut self,
        condition: &str,
        _params: impl Into<Vec<libsql::Value>>,
    ) -> Self {
        // This is a simplified implementation - in a real implementation you'd parse the condition
        self.where_clauses
            .push(FilterOperator::Custom(condition.to_string()));
        self
    }

    /// Add search
    pub fn search(mut self, field: &str, query: &str) -> Self {
        let condition = format!("{field} LIKE '%{query}%'");
        self.where_clauses.push(FilterOperator::Custom(condition));
        self
    }

    /// Add filter
    pub fn with_filter(mut self, filter: crate::Filter) -> Self {
        // Convert Filter to FilterOperator::Single
        self.where_clauses.push(FilterOperator::Single(filter));
        self
    }

    /// Add filters
    pub fn with_filters(mut self, filters: Vec<crate::Filter>) -> Self {
        for filter in filters {
            self = self.with_filter(filter);
        }
        self
    }

    /// Add sorts
    pub fn with_sorts(mut self, sorts: Vec<crate::Sort>) -> Self {
        for sort in sorts {
            self = self.order_by(sort);
        }
        self
    }

    /// Add having condition
    pub fn having_condition(
        mut self,
        condition: &str,
        _params: impl Into<Vec<libsql::Value>>,
    ) -> Self {
        // This is a simplified implementation
        self.having
            .push(FilterOperator::Custom(condition.to_string()));
        self
    }

    /// Add where in clause
    pub fn where_in(mut self, field: &str, subquery: QueryBuilder) -> Self {
        let (subquery_sql, _) = subquery.build().unwrap_or_default();
        let condition = format!("{field} IN ({subquery_sql})");
        self.where_clauses.push(FilterOperator::Custom(condition));
        self
    }

    /// Execute count query
    pub async fn execute_count(&self, db: &Database) -> Result<u64> {
        let (sql, params) = self.build_count()?;
        let mut rows = db.query(&sql, params).await?;

        if let Some(row) = rows.next().await? {
            row.get_value(0)
                .ok()
                .and_then(|v| match v {
                    libsql::Value::Integer(i) => Some(i as u64),
                    _ => None,
                })
                .ok_or_else(|| crate::Error::Query("Failed to get count".to_string()))
        } else {
            Err(crate::Error::Query("No count result".to_string()))
        }
    }

    /// Execute aggregate query
    pub async fn execute_aggregate(&self, db: &Database) -> Result<Vec<libsql::Row>> {
        let (sql, params) = self.build()?;
        let mut rows = db.query(&sql, params).await?;
        let mut results = Vec::new();
        while let Some(row) = rows.next().await? {
            results.push(row);
        }
        Ok(results)
    }

    /// Build the SQL query
    pub fn build(&self) -> Result<(String, Vec<libsql::Value>)> {
        let mut sql = String::new();
        let mut params = Vec::new();

        // SELECT clause
        sql.push_str("SELECT ");
        if self.distinct {
            sql.push_str("DISTINCT ");
        }

        if let Some(agg) = &self.aggregate {
            sql.push_str(&format!("{}({})", agg.function, agg.column));
            if let Some(alias) = &agg.alias {
                sql.push_str(&format!(" AS {alias}"));
            }
        } else {
            sql.push_str(&self.select_columns.join(", "));
        }

        // FROM clause
        sql.push_str(&format!(" FROM {}", self.table));

        // JOIN clauses
        for join in &self.joins {
            sql.push_str(&format!(" {} {}", join.join_type, join.table));
            if let Some(alias) = &join.alias {
                sql.push_str(&format!(" AS {alias}"));
            }
            sql.push_str(&format!(" ON {}", join.condition));
        }

        // WHERE clause
        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            let (where_sql, where_params) = self.build_where_clause(&self.where_clauses)?;
            sql.push_str(&where_sql);
            params.extend(where_params);
        }

        // GROUP BY clause
        if !self.group_by.is_empty() {
            sql.push_str(&format!(" GROUP BY {}", self.group_by.join(", ")));
        }

        // HAVING clause
        if !self.having.is_empty() {
            sql.push_str(" HAVING ");
            let (having_sql, having_params) = self.build_where_clause(&self.having)?;
            sql.push_str(&having_sql);
            params.extend(having_params);
        }

        // ORDER BY clause
        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            let order_clauses: Vec<String> = self
                .order_by
                .iter()
                .map(|sort| format!("{} {}", sort.column, sort.order))
                .collect();
            sql.push_str(&order_clauses.join(", "));
        }

        // LIMIT and OFFSET
        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {limit}"));
        }
        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {offset}"));
        }

        Ok((sql, params))
    }

    /// Build a count query
    pub fn build_count(&self) -> Result<(String, Vec<libsql::Value>)> {
        let mut sql = String::new();
        let mut params = Vec::new();

        sql.push_str("SELECT COUNT(*)");

        // FROM clause
        sql.push_str(&format!(" FROM {}", self.table));

        // JOIN clauses
        for join in &self.joins {
            sql.push_str(&format!(" {} {}", join.join_type, join.table));
            if let Some(alias) = &join.alias {
                sql.push_str(&format!(" AS {alias}"));
            }
            sql.push_str(&format!(" ON {}", join.condition));
        }

        // WHERE clause
        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            let (where_sql, where_params) = self.build_where_clause(&self.where_clauses)?;
            sql.push_str(&where_sql);
            params.extend(where_params);
        }

        // GROUP BY clause
        if !self.group_by.is_empty() {
            sql.push_str(&format!(" GROUP BY {}", self.group_by.join(", ")));
        }

        // HAVING clause
        if !self.having.is_empty() {
            sql.push_str(" HAVING ");
            let (having_sql, having_params) = self.build_where_clause(&self.having)?;
            sql.push_str(&having_sql);
            params.extend(having_params);
        }

        Ok((sql, params))
    }

    /// Build where clause from filter operators
    fn build_where_clause(
        &self,
        filters: &[FilterOperator],
    ) -> Result<(String, Vec<libsql::Value>)> {
        let mut sql = String::new();
        let mut params = Vec::new();

        for (i, filter) in filters.iter().enumerate() {
            if i > 0 {
                sql.push_str(" AND ");
            }
            let (filter_sql, filter_params) = self.build_filter_operator(filter)?;
            sql.push_str(&filter_sql);
            params.extend(filter_params);
        }

        Ok((sql, params))
    }

    /// Build filter operator
    fn build_filter_operator(
        &self,
        filter: &FilterOperator,
    ) -> Result<(String, Vec<libsql::Value>)> {
        match filter {
            FilterOperator::Single(filter) => self.build_filter(filter),
            FilterOperator::And(filters) => {
                let mut sql = String::new();
                let mut params = Vec::new();
                sql.push('(');
                for (i, filter) in filters.iter().enumerate() {
                    if i > 0 {
                        sql.push_str(" AND ");
                    }
                    let (filter_sql, filter_params) = self.build_filter_operator(filter)?;
                    sql.push_str(&filter_sql);
                    params.extend(filter_params);
                }
                sql.push(')');
                Ok((sql, params))
            }
            FilterOperator::Or(filters) => {
                let mut sql = String::new();
                let mut params = Vec::new();
                sql.push('(');
                for (i, filter) in filters.iter().enumerate() {
                    if i > 0 {
                        sql.push_str(" OR ");
                    }
                    let (filter_sql, filter_params) = self.build_filter_operator(filter)?;
                    sql.push_str(&filter_sql);
                    params.extend(filter_params);
                }
                sql.push(')');
                Ok((sql, params))
            }
            FilterOperator::Not(filter) => {
                let (filter_sql, filter_params) = self.build_filter_operator(filter)?;
                Ok((format!("NOT ({filter_sql})"), filter_params))
            }
            FilterOperator::Custom(condition) => Ok((condition.clone(), vec![])),
        }
    }

    /// Build individual filter
    fn build_filter(&self, filter: &crate::Filter) -> Result<(String, Vec<libsql::Value>)> {
        let mut sql = String::new();
        let mut params = Vec::new();

        match &filter.operator {
            Operator::IsNull => {
                sql.push_str(&format!("{} IS NULL", filter.column));
            }
            Operator::IsNotNull => {
                sql.push_str(&format!("{} IS NOT NULL", filter.column));
            }
            _ => {
                sql.push_str(&format!("{} {} ", filter.column, filter.operator));
                match &filter.value {
                    FilterValue::Single(value) => {
                        sql.push('?');
                        params.push(self.value_to_libsql_value(value));
                    }
                    FilterValue::Multiple(values) => {
                        sql.push('(');
                        for (i, value) in values.iter().enumerate() {
                            if i > 0 {
                                sql.push_str(", ");
                            }
                            sql.push('?');
                            params.push(self.value_to_libsql_value(value));
                        }
                        sql.push(')');
                    }
                    FilterValue::Range(min, max) => {
                        sql.push_str("? AND ?");
                        params.push(self.value_to_libsql_value(min));
                        params.push(self.value_to_libsql_value(max));
                    }
                }
            }
        }

        Ok((sql, params))
    }

    /// Convert our Value type to libsql::Value
    fn value_to_libsql_value(&self, value: &Value) -> libsql::Value {
        match value {
            Value::Null => libsql::Value::Null,
            Value::Integer(i) => libsql::Value::Integer(*i),
            Value::Real(f) => libsql::Value::Real(*f),
            Value::Text(s) => libsql::Value::Text(s.clone()),
            Value::Blob(b) => libsql::Value::Blob(b.clone()),
            Value::Boolean(b) => libsql::Value::Integer(if *b { 1 } else { 0 }),
        }
    }

    /// Execute the query
    pub async fn execute<T>(&self, db: &Database) -> Result<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let (sql, params) = self.build()?;
        let mut rows = db.query(&sql, params).await?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().await? {
            let mut map = HashMap::new();
            for i in 0..row.column_count() {
                if let Some(column_name) = row.column_name(i) {
                    let value = row.get_value(i).unwrap_or(libsql::Value::Null);
                    map.insert(
                        column_name.to_string(),
                        self.libsql_value_to_json_value(&value),
                    );
                }
            }
            let json_value = serde_json::to_value(map)?;
            let result: T = serde_json::from_value(json_value)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Execute the query with pagination
    pub async fn execute_paginated<T>(
        &self,
        db: &Database,
        pagination: &Pagination,
    ) -> Result<PaginatedResult<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        // Get total count
        let count_builder = QueryBuilder::new(&self.table).select(vec!["COUNT(*) as count"]);

        let (count_sql, count_params) = count_builder.build_count()?;
        let mut count_rows = db.query(&count_sql, count_params).await?;
        let total: u64 = if let Some(row) = count_rows.next().await? {
            row.get_value(0)
                .ok()
                .and_then(|v| match v {
                    libsql::Value::Integer(i) => Some(i as u64),
                    _ => None,
                })
                .unwrap_or(0)
        } else {
            0
        };

        // Get paginated data
        let data_builder = self
            .clone()
            .limit(pagination.limit())
            .offset(pagination.offset());

        let data = data_builder.execute::<T>(db).await?;

        Ok(PaginatedResult::with_total(data, pagination.clone(), total))
    }

    /// Convert libsql::Value to serde_json::Value
    fn libsql_value_to_json_value(&self, value: &libsql::Value) -> serde_json::Value {
        match value {
            libsql::Value::Null => serde_json::Value::Null,
            libsql::Value::Integer(i) => serde_json::Value::Number(serde_json::Number::from(*i)),
            libsql::Value::Real(f) => {
                if let Some(n) = serde_json::Number::from_f64(*f) {
                    serde_json::Value::Number(n)
                } else {
                    serde_json::Value::Null
                }
            }
            libsql::Value::Text(s) => serde_json::Value::String(s.clone()),
            libsql::Value::Blob(b) => serde_json::Value::Array(
                b.iter()
                    .map(|&byte| serde_json::Value::Number(serde_json::Number::from(byte)))
                    .collect(),
            ),
        }
    }
}

impl Clone for QueryBuilder {
    fn clone(&self) -> Self {
        Self {
            table: self.table.clone(),
            select_columns: self.select_columns.clone(),
            joins: self.joins.clone(),
            where_clauses: self.where_clauses.clone(),
            group_by: self.group_by.clone(),
            having: self.having.clone(),
            order_by: self.order_by.clone(),
            limit: self.limit,
            offset: self.offset,
            distinct: self.distinct,
            aggregate: self.aggregate.clone(),
        }
    }
}

impl Clone for JoinClause {
    fn clone(&self) -> Self {
        Self {
            join_type: self.join_type,
            table: self.table.clone(),
            alias: self.alias.clone(),
            condition: self.condition.clone(),
        }
    }
}

impl Clone for AggregateClause {
    fn clone(&self) -> Self {
        Self {
            function: self.function,
            column: self.column.clone(),
            alias: self.alias.clone(),
        }
    }
}
