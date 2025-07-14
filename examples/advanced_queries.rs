use chrono::{DateTime, Utc};
use libsql_orm::{Aggregate, Database, Filter, FilterOperator, Model, Pagination};
use serde::{Deserialize, Serialize};
use std::env;

/// Advanced Queries Example
///
/// This example demonstrates complex querying capabilities including
/// filtering, pagination, searching, aggregations, and bulk operations
/// in a standalone application.

#[derive(Model, Debug, Clone, Serialize, Deserialize)]
#[table_name("products")]
struct Product {
    pub id: Option<i64>,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub category: String,
    pub in_stock: bool,
    pub quantity: i32,
    pub created_at: DateTime<Utc>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to database using environment variables
    let database_url =
        env::var("LIBSQL_DATABASE_URL").unwrap_or_else(|_| "file:local.db".to_string());
    let auth_token = env::var("LIBSQL_AUTH_TOKEN").unwrap_or_else(|_| "".to_string());

    let db = Database::new_connect(&database_url, &auth_token).await?;

    println!("=== Advanced Queries Example ===");

    // Create sample data
    let sample_products = vec![
        Product {
            id: None,
            name: "Laptop".to_string(),
            description: "High-performance laptop for developers".to_string(),
            price: 1299.99,
            category: "Electronics".to_string(),
            in_stock: true,
            quantity: 50,
            created_at: Utc::now(),
        },
        Product {
            id: None,
            name: "Wireless Mouse".to_string(),
            description: "Ergonomic wireless mouse".to_string(),
            price: 29.99,
            category: "Electronics".to_string(),
            in_stock: true,
            quantity: 200,
            created_at: Utc::now(),
        },
        Product {
            id: None,
            name: "Coffee Mug".to_string(),
            description: "Ceramic coffee mug with logo".to_string(),
            price: 12.99,
            category: "Home".to_string(),
            in_stock: false,
            quantity: 0,
            created_at: Utc::now(),
        },
        Product {
            id: None,
            name: "Mechanical Keyboard".to_string(),
            description: "RGB mechanical keyboard".to_string(),
            price: 89.99,
            category: "Electronics".to_string(),
            in_stock: true,
            quantity: 75,
            created_at: Utc::now(),
        },
    ];

    // Bulk insert products
    println!("Creating sample products...");
    let created_products = Product::bulk_create(&sample_products, &db).await?;
    println!("Created {} products", created_products.len());

    // 1. Basic filtering
    println!("\n1. Basic Filtering:");
    let electronics = Product::find_where(
        FilterOperator::Single(Filter::eq("category", "Electronics")),
        &db,
    )
    .await?;
    println!("Electronics products: {}", electronics.len());

    // 2. Complex filtering with AND/OR
    println!("\n2. Complex Filtering:");
    let expensive_in_stock = Product::find_where(
        FilterOperator::And(vec![
            FilterOperator::Single(Filter::gt("price", 50.0)),
            FilterOperator::Single(Filter::eq("in_stock", true)),
        ]),
        &db,
    )
    .await?;
    println!("Expensive in-stock products: {}", expensive_in_stock.len());

    // 3. Range queries
    println!("\n3. Range Queries:");
    let mid_range_products = Product::find_where(
        FilterOperator::And(vec![
            FilterOperator::Single(Filter::ge("price", 20.0)),
            FilterOperator::Single(Filter::lt("price", 100.0)),
        ]),
        &db,
    )
    .await?;
    println!(
        "Mid-range products ($20-$100): {}",
        mid_range_products.len()
    );

    // 4. Pagination
    println!("\n4. Pagination:");
    let pagination = Pagination::new(1, 2); // Page 1, 2 items per page
    let paginated_result = Product::find_paginated(&pagination, &db).await?;
    println!(
        "Page {}/{}, Total: {}, Items on page: {}",
        paginated_result.pagination.page,
        paginated_result.pagination.total_pages.unwrap_or(0),
        paginated_result.pagination.total.unwrap_or(0),
        paginated_result.data.len()
    );

    // 5. Count operations
    println!("\n5. Count Operations:");

    // Count all products
    let total_count = Product::count(&db).await?;
    println!("Total products: {total_count}");

    // Count with filter
    let in_stock_count =
        Product::count_where(FilterOperator::Single(Filter::eq("in_stock", true)), &db).await?;
    println!("In-stock products: {in_stock_count}");

    // 6. Aggregations
    println!("\n6. Aggregations:");

    // Average price
    let avg_price = Product::aggregate(Aggregate::Avg, "price", None, &db).await?;
    println!("Average price: ${:.2}", avg_price.unwrap_or(0.0));

    // Max price in Electronics category
    let max_electronics_price = Product::aggregate(
        Aggregate::Max,
        "price",
        Some(FilterOperator::Single(Filter::eq(
            "category",
            "Electronics",
        ))),
        &db,
    )
    .await?;
    println!(
        "Max price in Electronics: ${:.2}",
        max_electronics_price.unwrap_or(0.0)
    );

    // Total quantity
    let total_quantity = Product::aggregate(Aggregate::Sum, "quantity", None, &db).await?;
    println!(
        "Total quantity in inventory: {}",
        total_quantity.unwrap_or(0.0) as i32
    );

    // 7. Bulk operations
    println!("\n7. Bulk Operations:");

    // Update multiple products
    let mut products_to_update = electronics.clone();
    for product in &mut products_to_update {
        product.price *= 0.9; // 10% discount
    }
    let updated_products = Product::bulk_update(&products_to_update, &db).await?;
    println!(
        "Updated {} electronics products with 10% discount",
        updated_products.len()
    );

    // Clean up - delete all test products
    let all_product_ids: Vec<i64> = created_products.iter().filter_map(|p| p.id).collect();

    let deleted_count = Product::bulk_delete(&all_product_ids, &db).await?;
    println!("\nCleaned up {deleted_count} test products");

    println!("\n=== Advanced Queries Example Complete ===");

    Ok(())
}
