use chrono::{DateTime, Utc};
use libsql_orm::{Database, Filter, FilterOperator, Model};
use serde::{Deserialize, Serialize};
use std::env;

/// Basic CRUD Operations Example
///
/// This example demonstrates the fundamental Create, Read, Update, Delete operations
/// using libsql-orm with a simple User model.

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

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get database credentials from environment
    let database_url = env::var("LIBSQL_DATABASE_URL").expect("LIBSQL_DATABASE_URL must be set");
    let auth_token = env::var("LIBSQL_AUTH_TOKEN").expect("LIBSQL_AUTH_TOKEN must be set");

    let db = Database::new_connect(&database_url, &auth_token).await?;
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
    println!("Created user: {created_user:?}");

    // READ - Find the user by ID
    if let Some(user_id) = created_user.id {
        let found_user = User::find_by_id(user_id, &db).await?;
        println!("Found user by ID: {found_user:?}");

        // READ ALL - Find all users
        let all_users = User::find_all(&db).await?;
        println!("All users count: {}", all_users.len());

        // READ WITH FILTER - Find active users
        let active_users =
            User::find_where(FilterOperator::Single(Filter::eq("is_active", true)), &db).await?;
        println!("Active users: {}", active_users.len());

        // UPDATE - Modify the user
        let mut user_to_update = found_user.unwrap();
        user_to_update.name = "Jane Doe".to_string();
        user_to_update.age = Some(25);

        let updated_user = user_to_update.update(&db).await?;
        println!("Updated user: {updated_user:?}");

        // VERIFY UPDATE - Check if the update was applied
        let verified_user = User::find_by_id(user_id, &db).await?;
        println!("Verified updated user: {verified_user:?}");

        // DELETE - Remove the user
        let deleted = updated_user.delete(&db).await?;
        println!("Deleted {deleted} user(s)");

        // VERIFY DELETION - Try to find the deleted user
        let deleted_user = User::find_by_id(user_id, &db).await?;
        match deleted_user {
            Some(user) => println!("Warning: User still exists: {user:?}"),
            None => println!("Confirmed: User successfully deleted"),
        }
    }

    println!("✅ Basic CRUD operations completed successfully!");
    Ok(())
}
