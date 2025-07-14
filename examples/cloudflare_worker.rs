use chrono::{DateTime, Utc};
use libsql_orm::{Database, Filter, FilterOperator, Model, Pagination};
use serde::{Deserialize, Serialize};
/// Cloudflare Worker Integration Example
///
/// This example shows how to use libsql-orm in a Cloudflare Worker environment
/// with proper error handling and HTTP responses.
use worker::*;

#[derive(Model, Debug, Clone, Serialize, Deserialize)]
struct Post {
    pub id: Option<i64>,
    pub title: String,
    pub content: String,
    pub author: String,
    pub published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
struct CreatePostRequest {
    title: String,
    content: String,
    author: String,
}

#[derive(Serialize, Deserialize)]
struct UpdatePostRequest {
    title: Option<String>,
    content: Option<String>,
    published: Option<bool>,
}

#[event(fetch)]
async fn fetch(mut req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // console_error_panic_hook::set_once(); // Uncomment if you add console-error-panic-hook to your dependencies

    // Initialize database connection
    let db = match init_database(&env).await {
        Ok(db) => db,
        Err(e) => return Response::error(format!("Database initialization failed: {e}"), 500),
    };

    // Route the request
    match (req.method(), req.path().as_str()) {
        // GET /posts - List all posts with optional pagination
        (Method::Get, "/posts") => {
            let url = req.url()?;
            let page: usize = url
                .query_pairs()
                .find(|(key, _)| key == "page")
                .and_then(|(_, value)| value.parse().ok())
                .unwrap_or(1);
            let per_page: usize = url
                .query_pairs()
                .find(|(key, _)| key == "per_page")
                .and_then(|(_, value)| value.parse().ok())
                .unwrap_or(10);

            let pagination = Pagination::new(page as u32, per_page as u32);

            match Post::find_paginated(&pagination, &db).await {
                Ok(result) => Response::from_json(&result),
                Err(e) => Response::error(format!("Failed to fetch posts: {e}"), 500),
            }
        }

        // GET /posts/{id} - Get specific post
        (Method::Get, path) if path.starts_with("/posts/") => {
            let id_str = path.strip_prefix("/posts/").unwrap();
            let id: i64 = match id_str.parse() {
                Ok(id) => id,
                Err(_) => return Response::error("Invalid post ID", 400),
            };

            match Post::find_by_id(id, &db).await {
                Ok(Some(post)) => Response::from_json(&post),
                Ok(None) => Response::error("Post not found", 404),
                Err(e) => Response::error(format!("Database error: {e}"), 500),
            }
        }

        // POST /posts - Create new post
        (Method::Post, "/posts") => {
            let create_req: CreatePostRequest = match req.json().await {
                Ok(req) => req,
                Err(_) => return Response::error("Invalid JSON", 400),
            };

            let now = Utc::now();
            let new_post = Post {
                id: None,
                title: create_req.title,
                content: create_req.content,
                author: create_req.author,
                published: false,
                created_at: now,
                updated_at: now,
            };

            match new_post.create(&db).await {
                Ok(created_post) => Response::from_json(&created_post),
                Err(e) => Response::error(format!("Failed to create post: {e}"), 500),
            }
        }

        // PUT /posts/{id} - Update existing post
        (Method::Put, path) if path.starts_with("/posts/") => {
            let id_str = path.strip_prefix("/posts/").unwrap();
            let id: i64 = match id_str.parse() {
                Ok(id) => id,
                Err(_) => return Response::error("Invalid post ID", 400),
            };

            let update_req: UpdatePostRequest = match req.json().await {
                Ok(req) => req,
                Err(_) => return Response::error("Invalid JSON", 400),
            };

            // Find existing post
            let mut post = match Post::find_by_id(id, &db).await {
                Ok(Some(post)) => post,
                Ok(None) => return Response::error("Post not found", 404),
                Err(e) => return Response::error(format!("Database error: {e}"), 500),
            };

            // Update fields if provided
            if let Some(title) = update_req.title {
                post.title = title;
            }
            if let Some(content) = update_req.content {
                post.content = content;
            }
            if let Some(published) = update_req.published {
                post.published = published;
            }
            post.updated_at = Utc::now();

            match post.update(&db).await {
                Ok(updated_post) => Response::from_json(&updated_post),
                Err(e) => Response::error(format!("Failed to update post: {e}"), 500),
            }
        }

        // DELETE /posts/{id} - Delete post
        (Method::Delete, path) if path.starts_with("/posts/") => {
            let id_str = path.strip_prefix("/posts/").unwrap();
            let id: i64 = match id_str.parse() {
                Ok(id) => id,
                Err(_) => return Response::error("Invalid post ID", 400),
            };

            // Find and delete post
            match Post::find_by_id(id, &db).await {
                Ok(Some(post)) => match post.delete(&db).await {
                    Ok(_) => Response::ok("Post deleted successfully"),
                    Err(e) => Response::error(format!("Failed to delete post: {e}"), 500),
                },
                Ok(None) => Response::error("Post not found", 404),
                Err(e) => Response::error(format!("Database error: {e}"), 500),
            }
        }

        // GET /posts/published - Get only published posts
        (Method::Get, "/posts/published") => {
            match Post::find_where(FilterOperator::Single(Filter::eq("published", true)), &db).await
            {
                Ok(posts) => Response::from_json(&posts),
                Err(e) => Response::error(format!("Failed to fetch published posts: {e}"), 500),
            }
        }

        // GET /posts/author/{author} - Get posts by author
        (Method::Get, path) if path.starts_with("/posts/author/") => {
            let author = path.strip_prefix("/posts/author/").unwrap();

            match Post::find_where(
                FilterOperator::Single(Filter::eq("author", author.to_string())),
                &db,
            )
            .await
            {
                Ok(posts) => Response::from_json(&posts),
                Err(e) => Response::error(format!("Failed to fetch posts by author: {e}"), 500),
            }
        }

        // Health check
        (Method::Get, "/health") => Response::ok("libsql-orm Cloudflare Worker is healthy!"),

        // 404 for unmatched routes
        _ => Response::error("Not Found", 404),
    }
}

async fn init_database(env: &Env) -> Result<Database> {
    // Get database credentials from environment variables
    let database_url = env.var("LIBSQL_DATABASE_URL")?.to_string();
    let auth_token = env.var("LIBSQL_AUTH_TOKEN")?.to_string();

    // Connect to database
    let db = Database::new_connect(&database_url, &auth_token)
        .await
        .map_err(|e| format!("Database connection failed: {e}"))?;

    // For now, skip migrations in the example to avoid ownership issues
    // In a real app, you'd handle this differently
    Ok(db)
}

// This is required for Rust examples but not used in Cloudflare Workers
fn main() {
    // This example is designed to run as a Cloudflare Worker.
    // To deploy it, use the Wrangler CLI tool:
    // 1. Install wrangler: npm install -g wrangler
    // 2. Configure wrangler.toml with your database credentials
    // 3. Deploy: wrangler publish
    println!("This is a Cloudflare Worker example. Please use wrangler to deploy it.");
}
