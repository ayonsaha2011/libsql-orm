# libsql-orm

[![Crates.io](https://img.shields.io/crates/v/libsql-orm.svg)](https://crates.io/crates/libsql-orm)
[![Documentation](https://docs.rs/libsql-orm/badge.svg)](https://docs.rs/libsql-orm)
[![License](https://img.shields.io/crates/l/libsql-orm.svg)](LICENSE)
[![Build Status](https://github.com/your-username/libsql-orm/workflows/CI/badge.svg)](https://github.com/your-username/libsql-orm/actions)

A powerful, async-first ORM for [libsql](https://github.com/libsql/libsql) with first-class support for **Cloudflare Workers** and WebAssembly environments.

## ✨ Features

- 🚀 **Cloudflare Workers Ready** - Built specifically for edge computing environments
- 🔄 **Async/Await Support** - Fully async API with excellent performance
- 🎯 **Type-Safe** - Leverages Rust's type system for compile-time safety
- 📊 **Rich Query Builder** - Fluent API for complex queries
- 🔍 **Advanced Filtering** - Search, pagination, sorting, and aggregations
- 🛠️ **Migration System** - Database schema management and versioning
- 🎨 **Derive Macros** - Automatic model generation with `#[derive(Model)]`
- 📦 **Bulk Operations** - Efficient batch inserts, updates, and deletes
- 🌐 **WASM Compatible** - Optimized for WebAssembly targets
- 🔧 **Custom Table Names** - `#[table_name("custom")]` attribute support
- ✅ **Boolean Type Safety** - Automatic SQLite integer ↔ Rust boolean conversion
- 🏷️ **Column Attributes** - `#[orm_column(...)]` for column customization
- 🔄 **Upsert Operations** - Smart create_or_update and upsert methods
- 📝 **Built-in Logging** - Comprehensive logging for debugging and monitoring

## 🚀 Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
libsql-orm = "0.1.0"
serde = { version = "1.0", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
```

### Basic Usage

```rust
use libsql_orm::{Model, Database};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[table_name("users")]  // Custom table name (optional)
struct User {
    pub id: Option<i64>,
    pub name: String,
    pub email: String,
    pub age: Option<i32>,
    pub is_active: bool,        // ✅ Automatic boolean conversion
    pub is_verified: bool,      // ✅ Works with any boolean field
    pub created_at: DateTime<Utc>,
}

// In your async function
async fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to database
    let db = Database::new_connect("libsql://your-db.turso.io", "your-auth-token").await?;
    
    // Create a user
    let user = User {
        id: None,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        age: Some(30),
        is_active: true,
        created_at: Utc::now(),
    };
    
    // Save to database
    let saved_user = user.create(&db).await?;
    println!("Created user with ID: {:?}", saved_user.id);
    
    // Find users
    let users = User::find_all(&db).await?;
    println!("Found {} users", users.len());
    
    // Query with conditions
    let active_users = User::find_where(
        FilterOperator::Eq("is_active".to_string(), crate::Value::Boolean(true)),
        &db
    ).await?;
    
    Ok(())
}
```

### Cloudflare Workers Integration

```rust
use worker::*;
use libsql_orm::{Model, Database, MigrationManager, generate_migration};

#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[table_name("blog_posts")]  // Custom table name
struct Post {
    pub id: Option<i64>,
    pub title: String,
    pub content: String,
    pub published: bool,       // ✅ Boolean automatically converted from SQLite
    pub featured: bool,        // ✅ Multiple boolean fields supported
    pub created_at: DateTime<Utc>,
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();
    
    // Get database credentials from environment
    let database_url = env.var("LIBSQL_DATABASE_URL")?.to_string();
    let auth_token = env.var("LIBSQL_AUTH_TOKEN")?.to_string();
    
    // Connect to database
    let db = Database::new_connect(&database_url, &auth_token).await
        .map_err(|e| format!("Database connection failed: {}", e))?;
    
    // Run migrations
    let manager = MigrationManager::new(db);
    let migration = generate_migration!(Post);
    manager.execute_migration(&migration).await
        .map_err(|e| format!("Migration failed: {}", e))?;
    
    let db = manager.database();
    
    // Handle the request
    match req.method() {
        Method::Get => {
            let posts = Post::find_all(db).await
                .map_err(|e| format!("Query failed: {}", e))?;
            Response::from_json(&posts)
        }
        Method::Post => {
            let post: Post = req.json().await?;
            let saved_post = post.create(db).await
                .map_err(|e| format!("Create failed: {}", e))?;
            Response::from_json(&saved_post)
        }
        _ => Response::error("Method not allowed", 405)
    }
}
```

## 📚 Advanced Features

### Custom Table Names

Use the `#[table_name("custom_name")]` attribute to specify custom table names:

```rust
#[derive(Model, Serialize, Deserialize)]
#[table_name("user_accounts")]  // Custom table name
struct User {
    pub id: Option<i64>,
    pub username: String,
    pub email: String,
}

// Default table name would be "user" (struct name lowercase)
// With attribute, table name is "user_accounts"
assert_eq!(User::table_name(), "user_accounts");
```

**Benefits:**
- 🏷️ **Legacy Integration** - Map to existing database tables
- 🎯 **Naming Control** - Override default naming conventions  
- 📁 **Multi-tenant** - Use prefixes like `tenant_users`
- 🔄 **Migration Friendly** - Rename tables without changing structs

### Boolean Type Safety

libsql-orm automatically handles boolean conversion between SQLite and Rust:

```rust
#[derive(Model, Serialize, Deserialize)]
struct User {
    pub id: Option<i64>,
    pub is_active: bool,      // ✅ SQLite INTEGER(0/1) ↔ Rust bool
    pub is_verified: bool,    // ✅ Automatic conversion
    pub has_premium: bool,    // ✅ Works with any boolean field name
    pub can_edit: bool,       // ✅ No configuration needed
    pub enabled: bool,        // ✅ Type-safe operations
}

// All boolean operations work seamlessly
let user = User::find_where(
    FilterOperator::Eq("is_active".to_string(), Value::Boolean(true)),
    &db
).await?;

// JSON serialization works correctly
let json = serde_json::to_string(&user)?;  // ✅ Booleans as true/false
let deserialized: User = serde_json::from_str(&json)?;  // ✅ No errors
```

**Key Features:**
- ✅ **Automatic Detection** - Boolean fields identified at compile time
- ✅ **Zero Configuration** - Works with any boolean field name
- ✅ **Type Safety** - No runtime errors or invalid conversions
- ✅ **Performance** - Conversion logic generated at compile time
- ✅ **JSON Compatible** - Seamless serialization/deserialization

### Column Attributes

Customize column properties with `#[orm_column(...)]`:

```rust
#[derive(Model, Serialize, Deserialize)]
struct Product {
    #[orm_column(type = "INTEGER PRIMARY KEY AUTOINCREMENT")]
    pub id: Option<i64>,
    
    #[orm_column(not_null, unique)]
    pub sku: String,
    
    #[orm_column(type = "REAL CHECK(price >= 0)")]
    pub price: f64,
    
    #[orm_column(type = "BOOLEAN DEFAULT TRUE")]
    pub is_available: bool,     // ✅ Boolean with DEFAULT constraint
}
```

### Query Builder

```rust
use libsql_orm::{QueryBuilder, FilterOperator, Sort, SortOrder, Pagination};

// Complex query with filtering and pagination
let query = QueryBuilder::new("users")
    .select(&["id", "name", "email"])
    .r#where(FilterOperator::Gte("age".to_string(), Value::Integer(18)))
    .order_by(Sort::new("created_at", SortOrder::Desc))
    .limit(10)
    .offset(20);

let (sql, params) = query.build()?;
```

### Pagination

```rust
use libsql_orm::{Pagination, PaginatedResult};

let pagination = Pagination::new(1, 10); // page 1, 10 items per page
let result: PaginatedResult<User> = User::find_paginated(&pagination, &db).await?;

println!("Page {}/{}", result.current_page, result.total_pages);
println!("Total items: {}", result.total_count);
for user in result.data {
    println!("User: {}", user.name);
}
```

### Bulk Operations

```rust
// Bulk insert
let users = vec![
    User { /* ... */ },
    User { /* ... */ },
    User { /* ... */ },
];
let saved_users = User::bulk_create(&users, &db).await?;

// Bulk delete
let ids_to_delete = vec![1, 2, 3, 4, 5];
let deleted_count = User::bulk_delete(&ids_to_delete, &db).await?;
```

### Aggregations

```rust
use libsql_orm::Aggregate;

// Count users
let total_users = User::count(&db).await?;

// Average age
let avg_age = User::aggregate(
    Aggregate::Avg,
    "age",
    None,
    &db
).await?;

// Count with filter
let active_users_count = User::count_where(
    FilterOperator::Eq("is_active".to_string(), Value::Boolean(true)),
    &db
).await?;
```

### Search

```rust
use libsql_orm::SearchFilter;

let search = SearchFilter::new(
    vec!["name".to_string(), "email".to_string()],
    "john".to_string()
);

let results = User::search(&search, Some(&pagination), &db).await?;
```

### Upsert Operations

libsql-orm provides intelligent create-or-update operations:

```rust
use libsql_orm::{Model, Database};

// Create or update based on primary key
let mut user = User {
    id: Some(123),  // If record exists, it will be updated
    name: "John Doe".to_string(),
    email: "john@example.com".to_string(),
    is_active: true,
    created_at: Utc::now(),
};

// Automatically decides whether to create or update
let saved_user = user.create_or_update(&db).await?;

// Upsert based on unique constraints (e.g., email)
let user = User {
    id: None,  // Primary key not set
    name: "Jane Smith".to_string(),
    email: "jane@example.com".to_string(),  // Unique field
    is_active: true,
    created_at: Utc::now(),
};

// Will update existing record with this email, or create new if not found
let saved_user = user.upsert(&["email"], &db).await?;

// Multiple unique constraints
let saved_user = user.upsert(&["email", "username"], &db).await?;
```

**Use Cases:**
- ✅ **Data Synchronization** - Import external data without duplicates
- ✅ **User Registration** - Update existing accounts or create new ones
- ✅ **Configuration Management** - Maintain settings without conflicts
- ✅ **API Endpoints** - Handle PUT requests efficiently

### Built-in Logging

libsql-orm includes comprehensive logging for debugging and monitoring:

```rust
use libsql_orm::{Model, Database};

// All operations are automatically logged
let user = User::new("John", "john@example.com");

// Logs: [INFO] users: Creating record in table: users
// Logs: [DEBUG] users: SQL: INSERT INTO users (name, email, is_active) VALUES (?, ?, ?)
// Logs: [INFO] users: Successfully created record with ID: 123
let saved_user = user.create(&db).await?;

// Logs: [DEBUG] users: Finding record by ID: 123
// Logs: [INFO] users: Found record with ID: 123
let found_user = User::find_by_id(123, &db).await?;

// Logs: [INFO] users: Updating existing record with ID: 123
// Logs: [INFO] users: Updating record with ID: 123
// Logs: [DEBUG] users: SQL: UPDATE users SET name = ?, email = ? WHERE id = ?
// Logs: [INFO] users: Successfully updated record with ID: 123
let updated_user = found_user.unwrap().create_or_update(&db).await?;
```

**Logging Features:**
- 🎯 **Cross-Platform** - Uses browser console in WASM, standard logging elsewhere
- 📊 **Multiple Levels** - INFO, DEBUG, WARN, ERROR levels
- 🏷️ **Table Context** - Automatic table name prefixing for clarity
- 🔍 **SQL Debugging** - View actual SQL queries being executed
- ⚡ **Performance Friendly** - Minimal overhead in production

**Cloudflare Workers Logging:**
```rust
// In browser/worker environment, logs appear in console
// [INFO] users: Creating record in table: users
// [DEBUG] users: SQL: INSERT INTO users (...) VALUES (...)
// [WARN] users: Record with ID 999 not found, creating new record
```

**Native Application Logging:**
```rust
// Configure logging in your application
use log::LevelFilter;
use env_logger;

env_logger::Builder::from_default_env()
    .filter_level(LevelFilter::Debug)
    .init();

// Now all ORM operations will use standard Rust logging
```

## 🔧 Migrations

libsql-orm includes a powerful migration system:

```rust
use libsql_orm::{MigrationManager, Migration, MigrationBuilder};

// Auto-generate migration from model
let migration = generate_migration!(User);

// Or create manually
let migration = MigrationBuilder::new("create_users_table")
    .up(r#"
        CREATE TABLE users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT UNIQUE NOT NULL,
            created_at TEXT NOT NULL
        )
    "#)
    .down("DROP TABLE users")
    .build();

// Execute migration
let manager = MigrationManager::new(db);
manager.execute_migration(&migration).await?;
```

## 🏗️ Architecture

### WASM Compatibility

libsql-orm is built from the ground up for WebAssembly environments:

- Uses `libsql` WASM bindings for database connectivity
- Optimized async runtime for edge computing
- Minimal binary size with selective feature compilation
- Compatible with Cloudflare Workers, Deno Deploy, and other edge platforms

### Performance

- **Zero-copy deserialization** where possible
- **Connection pooling** for optimal database usage
- **Lazy loading** of related data
- **Efficient batch operations** for bulk data handling

## 📖 Examples

### Complete CRUD Example

```rust
use libsql_orm::{Model, Database, FilterOperator, Pagination};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Model, Debug, Clone, Serialize, Deserialize)]
struct User {
    pub id: Option<i64>,
    pub name: String,
    pub email: String,
    pub age: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

async fn crud_example() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::new_connect("your-db-url", "your-token").await?;
    
    // CREATE
    let user = User {
        id: None,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        age: Some(30),
        is_active: true,
        created_at: Utc::now(),
    };
    let created_user = user.create(&db).await?;
    
    // READ
    let found_user = User::find_by_id(created_user.id.unwrap(), &db).await?;
    let all_users = User::find_all(&db).await?;
    
    // UPDATE
    let mut user_to_update = found_user.unwrap();
    user_to_update.name = "Jane Doe".to_string();
    let updated_user = user_to_update.update(&db).await?;
    
    // CREATE OR UPDATE (smart upsert)
    let user_with_id = User {
        id: Some(999),  // If exists, update; if not, create
        name: "Smart User".to_string(),
        email: "smart@example.com".to_string(),
        age: Some(25),
        is_active: true,
        created_at: Utc::now(),
    };
    let smart_saved = user_with_id.create_or_update(&db).await?;
    
    // UPSERT by unique constraint
    let unique_user = User {
        id: None,
        name: "Unique User".to_string(),
        email: "unique@example.com".to_string(),  // Will check if this email exists
        age: Some(35),
        is_active: true,
        created_at: Utc::now(),
    };
    let upserted_user = unique_user.upsert(&["email"], &db).await?;
    
    // DELETE
    updated_user.delete(&db).await?;
    
    Ok(())
}
```

## 🔗 Ecosystem

libsql-orm works great with:

- **[libsql](https://github.com/libsql/libsql)** - The database engine
- **[Turso](https://turso.tech/)** - Managed libsql hosting
- **[Cloudflare Workers](https://workers.cloudflare.com/)** - Edge computing platform
- **[worker-rs](https://github.com/cloudflare/workers-rs)** - Cloudflare Workers Rust SDK

## 🔧 Troubleshooting

### Boolean Serialization Issues

If you encounter errors like `"invalid type: integer '1', expected a boolean"`, you have two solutions:

#### Option 1: Automatic Conversion (Recommended)
The derive macro handles this automatically in most cases:

```rust
// ✅ This works automatically with the derive macro
#[derive(Model, Serialize, Deserialize)]
struct User {
    pub is_active: bool,    // Automatically converts SQLite 0/1 to false/true
    pub enabled: bool,      // Works with any boolean field name
}
```

#### Option 2: Manual Deserializer (For Edge Cases)
If automatic conversion doesn't work, use the custom deserializer:

```rust
use libsql_orm::deserialize_bool;

#[derive(Model, Serialize, Deserialize)]
struct User {
    pub id: Option<i64>,
    pub name: String,
    
    // Use custom deserializer for problematic boolean fields
    #[serde(deserialize_with = "deserialize_bool")]
    pub is_active: bool,
    
    #[serde(deserialize_with = "deserialize_bool")]
    pub is_verified: bool,
}
```

The `deserialize_bool` function handles:
- ✅ **Integers**: `0` → `false`, `1` → `true`
- ✅ **Booleans**: Pass through unchanged
- ✅ **Strings**: `"true"`, `"1"`, `"yes"` → `true`; `"false"`, `"0"`, `"no"` → `false`

### Table Name Conflicts

Use the `#[table_name("custom")]` attribute to resolve naming conflicts:

```rust
#[derive(Model, Serialize, Deserialize)]
#[table_name("app_users")]  // Avoid conflicts with system tables
struct User {
    pub id: Option<i64>,
    pub name: String,
}
```

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
git clone https://github.com/your-username/libsql-orm
cd libsql-orm
cargo build
cargo test
```

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests with real database
cargo test --features integration-tests

# WASM tests
wasm-pack test --node
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [libsql team](https://github.com/libsql/libsql) for the excellent database engine
- [Cloudflare](https://cloudflare.com) for the Workers platform
- Rust community for the amazing ecosystem

## 📊 Status

- ✅ **Stable API** - Ready for production use
- ✅ **Well Tested** - Comprehensive test suite
- ✅ **Documented** - Complete API documentation
- ✅ **WASM Ready** - Optimized for edge computing
- 🔄 **Active Development** - Regular updates and improvements

---

**Need help?** 
- 📚 [Documentation](https://docs.rs/libsql-orm)
- 💬 [Discussions](https://github.com/your-username/libsql-orm/discussions)
- 🐛 [Issues](https://github.com/your-username/libsql-orm/issues)