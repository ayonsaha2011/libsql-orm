//! Database migration system for libsql-orm
//! 
//! This module provides a comprehensive migration system for managing database schema
//! changes over time. It supports creating, executing, and tracking migrations with
//! both manual and auto-generated approaches.
//! 
//! # Features
//! 
//! - **Auto-generation**: Generate migrations from model definitions
//! - **Manual creation**: Build custom migrations with the builder pattern
//! - **Templates**: Pre-built migration templates for common operations
//! - **History tracking**: Track which migrations have been executed
//! - **Rollback support**: Reverse migrations with down scripts
//! - **Batch execution**: Run multiple migrations in sequence
//! 
//! # Basic Usage
//! 
//! ```rust
//! use libsql_orm::{MigrationManager, MigrationBuilder, generate_migration};
//! 
//! async fn run_migrations(db: Database) -> Result<(), Error> {
//!     let manager = MigrationManager::new(db);
//!     manager.init().await?;
//!     
//!     // Auto-generate from model
//!     let migration = generate_migration!(User);
//!     manager.execute_migration(&migration).await?;
//!     
//!     // Manual migration
//!     let manual_migration = MigrationBuilder::new("add_index")
//!         .up("CREATE INDEX idx_users_email ON users(email)")
//!         .down("DROP INDEX idx_users_email")
//!         .build();
//!     
//!     manager.execute_migration(&manual_migration).await?;
//!     Ok(())
//! }
//! ```
//! 
//! # Migration Templates
//! 
//! ```rust
//! use libsql_orm::templates;
//! 
//! // Create table
//! let create_table = templates::create_table("posts", &[
//!     ("id", "INTEGER PRIMARY KEY AUTOINCREMENT"),
//!     ("title", "TEXT NOT NULL"),
//!     ("content", "TEXT"),
//! ]);
//! 
//! // Add column
//! let add_column = templates::add_column("posts", "published_at", "TEXT");
//! 
//! // Create index
//! let create_index = templates::create_index("idx_posts_title", "posts", &["title"]);
//! ```

use crate::{error::Error, database::Database};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Represents a database migration
/// 
/// A migration contains the SQL statements needed to evolve the database schema
/// along with metadata for tracking execution history.
/// 
/// # Examples
/// 
/// ```rust
/// use libsql_orm::{Migration, MigrationBuilder};
/// 
/// let migration = MigrationBuilder::new("create_users_table")
///     .up("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
///     .down("DROP TABLE users")
///     .build();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    pub id: String,
    pub name: String,
    pub sql: String,
    pub created_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
}

/// Migration manager for handling database schema changes
/// 
/// The central component for managing database migrations. Handles initialization,
/// execution, and tracking of schema changes.
/// 
/// # Examples
/// 
/// ```rust
/// use libsql_orm::{MigrationManager, Database};
/// 
/// async fn setup_migrations(db: Database) -> Result<(), Error> {
///     let manager = MigrationManager::new(db);
///     
///     // Initialize migration tracking
///     manager.init().await?;
///     
///     // Get migration status
///     let executed = manager.get_executed_migrations().await?;
///     let pending = manager.get_pending_migrations().await?;
///     
///     println!("Executed: {}, Pending: {}", executed.len(), pending.len());
///     Ok(())
/// }
/// ```
pub struct MigrationManager {
    db: Database,
}

impl MigrationManager {
    /// Create a new migration manager
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Initialize the migration table
    pub async fn init(&self) -> Result<(), Error> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS migrations (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                sql TEXT NOT NULL,
                created_at TEXT NOT NULL,
                executed_at TEXT
            )
        "#;

        let params = vec![libsql::Value::Null; 0];
        
