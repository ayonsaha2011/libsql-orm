/// Basic CRUD Operations Example
/// 
/// This example demonstrates the fundamental Create, Read, Update, Delete operations
/// using libsql-orm with a simple User model in a Cloudflare Worker environment.

use worker::*;
use libsql_orm::{Model, Database, FilterOperator, Filter, Value};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[table_name("users")]
struct User {
    pub id: Option<i64>,
    pub name: String,
    pub email: String,
    pub age: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Get database credentials from environment
    let database_url = env.var("LIBSQL_DATABASE_URL")?.to_string();
    let auth_token = env.var("LIBSQL_AUTH_TOKEN")?.to_string();
    
    let db = Database::new_connect(&database_url, &auth_token).await
        .map_err(|e| worker::Error::RustError(format!("Database connection failed: {}", e)))?;
    
    println!("Connected to database successfully!");
    
    // CREATE - Insert a new user
    let new_user = User {
        id: None,
        name: "John Doe".to_string(),
        email: "john.doe@example.com".to_string(),
        age: Some(30),
        is_active: true,
        created_at: Utc::now(),
    };
    
    let created_user = new_user.create(&db).await?;
    println!("Created user: {:?}", created_user);
    
    // READ - Find the user by ID
    if let Some(user_id) = created_user.id {
        let found_user = User::find_by_id(user_id, &db).await?;
        println!("Found user by ID: {:?}", found_user);
        
        // READ - Find all users
        let all_users = User::find_all(&db).await?;
        println!("Total users in database: {}", all_users.len());
        
        // READ - Find users with filter
        let active_users = User::find_where(
            FilterOperator::Single(Filter::eq("is_active", true)),
            &db
        ).await?;
        println!("Active users: {}", active_users.len());
        
        // UPDATE - Modify the user
        let mut user_to_update = created_user.clone();
        user_to_update.name = "Jane Doe".to_string();
        user_to_update.age = Some(31);
        
        let updated_user = user_to_update.update(&db).await?;
        println!("Updated user: {:?}", updated_user);
        
        // Verify the update
        let verified_user = User::find_by_id(user_id, &db).await?;
        println!("Verified updated user: {:?}", verified_user);
        
        // DELETE - Remove the user
        let deleted = updated_user.delete(&db).await?;
        println!("User deleted successfully: {}", deleted);
        
        // Verify deletion
        let deleted_user = User::find_by_id(user_id, &db).await?;
        match deleted_user {
            Some(_) => println!("Error: User still exists after deletion"),
            None => println!("Confirmed: User successfully deleted"),
        }
    }
    
    Response::ok("✅ Basic CRUD operations completed successfully!")
}