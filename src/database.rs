//! Database connection and query execution
//!
//! This module handles the connection to libsql databases and provides
//! query execution capabilities for Cloudflare Workers.

#[cfg(target_arch = "wasm32")]
use libsql::wasm::{CloudflareSender, Connection, Rows};
#[cfg(not(target_arch = "wasm32"))]
use libsql::{Builder, Connection, Rows};

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
    #[cfg(target_arch = "wasm32")]
    pub inner: Connection<CloudflareSender>,
    #[cfg(not(target_arch = "wasm32"))]
    pub inner: Connection,
}

#[cfg(target_arch = "wasm32")]
impl From<Connection<CloudflareSender>> for Database {
    fn from(inner: Connection<CloudflareSender>) -> Self {
        Self { inner }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<Connection> for Database {
    fn from(inner: Connection) -> Self {
        Self { inner }
    }
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
        #[cfg(target_arch = "wasm32")]
        let conn = Connection::open_cloudflare_worker(url.to_string(), token.to_string());
        #[cfg(not(target_arch = "wasm32"))]
        let conn = Builder::new_remote(url.to_string(), token.to_string())
            .build()
            .await?
            .connect()?;
        conn.execute("SELECT 1", ()).await.map(|_| Self::from(conn))
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
    pub async fn query(
        &self,
        sql: &str,
        params: Vec<libsql::Value>,
    ) -> Result<Rows, libsql::Error> {
        self.inner.query(sql, params).await
    }
}
