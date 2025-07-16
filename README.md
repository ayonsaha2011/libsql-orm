# libsql-orm

[![Crates.io](https://img.shields.io/crates/v/libsql-orm.svg)](https://crates.io/crates/libsql-orm)
[![Documentation](https://docs.rs/libsql-orm/badge.svg)](https://docs.rs/libsql-orm)
[![License](https://img.shields.io/crates/l/libsql-orm.svg)](LICENSE)
[![Build Status](https://github.com/ayonsaha2011/libsql-orm/workflows/CI/badge.svg)](https://github.com/ayonsaha2011/libsql-orm/actions)

A powerful, async-first ORM for [libsql](https://github.com/libsql/libsql) with first-class support for **Cloudflare Workers** and WebAssembly environments.

> ⚠️ **Disclaimer**: This library is in early development and not fully tested in production environments. Use at your own risk. Please report any issues you encounter and feel free to contribute via pull requests - we're happy to address them and welcome community contributions!

## ✨ Features

- 🚀 **Cloudflare Workers Ready** - Built specifically for edge computing environments
- 🔄 **Async/Await Support** - Fully async API with excellent performance
- 🎯 **Type-Safe** - Leverages Rust's type system for compile-time safety
- 📊 **Rich Query Builder** - Fluent API for complex queries
- 🔍 **Advanced Filtering** - Search, pagination, sorting, and aggregations
- 🎨 **Derive Macros** - Automatic model generation with `#[derive(Model)]`
- 📦 **Bulk Operations** - Efficient batch inserts, updates, and deletes
- 🌐 **WASM Compatible** - Optimized for WebAssembly targets
- 🔧 **Custom Table Names** - `#[table_name("custom")]` attribute support
- ✅ **Boolean Type Safety** - Automatic SQLite integer ↔ Rust boolean conversion
- 🏷️ **Column Attributes** - `#[orm_column(...)]` for column customization
- 🔄 **Upsert Operations** - Smart create_or_update and upsert methods

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
use libsql_orm::{Model, Database, FilterOperator, Value};
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
    
    // Find users
    let users = User::find_all(&db).await?;
    
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
use libsql_orm::{Model, FilterOperator, Value};
use serde::{Serialize, Deserialize};

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
use libsql_orm::Model;
use serde::{Serialize, Deserialize};

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

// Access pagination info
// Page: result.pagination.page
// Total pages: result.pagination.total_pages.unwrap_or(0)
// Total items: result.pagination.total.unwrap_or(0)
for user in result.data {
    // Process user: user.name
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
use chrono::{DateTime, Utc};

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

## 🏗️ Architecture

### WASM Compatibility

libsql-orm is built from the ground up for WebAssembly environments:

- Uses `libsql` WASM bindings for database connectivity
- Optimized async runtime for edge computing
- Minimal binary size with selective feature compilation
- Compatible with Cloudflare Workers, Deno Deploy, and other edge platforms


## 🔗 Ecosystem

libsql-orm works great with:

- **[libsql](https://github.com/libsql/libsql)** - The database engine
- **[Turso](https://turso.tech/)** - Managed libsql hosting
- **[Cloudflare Workers](https://workers.cloudflare.com/)** - Edge computing platform
- **[worker-rs](https://github.com/cloudflare/workers-rs)** - Cloudflare Workers Rust SDK

## 🤝 Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## ☕ Support the Project

If you find this library helpful and would like to support its development, consider making a donation:

### 💰 Donation Options

- **GitHub Sponsors**: [Sponsor on GitHub](https://github.com/sponsors/ayonsaha2011)
- **Buy Me a Coffee**: [Buy me a coffee](https://coff.ee/ayonsaha2011)
- **PayPal**: [PayPal Donation](https://paypal.me/ayonsaha)

### 🎯 What Your Support Helps With

- 🚀 **Feature Development** - Building new capabilities and improvements
- 🐛 **Bug Fixes** - Maintaining stability and reliability  
- 📚 **Documentation** - Creating better guides and examples
- 🔧 **Maintenance** - Keeping the library up-to-date with dependencies
- ☁️ **Infrastructure** - Hosting costs for CI/CD and testing

Every contribution, no matter the size, helps make this library better for everyone! 🙏

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [libsql team](https://github.com/libsql/libsql) for the excellent database engine
- [Cloudflare](https://cloudflare.com) for the Workers platform
- Rust community for the amazing ecosystem

---
**Need help?** 
- 📚 [Documentation](https://docs.rs/libsql-orm)
- 💬 [Discussions](https://github.com/ayonsaha2011/libsql-orm/discussions)
- 🐛 [Issues](https://github.com/ayonsaha2011/libsql-orm/issues)

