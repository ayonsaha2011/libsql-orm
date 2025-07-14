use chrono::{DateTime, Utc};
use libsql_orm::{generate_migration, orm_column, Database, MigrationManager, Model};
use serde::{Deserialize, Serialize};
use std::env;

/// Column Attributes Example
///
/// This example demonstrates how to use the `#[orm_column(...)]` attribute
/// to specify custom column properties and constraints in a standalone application.
///
/// To run this example:
/// 1. Set environment variables:
///    export LIBSQL_DATABASE_URL="your_database_url"
///    export LIBSQL_AUTH_TOKEN="your_auth_token"
/// 2. Run: cargo run --example column_attributes

#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[table_name("advanced_users")]
struct AdvancedUser {
    #[orm_column(type = "INTEGER PRIMARY KEY AUTOINCREMENT")]
    pub id: Option<i64>,

    #[orm_column(not_null, unique)]
    pub username: String,

    #[orm_column(not_null, unique)]
    pub email: String,

    #[orm_column(not_null)]
    pub full_name: String,

    #[orm_column(type = "TEXT DEFAULT 'active'")]
    pub status: String,

    #[orm_column(type = "INTEGER DEFAULT 0")]
    pub login_count: i32,

    #[orm_column(type = "REAL DEFAULT 0.0")]
    pub account_balance: f64,

    #[orm_column(type = "BOOLEAN DEFAULT TRUE")]
    pub is_verified: bool,

    #[orm_column(type = "TEXT")]
    pub bio: Option<String>,

    #[orm_column(type = "TEXT NOT NULL")]
    pub created_at: DateTime<Utc>,

    #[orm_column(type = "TEXT")]
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[table_name("products_catalog")]
struct ProductCatalog {
    #[orm_column(primary_key, auto_increment)]
    pub id: Option<i64>,

    #[orm_column(not_null)]
    pub name: String,

    #[orm_column(unique)]
    pub sku: String,

    #[orm_column(type = "TEXT")]
    pub description: Option<String>,

    #[orm_column(type = "REAL NOT NULL CHECK(price >= 0)")]
    pub price: f64,

    #[orm_column(type = "INTEGER DEFAULT 0")]
    pub stock_quantity: i32,

    #[orm_column(type = "TEXT DEFAULT 'active'")]
    pub status: String,

    #[orm_column(type = "TEXT")]
    pub category: String,

    #[orm_column(type = "TEXT")]
    pub tags: Option<String>,

    #[orm_column(type = "REAL")]
    pub weight: Option<f64>,

    #[orm_column(type = "TEXT NOT NULL")]
    pub created_at: DateTime<Utc>,

    #[orm_column(type = "TEXT NOT NULL")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[table_name("orders")]
struct Order {
    #[orm_column(type = "INTEGER PRIMARY KEY AUTOINCREMENT")]
    pub id: Option<i64>,

    #[orm_column(unique)]
    pub order_number: String,

    #[orm_column(type = "INTEGER NOT NULL REFERENCES advanced_users(id)")]
    pub user_id: i64,

    #[orm_column(type = "REAL NOT NULL CHECK(total_amount >= 0)")]
    pub total_amount: f64,

    #[orm_column(type = "TEXT DEFAULT 'pending'")]
    pub status: String,

    #[orm_column(type = "TEXT")]
    pub shipping_address: String,

    #[orm_column(type = "TEXT")]
    pub billing_address: String,

    #[orm_column(type = "TEXT")]
    pub payment_method: String,

    #[orm_column(type = "TEXT")]
    pub notes: Option<String>,

    #[orm_column(type = "TEXT NOT NULL")]
    pub created_at: DateTime<Utc>,

    #[orm_column(type = "TEXT")]
    pub shipped_at: Option<DateTime<Utc>>,

    #[orm_column(type = "TEXT")]
    pub delivered_at: Option<DateTime<Utc>>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file or system (if available)
    // Note: Add dotenv dependency to Cargo.toml if you want to use .env files
    // Set environment variables:
    // export LIBSQL_DATABASE_URL="your_database_url"
    // export LIBSQL_AUTH_TOKEN="your_auth_token"

    // Connect to database
    let database_url = env::var("LIBSQL_DATABASE_URL")
        .expect("LIBSQL_DATABASE_URL environment variable must be set");
    let auth_token =
        env::var("LIBSQL_AUTH_TOKEN").expect("LIBSQL_AUTH_TOKEN environment variable must be set");

    let db = Database::new_connect(&database_url, &auth_token).await?;
    let manager = MigrationManager::new(db);

    println!("=== Column Attributes Example ===");

    // Initialize migration system
    manager.init().await?;

    // Generate and execute migrations
    println!("\n1. Creating Tables with Column Attributes:");

    let user_migration = generate_migration!(AdvancedUser);
    manager.execute_migration(&user_migration).await?;
    println!("✓ Created advanced_users table with column constraints");