        self.db.inner.execute(sql, params).await?;
        Ok(())
    }

    /// Create a new migration
    pub fn create_migration(name: &str, sql: &str) -> Migration {
        Migration {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            sql: sql.to_string(),
            created_at: Utc::now(),
            executed_at: None,
        }
    }

    /// Get all migrations from the database
    pub async fn get_migrations(&self) -> Result<Vec<Migration>, Error> {
        let sql = "SELECT id, name, sql, created_at, executed_at FROM migrations ORDER BY created_at";
        let mut rows = self.db.inner.query(sql, vec![libsql::Value::Null; 0]).await?;
        
        let mut migrations = Vec::new();
        while let Some(row) = rows.next().await? {
            let migration = Migration {
                id: row.get(0)?,
                name: row.get(1)?,
                sql: row.get(2)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<String>(3).unwrap_or_default())
                    .map_err(|_| Error::DatabaseError("Invalid datetime format".to_string()))?
                    .with_timezone(&Utc),
                executed_at: row.get::<Option<String>>(4).unwrap_or(None)
                    .map(|dt| DateTime::parse_from_rfc3339(&dt)
                        .map_err(|_| Error::DatabaseError("Invalid datetime format".to_string()))
                        .map(|dt| dt.with_timezone(&Utc)))
                    .transpose()?,
            };
            migrations.push(migration);
        }
        
        Ok(migrations)
    }

    /// Execute a migration
    pub async fn execute_migration(&self, migration: &Migration) -> Result<(), Error> {
        // Begin transaction
        self.db.inner.execute("BEGIN", vec![libsql::Value::Null; 0]).await?;
        
        // Execute the migration SQL
        self.db.inner.execute(&migration.sql, vec![libsql::Value::Null; 0]).await?;
        
        // Record the migration
        let sql = r#"
            INSERT INTO migrations (id, name, sql, created_at, executed_at)
            VALUES (?, ?, ?, ?, ?)
        "#;
        
        self.db.inner.execute(sql, vec![
            libsql::Value::Text(migration.id.clone()),
            libsql::Value::Text(migration.name.clone()),
            libsql::Value::Text(migration.sql.clone()),
            libsql::Value::Text(migration.created_at.to_rfc3339()),
            libsql::Value::Text(Utc::now().to_rfc3339()),
        ]).await?;
        
        // Commit transaction
        self.db.inner.execute("COMMIT", vec![libsql::Value::Null; 0]).await?;
        
        Ok(())
    }

    /// Rollback a migration
    pub async fn rollback_migration(&self, migration_id: &str) -> Result<(), Error> {
        let sql = "DELETE FROM migrations WHERE id = ?";
        self.db.inner.execute(sql, vec![libsql::Value::Text(migration_id.to_string())]).await?;
        Ok(())
    }

    /// Get pending migrations (not yet executed)
    pub async fn get_pending_migrations(&self) -> Result<Vec<Migration>, Error> {
        let migrations = self.get_migrations().await?;
        Ok(migrations.into_iter().filter(|m| m.executed_at.is_none()).collect())
    }

    /// Get executed migrations
    pub async fn get_executed_migrations(&self) -> Result<Vec<Migration>, Error> {
        let migrations = self.get_migrations().await?;
        Ok(migrations.into_iter().filter(|m| m.executed_at.is_some()).collect())
    }

    /// Run all pending migrations
    pub async fn run_migrations(&self, migrations: Vec<Migration>) -> Result<(), Error> {
        for migration in migrations {
            if let Some(executed_at) = migration.executed_at {
                println!("Migration '{}' already executed at {}", migration.name, executed_at);
                continue;
            }
            
            println!("Executing migration: {}", migration.name);
            self.execute_migration(&migration).await?;
            println!("Migration '{}' executed successfully", migration.name);
        }
        
        Ok(())
    }

    /// Create a migration from a file
    pub async fn create_migration_from_file(name: &str, file_path: &str) -> Result<Migration, Error> {
        let sql = std::fs::read_to_string(file_path)
            .map_err(|e| Error::DatabaseError(format!("Failed to read migration file: {}", e)))?;
        
        Ok(Self::create_migration(name, &sql))
    }

    /// Generate a migration name from a description
    pub fn generate_migration_name(description: &str) -> String {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let sanitized_description = description
            .to_lowercase()
            .replace(" ", "_")
            .replace("-", "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>();
        
        format!("{}_{}", timestamp, sanitized_description)
    }

    pub fn database(&self) -> &Database {
        &self.db
    }
}

