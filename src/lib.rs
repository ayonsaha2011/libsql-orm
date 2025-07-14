//! # libsql-orm
//! 
//! A powerful, async-first ORM for [libsql](https://github.com/libsql/libsql) with first-class support for **Cloudflare Workers** and WebAssembly environments.
//! 
//! ## ✨ Features
//! 
//! - 🚀 **Cloudflare Workers Ready** - Built specifically for edge computing environments
//! - 🔄 **Async/Await Support** - Fully async API with excellent performance  
//! - 🎯 **Type-Safe** - Leverages Rust's type system for compile-time safety
//! - 📊 **Rich Query Builder** - Fluent API for complex queries
//! - 🔍 **Advanced Filtering** - Search, pagination, sorting, and aggregations
//! - 🛠️ **Migration System** - Database schema management and versioning
//! - 🎨 **Derive Macros** - Automatic model generation with `#[derive(Model)]`
//! - 📦 **Bulk Operations** - Efficient batch inserts, updates, and deletes
//! - 🌐 **WASM Compatible** - Optimized for WebAssembly targets
//! - 🔄 **Type Conversion** - Automatic conversion between SQLite and Rust types
//! - 🔄 **Upsert Operations** - Smart create_or_update and upsert methods
//! - 📝 **Built-in Logging** - Comprehensive logging for debugging and monitoring
//! 
//! ## 🚀 Quick Start
//! 
//! ```rust
//! use libsql_orm::{Model, Database};
//! use serde::{Deserialize, Serialize};
//! use chrono::{DateTime, Utc};
//! 
//! #[derive(Model, Debug, Clone, Serialize, Deserialize)]
//! struct User {
//!     pub id: Option<i64>,
//!     pub name: String,
//!     pub email: String,
//!     pub age: Option<i32>,
//!     pub is_active: bool,
//!     pub created_at: DateTime<Utc>,
//! }
//! 
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to database
//!     let db = Database::new_connect("libsql://your-db.turso.io", "your-auth-token").await?;
//!     
//!     // Create a user
//!     let user = User {
//!         id: None,
//!         name: "Alice".to_string(),
//!         email: "alice@example.com".to_string(),
//!         age: Some(30),
//!         is_active: true,
//!         created_at: Utc::now(),
//!     };
//!     
//!     // Save to database
//!     let saved_user = user.create(&db).await?;
//!     println!("Created user with ID: {:?}", saved_user.id);
//!     
//!     // Find users
//!     let users = User::find_all(&db).await?;
//!     println!("Found {} users", users.len());
//!     
//!     // Use smart upsert operations
//!     let user_with_id = User {
//!         id: Some(123),  // Will update if exists, create if not
//!         name: "Updated Name".to_string(),
//!         email: "updated@example.com".to_string(),
//!         age: Some(31),
//!         is_active: true,
//!         created_at: Utc::now(),
//!     };
//!     let smart_saved = user_with_id.create_or_update(&db).await?;
//!     
//!     // Upsert by unique constraint
//!     let unique_user = User {
//!         id: None,
//!         name: "Unique User".to_string(),
//!         email: "unique@example.com".to_string(),  // Check by email
//!         age: Some(25),
//!         is_active: true,
//!         created_at: Utc::now(),
//!     };
//!     let upserted = unique_user.upsert(&["email"], &db).await?;
//!     
//!     Ok(())
//! }
//! ```
//! 
//! ## 🔄 Upsert Operations
//! 
//! Smart create-or-update operations for efficient data management:
//! 
//! ```rust
//! use libsql_orm::{Model, Database};
//! 
//! // Method 1: create_or_update (based on primary key)
//! let user = User { id: Some(123), name: "John".to_string(), ... };
//! let saved = user.create_or_update(&db).await?;  // Updates if ID exists, creates if not
//! 
//! // Method 2: upsert (based on unique constraints)
//! let user = User { id: None, email: "john@example.com".to_string(), ... };
//! let saved = user.upsert(&["email"], &db).await?;  // Updates if email exists, creates if not
//! 
//! // Multiple unique constraints
//! let saved = user.upsert(&["email", "username"], &db).await?;
//! ```
//! 
//! ## 📝 Built-in Logging
//! 
//! Comprehensive logging for debugging and monitoring:
//! 
//! ```rust
//! // All database operations are automatically logged
//! // Logs appear in browser console (WASM) or standard logging (native)
//! 
//! let user = User::new("John", "john@example.com");
//! 
//! // Logs: [INFO] users: Creating record in table: users
//! // Logs: [DEBUG] users: SQL: INSERT INTO users (...) VALUES (...)
//! let saved = user.create(&db).await?;
//! 
//! // Logs: [DEBUG] users: Finding record by ID: 123
//! let found = User::find_by_id(123, &db).await?;
//! 
//! // Logs: [INFO] users: Updating record with ID: 123
//! let updated = found.unwrap().update(&db).await?;
//! ```
//! 
//! ## 📚 Advanced Usage
//! 
//! ### Custom Table Names and Boolean Type Safety
//! 
//! ```rust
//! use libsql_orm::{Model, orm_column, deserialize_bool};
//! use serde::{Serialize, Deserialize};
//! 
//! #[derive(Model, Debug, Clone, Serialize, Deserialize)]
//! #[table_name("user_accounts")]  // Custom table name
//! struct User {
//!     #[orm_column(type = "INTEGER PRIMARY KEY AUTOINCREMENT")]
//!     pub id: Option<i64>,
//!     
//!     #[orm_column(not_null, unique)]
//!     pub email: String,
//!     
//!     pub is_active: bool,        // ✅ Automatic SQLite integer ↔ Rust bool conversion
//!     pub is_verified: bool,      // ✅ Type-safe boolean operations
//!     
//!     // For edge cases, use custom deserializer
//!     #[serde(deserialize_with = "deserialize_bool")]
//!     pub has_premium: bool,      // ✅ Manual boolean conversion
//!     
//!     #[orm_column(type = "TEXT DEFAULT 'active'")]
//!     pub status: String,
//! }
//! 
//! // Boolean filtering works seamlessly
//! let active_users = User::find_where(
//!     FilterOperator::Eq("is_active".to_string(), Value::Boolean(true)),
//!     &db
//! ).await?;
//! ```
//! 
//! ### Query Builder
//! 
//! ```rust
//! use libsql_orm::{QueryBuilder, FilterOperator, Sort, SortOrder, Pagination};
//! 
//! // Complex query with filtering and pagination
//! let query = QueryBuilder::new("users")
//!     .select(&["id", "name", "email"])
//!     .r#where(FilterOperator::Gte("age".to_string(), Value::Integer(18)))
//!     .order_by(Sort::new("created_at", SortOrder::Desc))
//!     .limit(10)
//!     .offset(20);
//! 
//! let (sql, params) = query.build()?;
//! ```
//! 
//! ### Cloudflare Workers Integration
//! 
//! ```rust
//! use worker::*;
//! use libsql_orm::{Model, Database, MigrationManager, generate_migration};
//! 
//! #[event(fetch)]
//! async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
//!     let database_url = env.var("LIBSQL_DATABASE_URL")?.to_string();
//!     let auth_token = env.var("LIBSQL_AUTH_TOKEN")?.to_string();
//!     
//!     let db = Database::new_connect(&database_url, &auth_token).await
//!         .map_err(|e| format!("Database connection failed: {}", e))?;
//!     
//!     // Your application logic here
//!     let users = User::find_all(&db).await
//!         .map_err(|e| format!("Query failed: {}", e))?;
//!         
//!     Response::from_json(&users)
//! }
//! ```
pub mod database;
pub mod model;
pub mod query;
pub mod macros;
pub mod error;
pub mod types;
pub mod pagination;
pub mod filters;
pub mod migrations;

#[cfg(test)]
mod tests;

pub use database::Database;
pub use model::Model;
pub use query::{QueryBuilder, QueryResult};
pub use error::{Error, Result};
pub use types::*;
pub use pagination::{Pagination, PaginatedResult, CursorPagination, CursorPaginatedResult};
pub use filters::{Filter, FilterOperator, Sort, SearchFilter};
pub use migrations::{Migration, MigrationManager, MigrationBuilder, templates};

// Export the boolean deserializer
pub use types::deserialize_bool;

// Re-export commonly used types
pub use serde::{Deserialize, Serialize};
pub use chrono;
pub use uuid::Uuid;

/// Re-export the Model macro for convenience
pub use libsql_orm_macros::{Model, orm_column, generate_migration};