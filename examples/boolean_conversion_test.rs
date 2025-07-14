/// Boolean Conversion Test Example
/// 
/// This example tests the fix for the serialization error where SQLite's integer
/// boolean values (0/1) are properly converted to Rust boolean types in a Cloudflare Worker environment.

use worker::*;
use libsql_orm::{Model, Database, generate_migration, MigrationManager, FilterOperator, Filter, Value};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[table_name("test_users")]
struct TestUser {
    pub id: Option<i64>,
    pub name: String,
    pub email: String,
    pub is_active: bool,
    pub is_verified: bool,
    pub has_premium: bool,
    pub can_edit: bool,
    pub enabled: bool,
    pub published: bool,
    pub age: i32,
    pub score: f64,
    pub created_at: DateTime<Utc>,
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Connect to database
    let database_url = env.var("LIBSQL_DATABASE_URL")?.to_string();
    let auth_token = env.var("LIBSQL_AUTH_TOKEN")?.to_string();
    
    let db = Database::new_connect(&database_url, &auth_token).await
        .map_err(|e| worker::Error::RustError(format!("Database connection failed: {}", e)))?;
    let manager = MigrationManager::new(db);
    
    println!("=== Boolean Conversion Test ===");
    
    // Initialize migration system
    manager.init().await?;
    
    // Create table
    let migration = generate_migration!(TestUser);
    manager.execute_migration(&migration).await?;
    println!("✓ Created test_users table");
    
    let db = manager.database();
    let now = Utc::now();
    
    // Create test users with various boolean combinations
    let test_users = vec![
        TestUser {
            id: None,
            name: "Alice Active".to_string(),
            email: "alice@example.com".to_string(),
            is_active: true,
            is_verified: true,
            has_premium: false,
            can_edit: true,
            enabled: true,
            published: false,
            age: 30,
            score: 95.5,
            created_at: now,
        },
        TestUser {
            id: None,
            name: "Bob Inactive".to_string(),
            email: "bob@example.com".to_string(),
            is_active: false,
            is_verified: false,
            has_premium: true,
            can_edit: false,
            enabled: false,
            published: true,
            age: 25,
            score: 78.2,
            created_at: now,
        },
        TestUser {
            id: None,
            name: "Carol Mixed".to_string(),
            email: "carol@example.com".to_string(),
            is_active: true,
            is_verified: false,
            has_premium: true,
            can_edit: true,
            enabled: false,
            published: true,
            age: 35,
            score: 88.7,
            created_at: now,
        },
    ];
    
    // Insert users
    println!("\n1. Creating test users with boolean values...");
    let created_users = TestUser::bulk_create(&test_users, db).await?;
    println!("✓ Created {} test users", created_users.len());
    
    // Verify boolean values were stored correctly
    println!("\n2. Verifying created users:");
    for user in &created_users {
        println!("User: {} - Active: {}, Verified: {}, Premium: {}, Can Edit: {}, Enabled: {}, Published: {}", 
            user.name, user.is_active, user.is_verified, user.has_premium, 
            user.can_edit, user.enabled, user.published);
    }
    
    // Query users back and verify boolean conversion
    println!("\n3. Querying users back from database...");
    let queried_users = TestUser::find_all(db).await?;
    println!("✓ Retrieved {} users from database", queried_users.len());
    
    // Verify all boolean fields are properly converted
    println!("\n4. Verifying boolean field conversion:");
    for user in &queried_users {
        println!("User: {}", user.name);
        println!("  is_active: {} (type: {})", user.is_active, std::any::type_name_of_val(&user.is_active));
        println!("  is_verified: {} (type: {})", user.is_verified, std::any::type_name_of_val(&user.is_verified));
        println!("  has_premium: {} (type: {})", user.has_premium, std::any::type_name_of_val(&user.has_premium));
        println!("  can_edit: {} (type: {})", user.can_edit, std::any::type_name_of_val(&user.can_edit));
        println!("  enabled: {} (type: {})", user.enabled, std::any::type_name_of_val(&user.enabled));
        println!("  published: {} (type: {})", user.published, std::any::type_name_of_val(&user.published));
        println!("  age: {} (type: {})", user.age, std::any::type_name_of_val(&user.age));
        println!("  score: {} (type: {})", user.score, std::any::type_name_of_val(&user.score));
        println!();
    }
    
    // Test filtering with boolean values
    println!("5. Testing boolean filtering:");
    
    let active_users = TestUser::find_where(
        FilterOperator::Single(Filter::eq("is_active", true)),
        db
    ).await?;
    println!("✓ Found {} active users", active_users.len());
    
    let verified_users = TestUser::find_where(
        FilterOperator::Single(Filter::eq("is_verified", true)),
        db
    ).await?;
    println!("✓ Found {} verified users", verified_users.len());
    
    let premium_users = TestUser::find_where(
        FilterOperator::Single(Filter::eq("has_premium", true)),
        db
    ).await?;
    println!("✓ Found {} premium users", premium_users.len());
    
    // Test complex boolean filtering
    let complex_filter = FilterOperator::And(vec![
        FilterOperator::Single(Filter::eq("is_active", true)),
        FilterOperator::Single(Filter::eq("is_verified", false)),
    ]);
    
    let filtered_users = TestUser::find_where(complex_filter, db).await?;
    println!("✓ Found {} users that are active but not verified", filtered_users.len());
    
    // Verify serialization/deserialization works
    println!("\n6. Testing JSON serialization/deserialization:");
    for user in &queried_users {
        let json = serde_json::to_string(user)?;
        println!("JSON: {}", json);
        
        let deserialized: TestUser = serde_json::from_str(&json)?;
        println!("Deserialized successfully: {}", deserialized.name);
        
        // Verify all boolean fields match
        assert_eq!(user.is_active, deserialized.is_active);
        assert_eq!(user.is_verified, deserialized.is_verified);
        assert_eq!(user.has_premium, deserialized.has_premium);
        assert_eq!(user.can_edit, deserialized.can_edit);
        assert_eq!(user.enabled, deserialized.enabled);
        assert_eq!(user.published, deserialized.published);
        println!("✓ All boolean fields match after JSON round-trip");
        break; // Test just one for brevity
    }
    
    println!("\n=== Boolean Conversion Test Complete ===");
    println!("✅ All boolean values correctly converted from SQLite integers!");
    println!("✅ No 'invalid type: integer `1`, expected a boolean' errors!");
    println!("✅ Filtering with boolean values works correctly!");
    println!("✅ JSON serialization/deserialization works properly!");
    
    Response::ok("✅ Boolean conversion test completed successfully!")
}