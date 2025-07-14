/// Database Migrations Example
/// 
/// This example demonstrates how to use the migration system to manage
/// database schema changes over time in a Cloudflare Worker environment.

use worker::*;
use libsql_orm::{
    Database, MigrationManager, MigrationBuilder, Migration, 
    Model, generate_migration, templates
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// Version 1: Initial User model
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
struct User {
    pub id: Option<i64>,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

// Version 2: User model with additional fields
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
struct UserV2 {
    pub id: Option<i64>,
    pub name: String,
    pub email: String,
    pub age: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Connect to database
    let database_url = env.var("LIBSQL_DATABASE_URL")?.to_string();
    let auth_token = env.var("LIBSQL_AUTH_TOKEN")?.to_string();
    
    let db = Database::new_connect(&database_url, &auth_token).await
        .map_err(|e| worker::Error::RustError(format!("Database connection failed: {}", e)))?;
    let manager = MigrationManager::new(db);
    
    println!("=== Database Migrations Example ===");
    
    // Initialize the migration system
    println!("Initializing migration system...");
    manager.init().await?;
    
    // 1. Auto-generated migration from model
    println!("\n1. Auto-generated Migration:");
    let user_migration = generate_migration!(User);
    println!("Generated migration for User model");
    
    // Execute the migration
    manager.execute_migration(&user_migration).await?;
    println!("Executed User table migration");
    
    // 2. Manual migration using builder
    println!("\n2. Manual Migration with Builder:");
    let add_index_migration = MigrationBuilder::new("add_email_index")
        .up("CREATE UNIQUE INDEX idx_users_email ON users(email)")
        .down("DROP INDEX idx_users_email")
        .build();
    
    manager.execute_migration(&add_index_migration).await?;
    println!("Added unique index on email column");
    
    // 3. Template-based migrations
    println!("\n3. Template-based Migrations:");
    
    // Create a posts table using template
    let posts_migration = templates::create_table("posts", &[
        ("id", "INTEGER PRIMARY KEY AUTOINCREMENT"),
        ("title", "TEXT NOT NULL"),
        ("content", "TEXT NOT NULL"),
        ("user_id", "INTEGER NOT NULL"),
        ("published", "BOOLEAN DEFAULT FALSE"),
        ("created_at", "TEXT NOT NULL"),
        ("FOREIGN KEY (user_id)", "REFERENCES users(id)"),
    ]);
    
    manager.execute_migration(&posts_migration).await?;
    println!("Created posts table with foreign key");
    
    // Add an index on posts
    let posts_index = templates::create_index("idx_posts_user_id", "posts", &["user_id"]);
    manager.execute_migration(&posts_index).await?;
    println!("Added index on posts.user_id");
    
    // 4. Schema evolution - adding columns
    println!("\n4. Schema Evolution:");
    
    // Add age column to users
    let add_age_migration = templates::add_column("users", "age", "INTEGER");
    manager.execute_migration(&add_age_migration).await?;
    println!("Added age column to users table");
    
    // Add is_active column
    let add_active_migration = templates::add_column("users", "is_active", "BOOLEAN DEFAULT TRUE");
    manager.execute_migration(&add_active_migration).await?;
    println!("Added is_active column to users table");
    
    // Add updated_at column
    let add_updated_migration = templates::add_column("users", "updated_at", "TEXT");
    manager.execute_migration(&add_updated_migration).await?;
    println!("Added updated_at column to users table");
    
    // 5. Complex migration with data transformation
    println!("\n5. Complex Migration with Data Transformation:");
    let data_migration = MigrationBuilder::new("set_default_values")
        .up(r#"
            UPDATE users 
            SET 
                is_active = TRUE,
                updated_at = created_at
            WHERE is_active IS NULL OR updated_at IS NULL
        "#)
        .build();
    
    manager.execute_migration(&data_migration).await?;
    println!("Set default values for new columns");
    
    // 6. Migration status and history
    println!("\n6. Migration Status:");
    let all_migrations = manager.get_migrations().await?;
    println!("Total migrations: {}", all_migrations.len());
    
    let executed_migrations = manager.get_executed_migrations().await?;
    println!("Executed migrations: {}", executed_migrations.len());
    
    let pending_migrations = manager.get_pending_migrations().await?;
    println!("Pending migrations: {}", pending_migrations.len());
    
    // Display migration history
    println!("\nMigration History:");
    for migration in executed_migrations {
        println!("  ✓ {} (executed at: {})", 
            migration.name, 
            migration.executed_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "unknown".to_string())
        );
    }
    
    // 7. Batch migration execution
    println!("\n7. Batch Migration Execution:");
    
    let batch_migrations = vec![
        MigrationBuilder::new("add_user_profile_table")
            .up(r#"
                CREATE TABLE user_profiles (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    user_id INTEGER NOT NULL UNIQUE,
                    bio TEXT,
                    avatar_url TEXT,
                    website TEXT,
                    created_at TEXT NOT NULL,
                    FOREIGN KEY (user_id) REFERENCES users(id)
                )
            "#)
            .build(),
        
        templates::create_index("idx_user_profiles_user_id", "user_profiles", &["user_id"]),
        
        MigrationBuilder::new("add_categories_table")
            .up(r#"
                CREATE TABLE categories (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    description TEXT,
                    created_at TEXT NOT NULL
                )
            "#)
            .build(),
    ];
    
    manager.run_migrations(batch_migrations).await?;
    println!("Executed batch of {} migrations", 3);
    
    // 8. Testing with sample data
    println!("\n8. Testing Schema with Sample Data:");
    
    // Insert a test user using the evolved schema
    let now = Utc::now().to_rfc3339();
    let create_user_sql = r#"
        INSERT INTO users (name, email, age, is_active, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
    "#;
    
    manager.database().inner.execute(create_user_sql, vec![
        libsql::Value::Text("John Doe".to_string()),
        libsql::Value::Text("john.doe@example.com".to_string()),
        libsql::Value::Integer(30),
        libsql::Value::Integer(1), // true as integer
        libsql::Value::Text(now.clone()),
        libsql::Value::Text(now),
    ]).await?;
    
    println!("Successfully inserted test user with evolved schema");
    
    // Query to verify the schema works
    let query_sql = "SELECT name, email, age, is_active FROM users WHERE email = ?";
    let mut rows = manager.database().inner.query(query_sql, vec![
        libsql::Value::Text("john.doe@example.com".to_string())
    ]).await?;
    
    if let Some(row) = rows.next().await? {
        let name: String = row.get(0)?;
        let email: String = row.get(1)?;
        let age: Option<i64> = row.get(2)?;
        let is_active: i64 = row.get(3)?;
        
        println!("Retrieved user: {} <{}>, age: {:?}, active: {}", 
            name, email, age, is_active == 1);
    }
    
    println!("\n=== Migration Example Complete ===");
    println!("Schema has been successfully evolved through multiple migrations!");
    
    Response::ok("✅ Database migrations completed successfully!")
}