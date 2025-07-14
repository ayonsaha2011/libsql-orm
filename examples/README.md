# libsql-orm Examples

This directory contains comprehensive examples demonstrating various features and use cases of libsql-orm.

## Examples Overview

### 1. **basic_crud.rs** - Basic CRUD Operations
Demonstrates fundamental Create, Read, Update, Delete operations with a simple User model.

**What you'll learn:**
- Connecting to a libsql database
- Creating, reading, updating, and deleting records
- Basic filtering and querying
- Error handling patterns

**Run with:**
```bash
LIBSQL_DATABASE_URL="your-url" LIBSQL_AUTH_TOKEN="your-token" cargo run --example basic_crud
```

### 2. **cloudflare_worker.rs** - Cloudflare Worker Integration
Shows how to build a complete REST API using libsql-orm in a Cloudflare Worker environment.

**What you'll learn:**
- Setting up libsql-orm in Cloudflare Workers
- Handling HTTP requests and responses
- Database migrations in edge environments
- RESTful API patterns
- Error handling and status codes

**Features demonstrated:**
- GET /posts - List posts with pagination
- GET /posts/{id} - Get specific post
- POST /posts - Create new post
- PUT /posts/{id} - Update existing post
- DELETE /posts/{id} - Delete post
- GET /posts/published - Filter published posts
- GET /posts/author/{author} - Filter by author

### 3. **advanced_queries.rs** - Advanced Querying
Comprehensive example of complex querying capabilities including filtering, pagination, searching, and aggregations.

**What you'll learn:**
- Complex filtering with AND/OR operations
- Range queries and comparisons
- Pagination and sorting
- Text search across multiple fields
- Aggregation functions (COUNT, SUM, AVG, MIN, MAX)
- Custom query building
- Bulk operations (create, update, delete)

### 4. **migrations.rs** - Database Schema Management
Demonstrates the migration system for managing database schema changes over time.

**What you'll learn:**
- Auto-generating migrations from models
- Manual migration creation with builders
- Template-based migrations for common operations
- Schema evolution (adding columns, indexes)
- Data transformation migrations
- Migration history and status tracking
- Batch migration execution

### 5. **table_name_macro.rs** - Custom Table Names
Shows how to use the `#[table_name("custom_name")]` attribute to specify custom table names.

**What you'll learn:**
- Using the table_name attribute macro
- Default vs custom table naming
- Best practices for table naming
- Legacy database integration
- Multi-tenant table prefixing

### 6. **column_attributes.rs** - Column Constraints and Properties
Demonstrates the `#[orm_column(...)]` attribute for defining custom column properties.

**What you'll learn:**
- Column type customization
- NOT NULL and UNIQUE constraints
- PRIMARY KEY and AUTOINCREMENT
- DEFAULT values and CHECK constraints
- Foreign key relationships
- Advanced SQL column definitions

## Prerequisites

Before running the examples, make sure you have:

1. **Environment Variables Set:**
   ```bash
   export LIBSQL_DATABASE_URL="libsql://your-database.turso.io"
   export LIBSQL_AUTH_TOKEN="your-auth-token"
   ```

2. **Rust and Cargo Installed:**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **libsql Database:** 
   - You can use [Turso](https://turso.tech/) for a managed libsql database
   - Or set up your own libsql instance

## Running Examples

### Individual Examples
```bash
# Basic CRUD operations
cargo run --example basic_crud

# Advanced querying
cargo run --example advanced_queries

# Database migrations
cargo run --example migrations

# Table name macros
cargo run --example table_name_macro

# Column attributes
cargo run --example column_attributes
```

### Cloudflare Worker Example
The Cloudflare Worker example is meant to be deployed to Cloudflare Workers. To test it locally:

1. Install Wrangler CLI:
   ```bash
   npm install -g wrangler
   ```

2. Create a `wrangler.toml` file in your project root:
   ```toml
   name = "libsql-orm-example"
   main = "examples/cloudflare_worker.rs"
   compatibility_date = "2024-07-14"
   
   [env.production.vars]
   LIBSQL_DATABASE_URL = "your-database-url"
   LIBSQL_AUTH_TOKEN = "your-auth-token"
   ```

3. Deploy to Cloudflare Workers:
   ```bash
   wrangler publish
   ```

## Example Structure

Each example follows a consistent structure:

1. **Imports and Setup** - Required dependencies and model definitions
2. **Database Connection** - Establishing connection with proper error handling
3. **Feature Demonstrations** - Step-by-step examples with explanations
4. **Cleanup** - Proper resource cleanup and data removal
5. **Error Handling** - Comprehensive error handling patterns

## Common Patterns

### Error Handling
All examples use `Result<(), Box<dyn std::error::Error>>` for comprehensive error handling:

```rust
async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::new_connect(&url, &token).await?;
    // ... operations
    Ok(())
}
```

### Model Definition
Consistent model patterns using derive macros:

```rust
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[table_name("custom_table")]
struct MyModel {
    #[orm_column(type = "INTEGER PRIMARY KEY AUTOINCREMENT")]
    pub id: Option<i64>,
    
    #[orm_column(not_null, unique)]
    pub name: String,
    
    #[orm_column(type = "TEXT NOT NULL")]
    pub created_at: DateTime<Utc>,
}
```

### Environment Configuration
All examples use environment variables for configuration:

```rust
let database_url = std::env::var("LIBSQL_DATABASE_URL")
    .expect("LIBSQL_DATABASE_URL must be set");
```

## Next Steps

After exploring these examples:

1. **Read the API Documentation** - Check out the complete API docs at [docs.rs/libsql-orm](https://docs.rs/libsql-orm)
2. **Build Your Own Application** - Use these patterns in your own projects
3. **Contribute** - Submit your own examples or improvements
4. **Join the Community** - Participate in discussions and get help

## Need Help?

- 📚 [Documentation](https://docs.rs/libsql-orm)
- 💬 [GitHub Discussions](https://github.com/ayonsaha2011/libsql-orm/discussions)
- 🐛 [Report Issues](https://github.com/ayonsaha2011/libsql-orm/issues)
- 📧 [Contact Author](mailto:ayonsaha2011@gmail.com)

Happy coding with libsql-orm! 🚀