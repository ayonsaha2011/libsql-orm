//! Model trait and core database operations
//!
//! This module provides the main Model trait that enables ORM functionality
//! for structs that derive from it. Features include:
//!
//! - **Custom Table Names**: Use `#[table_name("custom")]` to override default naming
//! - **Boolean Type Safety**: Automatic conversion between SQLite integers (0/1) and Rust booleans
//! - **Column Attributes**: Customize column properties with `#[orm_column(...)]`
//! - **Full CRUD Operations**: Create, read, update, delete with type safety
//!
//! # Examples
//!
//! ```rust
//! use libsql_orm::Model;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Model, Serialize, Deserialize)]
//! #[table_name("user_accounts")]  // Custom table name
//! struct User {
//!     pub id: Option<i64>,
//!     pub name: String,
//!     pub is_active: bool,     // ✅ Automatic boolean conversion
//!     pub is_verified: bool,   // ✅ Type-safe operations
//! }
//! ```

use crate::{
    Aggregate, Database, Error, FilterOperator, PaginatedResult, Pagination, QueryBuilder, Result,
    SearchFilter, Sort,
};
use std::collections::HashMap;

use serde::{de::DeserializeOwned, Serialize};

/// Mask numeric IDs for logging
fn mask_id(id: i64) -> String {
    if id < 100 {
        return "*".repeat(id.to_string().len());
    }
    let id_str = id.to_string();
    let visible_digits = 2;
    let masked_digits = id_str.len() - visible_digits;
    format!("{}{}", &id_str[..visible_digits], "*".repeat(masked_digits))
}

/// Core trait for all database models
#[allow(async_fn_in_trait)]
pub trait Model: Serialize + DeserializeOwned + Send + Sync + Clone {
    /// Get the table name for this model
    fn table_name() -> &'static str;

