use chrono::{DateTime, Utc};
use libsql_orm::{generate_migration, Database, MigrationManager, Model};
use serde::{Deserialize, Serialize};
use std::env;

/// Table Name Macro Example
///
/// This example demonstrates how to use the `#[table_name("custom_name")]` attribute
/// to specify custom table names for your models.

// Default table name (struct name converted to lowercase)
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
struct Product {
    pub id: Option<i64>,
    pub name: String,
    pub price: f64,
    pub created_at: DateTime<Utc>,
}

// Custom table name using the table_name attribute
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[table_name("user_accounts")]
struct User {
    pub id: Option<i64>,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// Another example with a descriptive table name
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[table_name("blog_posts")]
struct BlogPost {
    pub id: Option<i64>,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub author_id: i64,
    pub published: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Legacy table name mapping
#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[table_name("legacy_customer_data")]
struct Customer {
    pub id: Option<i64>,
    pub customer_code: String,
    pub company_name: String,
    pub contact_email: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to database using environment variables
    let database_url = env::var("LIBSQL_DATABASE_URL")
        .expect("LIBSQL_DATABASE_URL environment variable is required");
    let auth_token =
        env::var("LIBSQL_AUTH_TOKEN").expect("LIBSQL_AUTH_TOKEN environment variable is required");

    let db = Database::new_connect(&database_url, &auth_token).await?;
    let manager = MigrationManager::new(db);

    println!("=== Table Name Macro Example ===");

    // Initialize migration system
    manager.init().await?;

    // Demonstrate table name resolution
    println!("\n1. Table Name Resolution:");
    println!("Product struct -> table: '{}'", Product::table_name());
    println!(
        "User struct with #[table_name(\"user_accounts\")] -> table: '{}'",
        User::table_name()
    );
    println!(
        "BlogPost struct with #[table_name(\"blog_posts\")] -> table: '{}'",
        BlogPost::table_name()
    );
    println!(
        "Customer struct with #[table_name(\"legacy_customer_data\")] -> table: '{}'",
        Customer::table_name()
    );

    // Generate and execute migrations for each model
    println!("\n2. Creating Tables with Custom Names:");

    // Product table (default naming)
    let product_migration = generate_migration!(Product);
    manager.execute_migration(&product_migration).await?;
    println!("✓ Created table: {}", Product::table_name());

    // User table (custom naming)
    let user_migration = generate_migration!(User);
    manager.execute_migration(&user_migration).await?;
    println!("✓ Created table: {}", User::table_name());

    // BlogPost table (custom naming)
    let blog_migration = generate_migration!(BlogPost);
    manager.execute_migration(&blog_migration).await?;
    println!("✓ Created table: {}", BlogPost::table_name());

    // Customer table (legacy naming)
    let customer_migration = generate_migration!(Customer);
    manager.execute_migration(&customer_migration).await?;
    println!("✓ Created table: {}", Customer::table_name());

    // Create sample data
    println!("\n3. Creating Sample Data:");

    let db = manager.database();
    let now = Utc::now();

    // Create a product
    let product = Product {
        id: None,
        name: "Laptop Computer".to_string(),
        price: 999.99,
        created_at: now,
    };
    let created_product = product.create(db).await?;
    println!(
        "✓ Created product in '{}' table: {}",
        Product::table_name(),
        created_product.name
    );

    // Create a user
    let user = User {
        id: None,
        username: "johndoe".to_string(),
        email: "john.doe@example.com".to_string(),
        full_name: "John Doe".to_string(),
        is_active: true,
        created_at: now,
    };
    let created_user = user.create(db).await?;
    println!(
        "✓ Created user in '{}' table: {}",
        User::table_name(),
        created_user.username
    );

    // Create a blog post
    let blog_post = BlogPost {
        id: None,
        title: "Getting Started with libsql-orm".to_string(),
        slug: "getting-started-libsql-orm".to_string(),
        content: "This is a comprehensive guide to using libsql-orm...".to_string(),
        author_id: created_user.id.unwrap_or(1),
        published: true,
        published_at: Some(now),
        created_at: now,
        updated_at: now,
    };
    let created_post = blog_post.create(db).await?;
    println!(
        "✓ Created blog post in '{}' table: {}",
        BlogPost::table_name(),
        created_post.title
    );

    // Create a customer
    let customer = Customer {
        id: None,
        customer_code: "CUST001".to_string(),
        company_name: "Acme Corporation".to_string(),
        contact_email: "contact@acme.com".to_string(),
        status: "active".to_string(),
        created_at: now,
    };
    let created_customer = customer.create(db).await?;
    println!(
        "✓ Created customer in '{}' table: {}",
        Customer::table_name(),
        created_customer.company_name
    );

    // Query data from custom tables
    println!("\n4. Querying Data from Custom Tables:");

    let products = Product::find_all(db).await?;
    println!(
        "Found {} products in '{}' table",
        products.len(),
        Product::table_name()
    );

    let users = User::find_all(db).await?;
    println!(
        "Found {} users in '{}' table",
        users.len(),
        User::table_name()
    );

    let posts = BlogPost::find_all(db).await?;
    println!(
        "Found {} blog posts in '{}' table",
        posts.len(),
        BlogPost::table_name()
    );

    let customers = Customer::find_all(db).await?;
    println!(
        "Found {} customers in '{}' table",
        customers.len(),
        Customer::table_name()
    );

    // Demonstrate SQL generation with custom table names
    println!("\n5. Generated SQL with Custom Table Names:");
    println!("Product migration SQL:");
    println!("{}", Product::migration_sql());

    println!("\nUser migration SQL (custom table name):");
    println!("{}", User::migration_sql());

    println!("\nBlogPost migration SQL (custom table name):");
    println!("{}", BlogPost::migration_sql());

    // Best practices
    println!("\n6. Best Practices for Table Naming:");
    println!("✓ Use snake_case for table names");
    println!("✓ Use descriptive names that reflect the data");
    println!("✓ Consider pluralization for consistency (users, products, blog_posts)");
    println!("✓ Prefix with namespace for multi-tenant applications");
    println!("✓ Use the attribute for legacy database integration");

    println!("\n=== Table Name Macro Example Complete ===");
    println!("All models created with their respective custom table names!");

    Ok(())
}