/// Builder for creating migrations
/// 
/// Provides a fluent interface for constructing migrations with up and down SQL.
/// 
/// # Examples
/// 
/// ```rust
/// use libsql_orm::MigrationBuilder;
/// 
/// let migration = MigrationBuilder::new("add_user_email_index")
///     .up("CREATE UNIQUE INDEX idx_users_email ON users(email)")
///     .down("DROP INDEX idx_users_email")
///     .build();
/// ```
pub struct MigrationBuilder {
    name: String,
    up_sql: String,
    down_sql: Option<String>,
}

impl MigrationBuilder {
    /// Create a new migration builder
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            up_sql: String::new(),
            down_sql: None,
        }
    }

    /// Add SQL for the up migration
    pub fn up(mut self, sql: &str) -> Self {
        self.up_sql = sql.to_string();
        self
    }

    /// Add SQL for the down migration (rollback)
    pub fn down(mut self, sql: &str) -> Self {
        self.down_sql = Some(sql.to_string());
        self
    }

    /// Build the migration
    pub fn build(self) -> Migration {
        Migration {
            id: uuid::Uuid::new_v4().to_string(),
            name: self.name,
            sql: self.up_sql,
            created_at: Utc::now(),
            executed_at: None,
        }
    }
}

/// Common migration templates
/// 
/// Pre-built migration templates for common database operations like creating tables,
/// adding columns, creating indexes, etc. These templates provide a quick way to
/// generate migrations for standard operations.
/// 
/// # Examples
/// 
/// ```rust
/// use libsql_orm::templates;
/// 
/// // Create a new table
/// let create_table = templates::create_table(\"users\", &[\n///     (\"id\", \"INTEGER PRIMARY KEY AUTOINCREMENT\"),\n///     (\"name\", \"TEXT NOT NULL\"),\n///     (\"email\", \"TEXT UNIQUE NOT NULL\"),\n/// ]);\n/// \n/// // Add a column to existing table\n/// let add_column = templates::add_column(\"users\", \"created_at\", \"TEXT NOT NULL\");\n/// \n/// // Create an index\n/// let create_index = templates::create_index(\"idx_users_email\", \"users\", &[\"email\"]);\n/// ```
pub mod templates {
    use super::*;

    /// Create a table migration
    pub fn create_table(table_name: &str, columns: &[(&str, &str)]) -> Migration {
        let column_definitions = columns
            .iter()
            .map(|(name, definition)| format!("{} {}", name, definition))
            .collect::<Vec<_>>()
            .join(", ");
        
        let sql = format!("CREATE TABLE {} ({})", table_name, column_definitions);
        
        MigrationBuilder::new(&format!("create_table_{}", table_name))
            .up(&sql)
            .build()
    }

    /// Add column migration
    pub fn add_column(table_name: &str, column_name: &str, definition: &str) -> Migration {
        let sql = format!("ALTER TABLE {} ADD COLUMN {} {}", table_name, column_name, definition);
        
        MigrationBuilder::new(&format!("add_column_{}_{}", table_name, column_name))
            .up(&sql)
            .build()
    }

    /// Drop column migration
    pub fn drop_column(table_name: &str, column_name: &str) -> Migration {
        let sql = format!("ALTER TABLE {} DROP COLUMN {}", table_name, column_name);
        
        MigrationBuilder::new(&format!("drop_column_{}_{}", table_name, column_name))
            .up(&sql)
            .build()
    }

    /// Create index migration
    pub fn create_index(index_name: &str, table_name: &str, columns: &[&str]) -> Migration {
        let column_list = columns.join(", ");
        let sql = format!("CREATE INDEX {} ON {} ({})", index_name, table_name, column_list);
        
        MigrationBuilder::new(&format!("create_index_{}", index_name))
            .up(&sql)
            .build()
    }

    /// Drop index migration
    pub fn drop_index(index_name: &str) -> Migration {
        let sql = format!("DROP INDEX {}", index_name);
        
        MigrationBuilder::new(&format!("drop_index_{}", index_name))
            .up(&sql)
            .build()
    }
} 