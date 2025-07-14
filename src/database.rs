//! Database connection and query execution
//! 
//! This module handles the connection to libsql databases and provides
//! query execution capabilities for Cloudflare Workers.

use libsql::wasm::{CloudflareSender, Connection, Rows};

/// Database connection wrapper for libsql in Cloudflare Workers
/// 
/// Provides a high-level interface for connecting to and interacting with
/// libsql databases in WebAssembly environments, specifically optimized
/// for Cloudflare Workers.
/// 
/// # Examples
/// 
/// ```no_run
/// use libsql_orm::Database;
/// 
/// async fn connect_example() -> Result<(), Box<dyn std::error::Error>> {
///     let db = Database::new_connect(
///         "libsql://your-db.turso.io", 
///         "your-auth-token"
///     ).await?;
///     Ok(())
/// }
/// ```
pub struct Database {
    pub inner: Connection<CloudflareSender>,
}

impl Database {
    /// Creates a new database connection to a libsql database
    /// 
    /// # Arguments
    /// 
    /// * `url` - The database URL (e.g., "libsql://your-db.turso.io")
    /// * `token` - The authentication token for the database
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing the `Database` instance or a `libsql::Error`
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// use libsql_orm::Database;
    /// 
    /// async fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let db = Database::new_connect(
    ///         "libsql://your-db.turso.io",
    ///         "your-auth-token"
    ///     ).await?;
    ///     println!("Connected to database successfully!");
    ///     Ok(())
    /// }
    /// ```
    pub async fn new_connect(url: &str, token: &str) -> std::result::Result<Self, libsql::Error> {
        let conn = Connection::open_cloudflare_worker(url.to_string(), token.to_string());
        match conn.execute("SELECT 1", ()).await {
            Ok(_) => Ok(Database { inner: conn }),
            Err(e) => Err(e),
        }
    }
    
    /// Gets a reference to the underlying libsql connection
    /// 
    /// This method provides direct access to the libsql connection for advanced use cases
    /// where you need to interact with the libsql API directly.
    /// 
    /// # Returns
    /// 
    /// A reference to the underlying `Connection<CloudflareSender>`
    pub fn get_connection(&self) -> &Connection<CloudflareSender> {
        &self.inner
    }
    
    /// Executes a SQL query with parameters
    /// 
    /// # Arguments
    /// 
    /// * `sql` - The SQL query string
    /// * `params` - Vector of parameters to bind to the query
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing `Rows` iterator or a `libsql::Error`
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// use libsql_orm::Database;
    /// 
    /// async fn query_example(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    ///     let rows = db.query(
    ///         "SELECT * FROM users WHERE age > ?", 
    ///         vec![libsql::Value::Integer(18)]
    ///     ).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn query(&self, sql: &str, params: Vec<libsql::Value>) -> Result<Rows, libsql::Error> {
        self.inner.query(sql, params).await
    }
}