    /// Get the primary key column name
    fn primary_key() -> &'static str {
        "id"
    }

    /// Get the primary key value
    fn get_primary_key(&self) -> Option<i64>;

    /// Set the primary key value
    fn set_primary_key(&mut self, id: i64);

    /// Get all column names for this model
    fn columns() -> Vec<&'static str>;

    /// Generate SQL for creating the table
    fn migration_sql() -> String;

    /// Convert the model to a HashMap for database operations
    fn to_map(&self) -> Result<HashMap<String, crate::Value>>;

    /// Create a model from a HashMap
    fn from_map(map: HashMap<String, crate::Value>) -> Result<Self>;

    /// Create a new record in the database
    async fn create(&self, db: &Database) -> Result<Self> {
        let map = self.to_map()?;
        let columns: Vec<String> = map.keys().cloned().collect();
        let values: Vec<String> = map.keys().map(|_| "?".to_string()).collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            Self::table_name(),
            columns.join(", "),
            values.join(", ")
        );

        Self::log_info(&format!("Creating record in table: {}", Self::table_name()));
        Self::log_debug(&format!("SQL: {sql}"));

        let params: Vec<libsql::Value> = map
            .values()
            .map(|v| Self::value_to_libsql_value(v))
            .collect();

        db.inner.execute(&sql, params).await?;
        let id = 1i64; // Placeholder - libsql WASM doesn't support last_insert_rowid

        let mut result = self.clone();
        result.set_primary_key(id);

        Self::log_info(&format!(
            "Successfully created record with ID: {}",
            mask_id(id)
        ));
        Ok(result)
    }

    /// Create or update a record based on whether it has a primary key
    async fn create_or_update(&self, db: &Database) -> Result<Self> {
        if let Some(id) = self.get_primary_key() {
            Self::log_info(&format!(
                "Updating existing record with ID: {}",
                mask_id(id)
            ));
            // Check if record exists
            match Self::find_by_id(id, db).await? {
                Some(_) => {
                    // Record exists, update it
                    self.update(db).await
                }
                None => {
                    // Record doesn't exist, create it
                    Self::log_warn(&format!(
                        "Record with ID {} not found, creating new record",
                        mask_id(id)
                    ));
                    self.create(db).await
                }
            }
        } else {
            // No primary key, create new record
            Self::log_info("Creating new record (no primary key provided)");
            self.create(db).await
        }
    }

    /// Create or update a record based on unique constraints
    async fn upsert(&self, unique_columns: &[&str], db: &Database) -> Result<Self> {
        let map = self.to_map()?;

        // Build WHERE clause for unique columns
        let mut where_conditions = Vec::new();
        let mut where_params = Vec::new();

        for &column in unique_columns {
            if let Some(value) = map.get(column) {
                where_conditions.push(format!("{column} = ?"));
                where_params.push(Self::value_to_libsql_value(value));
            }
        }

        if where_conditions.is_empty() {
            return Err(Error::Validation(
                "No unique columns provided for upsert".to_string(),
            ));
        }

        let where_clause = where_conditions.join(" AND ");
        let sql = format!(
            "SELECT {} FROM {} WHERE {}",
            Self::primary_key(),
            Self::table_name(),
            where_clause
        );

        Self::log_info(&format!(
            "Checking for existing record in table: {}",
            Self::table_name()
        ));
        Self::log_debug(&format!("SQL: {sql}"));

        let mut rows = db.inner.query(&sql, where_params).await?;

        if let Some(row) = rows.next().await? {
            // Record exists, update it
            if let Some(existing_id) = row.get_value(0).ok().and_then(|v| match v {
                libsql::Value::Integer(i) => Some(i),
                _ => None,
            }) {
                Self::log_info(&format!(
                    "Found existing record with ID: {}, updating",
                    mask_id(existing_id)
                ));
                let mut updated_self = self.clone();
                updated_self.set_primary_key(existing_id);
                updated_self.update(db).await
            } else {
                Err(Error::Query(
                    "Failed to get primary key from existing record".to_string(),
                ))
            }
        } else {
            // Record doesn't exist, create it
            Self::log_info("No existing record found, creating new one");
            self.create(db).await
        }
    }

    /// Create multiple records in the database
    async fn bulk_create(models: &[Self], db: &Database) -> Result<Vec<Self>> {
        if models.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();
        // Note: Manual transaction handling for WASM
        db.inner
            .execute("BEGIN", vec![libsql::Value::Null; 0])
            .await?;

        for model in models {
            let map = model.to_map()?;
            let columns: Vec<String> = map.keys().cloned().collect();
            let values: Vec<String> = map.keys().map(|_| "?".to_string()).collect();

            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                Self::table_name(),
                columns.join(", "),
                values.join(", ")
            );

            let params: Vec<libsql::Value> = map
                .values()
                .map(|v| Self::value_to_libsql_value(v))
                .collect();

            db.inner.execute(&sql, params).await?;
            let id = 1i64; // Placeholder - libsql WASM doesn't support last_insert_rowid

            let mut result = model.clone();
            result.set_primary_key(id);
            results.push(result);
        }

        db.inner
            .execute("COMMIT", vec![libsql::Value::Null; 0])
            .await?;
        Ok(results)
    }

    /// Find a record by its primary key
    async fn find_by_id(id: i64, db: &Database) -> Result<Option<Self>> {
        let sql = format!(
            "SELECT * FROM {} WHERE {} = ?",
            Self::table_name(),
            Self::primary_key()
        );

        Self::log_debug(&format!("Finding record by ID: {}", mask_id(id)));
        Self::log_debug(&format!("SQL: {sql}"));

        let mut rows = db
            .inner
            .query(&sql, vec![libsql::Value::Integer(id)])
            .await?;

        if let Some(row) = rows.next().await? {
            let map = Self::row_to_map(&row)?;
            Self::log_debug(&format!("Found record with ID: {}", mask_id(id)));
            Ok(Some(Self::from_map(map)?))
        } else {
            Self::log_debug(&format!("No record found with ID: {}", mask_id(id)));
            Ok(None)
        }
    }

    /// Find a single record by a specific condition
    async fn find_one(filter: FilterOperator, db: &Database) -> Result<Option<Self>> {
        let builder = QueryBuilder::new(Self::table_name())
            .r#where(filter)
            .limit(1);

        let results = builder.execute::<Self>(db).await?;
        Ok(results.into_iter().next())
    }

    /// Find all records
    async fn find_all(db: &Database) -> Result<Vec<Self>> {
        let builder = QueryBuilder::new(Self::table_name());
        builder.execute::<Self>(db).await
    }

    /// Find records with a filter
    async fn find_where(filter: FilterOperator, db: &Database) -> Result<Vec<Self>> {
        let builder = QueryBuilder::new(Self::table_name()).r#where(filter);
        builder.execute::<Self>(db).await
    }

    /// Find records with pagination
    async fn find_paginated(
        pagination: &Pagination,
        db: &Database,
    ) -> Result<PaginatedResult<Self>> {
        let builder = QueryBuilder::new(Self::table_name());
        builder.execute_paginated::<Self>(db, pagination).await
    }

    /// Find records with filter and pagination
    async fn find_where_paginated(
        filter: FilterOperator,
        pagination: &Pagination,
        db: &Database,
    ) -> Result<PaginatedResult<Self>> {
        let builder = QueryBuilder::new(Self::table_name()).r#where(filter);
        builder.execute_paginated::<Self>(db, pagination).await
    }

    /// Search records with text search
    async fn search(
        search_filter: &SearchFilter,
        pagination: Option<&Pagination>,
        db: &Database,
    ) -> Result<PaginatedResult<Self>> {
        let filter = search_filter.to_filter_operator();
        let pagination = pagination.unwrap_or(&Pagination::default()).clone();

        Self::find_where_paginated(filter, &pagination, db).await
    }

    /// Count all records
    async fn count(db: &Database) -> Result<u64> {
        let sql = format!("SELECT COUNT(*) FROM {}", Self::table_name());
        let mut rows = db.inner.query(&sql, vec![libsql::Value::Null; 0]).await?;

        if let Some(row) = rows.next().await? {
            row.get_value(0)
                .ok()
                .and_then(|v| match v {
                    libsql::Value::Integer(i) => Some(i as u64),
                    _ => None,
                })
                .ok_or_else(|| Error::Query("Failed to get count".to_string()))
        } else {
            Err(Error::Query("No count result".to_string()))
        }
    }

    /// Count records with a filter
    async fn count_where(filter: FilterOperator, db: &Database) -> Result<u64> {
        let builder = QueryBuilder::new(Self::table_name()).r#where(filter);

        let (sql, params) = builder.build_count()?;
        let mut rows = db.inner.query(&sql, params).await?;

        if let Some(row) = rows.next().await? {
            row.get_value(0)
                .ok()
                .and_then(|v| match v {
                    libsql::Value::Integer(i) => Some(i as u64),
                    _ => None,
                })
                .ok_or_else(|| Error::Query("Failed to get count".to_string()))
        } else {
            Err(Error::Query("No count result".to_string()))
        }
    }

    /// Update a record
    async fn update(&self, db: &Database) -> Result<Self> {
        let id = self.get_primary_key().ok_or_else(|| {
            Error::Validation("Cannot update record without primary key".to_string())
        })?;

        let map = self.to_map()?;
        let set_clauses: Vec<String> = map
            .keys()
            .filter(|&k| k != Self::primary_key())
            .map(|k| format!("{k} = ?"))
            .collect();

        let sql = format!(
            "UPDATE {} SET {} WHERE {} = ?",
            Self::table_name(),
            set_clauses.join(", "),
            Self::primary_key()
        );

        Self::log_info(&format!("Updating record with ID: {}", mask_id(id)));
        Self::log_debug(&format!("SQL: {sql}"));

        let mut params: Vec<libsql::Value> = map
            .iter()
            .filter(|(k, _)| k != &Self::primary_key())
            .map(|(_, v)| Self::value_to_libsql_value(v))
            .collect();
        params.push(libsql::Value::Integer(id));

        db.inner.execute(&sql, params).await?;
        Self::log_info(&format!(
            "Successfully updated record with ID: {}",
            mask_id(id)
        ));
        Ok(self.clone())
    }

    /// Update multiple records
    async fn bulk_update(models: &[Self], db: &Database) -> Result<Vec<Self>> {
        if models.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();
        // Note: Manual transaction handling for WASM
        db.inner
            .execute("BEGIN", vec![libsql::Value::Null; 0])
            .await?;

        for model in models {
            let result = model.update(db).await?;
            results.push(result);
        }

        db.inner
            .execute("COMMIT", vec![libsql::Value::Null; 0])
            .await?;
        Ok(results)
    }

    /// Delete a record
    async fn delete(&self, db: &Database) -> Result<bool> {
        let id = self.get_primary_key().ok_or_else(|| {
            Error::Validation("Cannot delete record without primary key".to_string())
        })?;

        let sql = format!(
            "DELETE FROM {} WHERE {} = ?",
            Self::table_name(),
            Self::primary_key()
        );

        Self::log_info(&format!("Deleting record with ID: {}", mask_id(id)));
        Self::log_debug(&format!("SQL: {sql}"));

        db.inner
            .execute(&sql, vec![libsql::Value::Integer(id)])
            .await?;
        Self::log_info(&format!(
            "Successfully deleted record with ID: {}",
            mask_id(id)
        ));
        Ok(true)
    }

    /// Delete multiple records
    async fn bulk_delete(ids: &[i64], db: &Database) -> Result<u64> {
        if ids.is_empty() {
            return Ok(0);
        }

        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "DELETE FROM {} WHERE {} IN ({})",
            Self::table_name(),
            Self::primary_key(),
            placeholders.join(", ")
        );

        let params: Vec<libsql::Value> = ids.iter().map(|&id| libsql::Value::Integer(id)).collect();
        db.inner.execute(&sql, params).await?;
        Ok(ids.len() as u64)
    }

    /// Delete records with a filter
    async fn delete_where(filter: FilterOperator, db: &Database) -> Result<u64> {
        let builder = QueryBuilder::new(Self::table_name()).r#where(filter);

        let (sql, params) = builder.build()?;
        let delete_sql = sql.replace("SELECT *", "DELETE");
        db.inner.execute(&delete_sql, params).await?;

        // Note: SQLite doesn't return the number of affected rows directly
        // This is a simplified implementation
        Ok(1)
    }

    /// List records with optional sorting and pagination
    async fn list(
        sort: Option<Vec<Sort>>,
        pagination: Option<&Pagination>,
        db: &Database,
    ) -> Result<PaginatedResult<Self>> {
        let mut builder = QueryBuilder::new(Self::table_name());

        if let Some(sorts) = sort {
            builder = builder.order_by_multiple(sorts);
        }

        let pagination = pagination.unwrap_or(&Pagination::default()).clone();
        builder.execute_paginated::<Self>(db, &pagination).await
    }

    /// List records with filter, sorting, and pagination
    async fn list_where(
        filter: FilterOperator,
        sort: Option<Vec<Sort>>,
        pagination: Option<&Pagination>,
        db: &Database,
    ) -> Result<PaginatedResult<Self>> {
        let mut builder = QueryBuilder::new(Self::table_name()).r#where(filter);

        if let Some(sorts) = sort {
            builder = builder.order_by_multiple(sorts);
        }

        let pagination = pagination.unwrap_or(&Pagination::default()).clone();
        builder.execute_paginated::<Self>(db, &pagination).await
    }

    /// Execute a custom query
    async fn query(builder: QueryBuilder, db: &Database) -> Result<Vec<Self>> {
        builder.execute::<Self>(db).await
    }

    /// Execute a custom query with pagination
    async fn query_paginated(
        builder: QueryBuilder,
        pagination: &Pagination,
        db: &Database,
    ) -> Result<PaginatedResult<Self>> {
        builder.execute_paginated::<Self>(db, pagination).await
    }

    /// Get aggregate value
    async fn aggregate(
        function: Aggregate,
        column: &str,
        filter: Option<FilterOperator>,
        db: &Database,
    ) -> Result<Option<f64>> {
        let mut builder =
            QueryBuilder::new(Self::table_name()).aggregate(function, column, None::<String>);

        if let Some(filter) = filter {
            builder = builder.r#where(filter);
        }

        let (sql, params) = builder.build()?;
        let mut rows = db.inner.query(&sql, params).await?;

        if let Some(row) = rows.next().await? {
            let value = row
                .get_value(0)
                .ok()
                .and_then(|v| match v {
                    libsql::Value::Integer(i) => Some(i as f64),
                    libsql::Value::Real(f) => Some(f),
                    _ => None,
                })
                .ok_or_else(|| Error::Query("Failed to get aggregate value".to_string()))?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Convert a database row to a HashMap
    fn row_to_map(row: &libsql::Row) -> Result<HashMap<String, crate::Value>> {
        let mut map = HashMap::new();
        for i in 0..row.column_count() {
            if let Some(column_name) = row.column_name(i) {
                let value = row.get_value(i).unwrap_or(libsql::Value::Null);
                map.insert(column_name.to_string(), Self::libsql_value_to_value(&value));
            }
        }
        Ok(map)
    }

    /// Convert our Value type to libsql::Value
    fn value_to_libsql_value(value: &crate::Value) -> libsql::Value {
        match value {
            crate::Value::Null => libsql::Value::Null,
            crate::Value::Integer(i) => libsql::Value::Integer(*i),
            crate::Value::Real(f) => libsql::Value::Real(*f),
            crate::Value::Text(s) => libsql::Value::Text(s.clone()),
            crate::Value::Blob(b) => libsql::Value::Blob(b.clone()),
            crate::Value::Boolean(b) => libsql::Value::Integer(if *b { 1 } else { 0 }),
        }
    }

    /// Convert libsql::Value to our Value type
    fn libsql_value_to_value(value: &libsql::Value) -> crate::Value {
        match value {
            libsql::Value::Null => crate::Value::Null,
            libsql::Value::Integer(i) => crate::Value::Integer(*i),
            libsql::Value::Real(f) => crate::Value::Real(*f),
            libsql::Value::Text(s) => crate::Value::Text(s.clone()),
            libsql::Value::Blob(b) => crate::Value::Blob(b.clone()),
        }
    }

    /// Log an info message
    fn log_info(message: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::log_1(&format!("[INFO] {}: {}", Self::table_name(), message).into());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            log::info!("[{}] {}", Self::table_name(), message);
        }
    }

    /// Log a debug message
    fn log_debug(message: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::log_1(&format!("[DEBUG] {}: {}", Self::table_name(), message).into());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            log::debug!("[{}] {}", Self::table_name(), message);
        }
    }

    /// Log a warning message
    fn log_warn(message: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::warn_1(&format!("[WARN] {}: {}", Self::table_name(), message).into());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            log::warn!("[{}] {}", Self::table_name(), message);
        }
    }

    /// Log an error message
    fn log_error(message: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::error_1(
                &format!("[ERROR] {}: {}", Self::table_name(), message).into(),
            );
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            log::error!("[{}] {}", Self::table_name(), message);
        }
    }
}