    let product_migration = generate_migration!(ProductCatalog);
    manager.execute_migration(&product_migration).await?;
    println!("✓ Created products_catalog table with column constraints");

    let order_migration = generate_migration!(Order);
    manager.execute_migration(&order_migration).await?;
    println!("✓ Created orders table with column constraints");

    // Show generated SQL
    println!("\n2. Generated SQL with Column Attributes:");

    println!("AdvancedUser migration SQL:");
    println!("{}", AdvancedUser::migration_sql());
    println!();

    println!("ProductCatalog migration SQL:");
    println!("{}", ProductCatalog::migration_sql());
    println!();

    println!("Order migration SQL:");
    println!("{}", Order::migration_sql());
    println!();

    // Create sample data
    println!("3. Creating Sample Data with Constraints:");

    let db = manager.database();
    let now = Utc::now();

    // Create an advanced user
    let user = AdvancedUser {
        id: None,
        username: "advanced_user_001".to_string(),
        email: "advanced@example.com".to_string(),
        full_name: "Advanced User".to_string(),
        status: "premium".to_string(),
        login_count: 5,
        account_balance: 150.75,
        is_verified: true,
        bio: Some("I'm an advanced user of this system.".to_string()),
        created_at: now,
        last_login: Some(now),
    };

    let created_user = user.create(db).await?;
    println!("✓ Created advanced user: {}", created_user.username);

    // Create products
    let products = vec![
        ProductCatalog {
            id: None,
            name: "Premium Laptop".to_string(),
            sku: "LAPTOP-001".to_string(),
            description: Some("High-performance laptop for professionals".to_string()),
            price: 1299.99,
            stock_quantity: 25,
            status: "active".to_string(),
            category: "Electronics".to_string(),
            tags: Some("laptop,computer,premium".to_string()),
            weight: Some(2.5),
            created_at: now,
            updated_at: now,
        },
        ProductCatalog {
            id: None,
            name: "Wireless Mouse".to_string(),
            sku: "MOUSE-002".to_string(),
            description: Some("Ergonomic wireless mouse".to_string()),
            price: 49.99,
            stock_quantity: 100,
            status: "active".to_string(),
            category: "Accessories".to_string(),
            tags: Some("mouse,wireless,ergonomic".to_string()),
            weight: Some(0.15),
            created_at: now,
            updated_at: now,
        },
    ];

    let created_products = ProductCatalog::bulk_create(&products, db).await?;
    println!(
        "✓ Created {} products with constraints",
        created_products.len()
    );

    // Create an order
    let order = Order {
        id: None,
        order_number: format!("ORD-{}", now.timestamp()),
        user_id: created_user.id.unwrap(),
        total_amount: 1349.98,
        status: "confirmed".to_string(),
        shipping_address: "123 Main St, City, State 12345".to_string(),
        billing_address: "123 Main St, City, State 12345".to_string(),
        payment_method: "credit_card".to_string(),
        notes: Some("Express delivery requested".to_string()),
        created_at: now,
        shipped_at: None,
        delivered_at: None,
    };

    let created_order = order.create(db).await?;
    println!("✓ Created order: {}", created_order.order_number);

    // Query and display data
    println!("\n4. Querying Data with Constraints:");

    let users = AdvancedUser::find_all(db).await?;
    println!("Found {} advanced users", users.len());
    for user in users {
        println!(
            "  User: {} ({}) - Balance: ${:.2}",
            user.username, user.status, user.account_balance
        );
    }

    let products = ProductCatalog::find_all(db).await?;
    println!("Found {} products in catalog", products.len());
    for product in products {
        println!(
            "  Product: {} - ${:.2} (Stock: {})",
            product.name, product.price, product.stock_quantity
        );
    }

    let orders = Order::find_all(db).await?;
    println!("Found {} orders", orders.len());
    for order in orders {
        println!(
            "  Order: {} - Total: ${:.2} ({})",
            order.order_number, order.total_amount, order.status
        );
    }

    // Demonstrate column attributes benefits
    println!("\n5. Column Attributes Benefits:");
    println!("✓ NOT NULL constraints ensure data integrity");
    println!("✓ UNIQUE constraints prevent duplicate values");
    println!("✓ DEFAULT values provide sensible fallbacks");
    println!("✓ CHECK constraints validate data ranges");
    println!("✓ FOREIGN KEY references maintain relationships");
    println!("✓ PRIMARY KEY and AUTOINCREMENT handle ID generation");

    println!("\n6. Supported Column Attributes:");
    println!("• type = \"SQL_TYPE\" - Custom SQL type definition");
    println!("• not_null - Add NOT NULL constraint");
    println!("• unique - Add UNIQUE constraint");
    println!("• primary_key - Mark as PRIMARY KEY");
    println!("• auto_increment - Add AUTOINCREMENT");
    println!("• Custom SQL in type attribute for advanced constraints");

    println!("\n=== Column Attributes Example Complete ===");
    println!("All tables created with appropriate column constraints!");

    Ok(())
}
