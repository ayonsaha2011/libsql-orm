//! Pagination support for libsql-orm
//!
//! This module provides both offset-based and cursor-based pagination for handling
//! large datasets efficiently. Offset-based pagination is simpler but less efficient
//! for large datasets, while cursor-based pagination provides better performance
//! and consistency.
//!
//! # Offset-based Pagination
//!
//! ```rust
//! use libsql_orm::{Pagination, PaginatedResult, Model};
//!
//! async fn paginate_users(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
//!     let pagination = Pagination::new(1, 10); // Page 1, 10 items per page
//!     let result: PaginatedResult<User> = User::find_paginated(&pagination, db).await?;
//!     
//!     println!("Page {}/{}", result.pagination.page, result.pagination.total_pages.unwrap_or(0));
//!     println!("Total items: {}", result.pagination.total.unwrap_or(0));
//!     
//!     for user in result.data {
//!         println!("User: {}", user.name);
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Cursor-based Pagination
//!
//! ```rust
//! use libsql_orm::{CursorPagination, CursorPaginatedResult};
//!
//! async fn cursor_paginate(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
//!     let mut cursor_pagination = CursorPagination::new(10);
//!     
//!     loop {
//!         // Implement cursor-based pagination logic here
//!         // This would typically use a timestamp or ID as the cursor
//!         break;
//!     }
//!     
//!     Ok(())
//! }
//! ```

use serde::{Deserialize, Serialize};

/// Pagination parameters for queries
///
/// Provides offset-based pagination with helpful utility methods for calculating
/// offsets, page numbers, and navigation state.
///
/// # Examples
///
/// ```rust
/// use libsql_orm::Pagination;
///
/// let mut pagination = Pagination::new(2, 10); // Page 2, 10 items per page
/// pagination.set_total(45); // 45 total items
///
/// assert_eq!(pagination.offset(), 10); // Skip first 10 items
/// assert_eq!(pagination.limit(), 10);  // Take 10 items
/// assert_eq!(pagination.total_pages, Some(5)); // 5 total pages
/// assert!(pagination.has_prev()); // Has previous page
/// assert!(pagination.has_next()); // Has next page
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// Page number (1-based)
    pub page: u32,
    /// Number of items per page
    pub per_page: u32,
    /// Total number of items (set after query execution)
    pub total: Option<u64>,
    /// Total number of pages (calculated)
    pub total_pages: Option<u32>,
}

impl Pagination {
    /// Create a new pagination instance
    pub fn new(page: u32, per_page: u32) -> Self {
        Self {
            page,
            per_page,
            total: None,
            total_pages: None,
        }
    }

    /// Get the offset for SQL LIMIT/OFFSET
    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.per_page
    }

    /// Get the limit for SQL LIMIT/OFFSET
    pub fn limit(&self) -> u32 {
        self.per_page
    }

    /// Set the total count and calculate total pages
    pub fn set_total(&mut self, total: u64) {
        self.total = Some(total);
        self.total_pages = Some(((total as f64) / (self.per_page as f64)).ceil() as u32);
    }

    /// Check if there's a next page
    pub fn has_next(&self) -> bool {
        if let (Some(total_pages), Some(current_page)) = (self.total_pages, Some(self.page)) {
            current_page < total_pages
        } else {
            false
        }
    }

    /// Check if there's a previous page
    pub fn has_prev(&self) -> bool {
        self.page > 1
    }

    /// Get the start item number for the current page
    pub fn start_item(&self) -> u32 {
        (self.page - 1) * self.per_page + 1
    }

    /// Get the end item number for the current page
    pub fn end_item(&self) -> u32 {
        self.page * self.per_page
    }

    /// Get the next page number
    pub fn next_page(&self) -> Option<u32> {
        if self.has_next() {
            Some(self.page + 1)
        } else {
            None
        }
    }

    /// Get the previous page number
    pub fn prev_page(&self) -> Option<u32> {
        if self.has_prev() {
            Some(self.page - 1)
        } else {
            None
        }
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self::new(1, 20)
    }
}

/// Paginated result containing data and pagination metadata
///
/// Contains both the data items for the current page and pagination metadata
/// including current page, total pages, and navigation information.
///
/// # Examples
///
/// ```rust
/// use libsql_orm::{PaginatedResult, Pagination};
///
/// let pagination = Pagination::new(1, 10);
/// let data = vec!["item1".to_string(), "item2".to_string()];
/// let result = PaginatedResult::with_total(data, pagination, 25);
///
/// println!("Items on page: {}", result.len());
/// println!("Total items: {}", result.pagination.total.unwrap_or(0));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    /// The data items for the current page
    pub data: Vec<T>,
    /// Pagination metadata
    pub pagination: Pagination,
}

impl<T> PaginatedResult<T> {
    /// Create a new paginated result
    pub fn new(data: Vec<T>, pagination: Pagination) -> Self {
        Self { data, pagination }
    }

    /// Create a paginated result with total count
    pub fn with_total(data: Vec<T>, mut pagination: Pagination, total: u64) -> Self {
        pagination.set_total(total);
        Self { data, pagination }
    }

    /// Get the data items
    pub fn data(&self) -> &[T] {
        &self.data
    }

    /// Get the pagination metadata
    pub fn pagination(&self) -> &Pagination {
        &self.pagination
    }

    /// Get the number of items in the current page
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the current page is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Map the data items to a new type
    pub fn map<U, F>(self, f: F) -> PaginatedResult<U>
    where
        F: FnMut(T) -> U,
    {
        PaginatedResult {
            data: self.data.into_iter().map(f).collect(),
            pagination: self.pagination,
        }
    }
}

/// Cursor-based pagination for better performance with large datasets
///
/// Cursor-based pagination uses a cursor (typically a timestamp or ID) to mark
/// the position in the dataset, providing consistent results even when data is
/// being added or removed during pagination.
///
/// # Advantages
///
/// - **Consistent results**: No duplicate or missing items when data changes
/// - **Better performance**: No need to count total items or skip large offsets  
/// - **Real-time friendly**: Works well with live data streams
///
/// # Examples
///
/// ```rust
/// use libsql_orm::CursorPagination;
///
/// // First page
/// let pagination = CursorPagination::new(10);
///
/// // Subsequent pages using cursor from previous result
/// let next_pagination = CursorPagination::with_cursor(10, Some("cursor_value".to_string()));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPagination {
    /// Cursor for the next page
    pub cursor: Option<String>,
    /// Number of items per page
    pub limit: u32,
    /// Whether to include the cursor item in results
    pub include_cursor: bool,
    /// Whether there are more items
    pub has_next: bool,
    /// Whether there are previous items
    pub has_prev: bool,
    /// Cursor for the next page
    pub next_cursor: Option<String>,
    /// Cursor for the previous page
    pub prev_cursor: Option<String>,
    /// Total number of items
    pub total: Option<u64>,
}

impl CursorPagination {
    /// Create a new cursor pagination instance
    pub fn new(limit: u32) -> Self {
        Self {
            cursor: None,
            limit,
            include_cursor: false,
            has_next: false,
            has_prev: false,
            next_cursor: None,
            prev_cursor: None,
            total: None,
        }
    }

    /// Create with a specific cursor
    pub fn with_cursor(limit: u32, cursor: Option<String>) -> Self {
        let has_prev = cursor.is_some();
        Self {
            cursor,
            limit,
            include_cursor: false,
            has_next: false,
            has_prev,
            next_cursor: None,
            prev_cursor: None,
            total: None,
        }
    }

    /// Create with a specific cursor (deprecated, use with_cursor(limit, cursor) instead)
    pub fn with_cursor_old(cursor: String, limit: u32) -> Self {
        Self {
            cursor: Some(cursor),
            limit,
            include_cursor: false,
            has_next: false,
            has_prev: true,
            next_cursor: None,
            prev_cursor: None,
            total: None,
        }
    }

    /// Set the cursor
    pub fn set_cursor(&mut self, cursor: Option<String>) {
        self.cursor = cursor;
    }

    /// Get the limit for SQL queries
    pub fn limit(&self) -> u32 {
        self.limit
    }
}

impl Default for CursorPagination {
    fn default() -> Self {
        Self::new(20)
    }
}

/// Cursor-based paginated result
///
/// Contains data items and cursor pagination metadata for navigating
/// through large datasets efficiently.
///
/// # Examples
///
/// ```rust
/// use libsql_orm::{CursorPaginatedResult, CursorPagination};
///
/// let pagination = CursorPagination::new(10);
/// let data = vec!["item1".to_string(), "item2".to_string()];
/// let result = CursorPaginatedResult::new(data, pagination);
///
/// println!("Items: {}", result.data().len());
/// println!("Has next: {}", result.pagination().has_next);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPaginatedResult<T> {
    /// The data items
    pub data: Vec<T>,
    /// Pagination metadata
    pub pagination: CursorPagination,
}

impl<T> CursorPaginatedResult<T> {
    /// Create a new cursor paginated result
    pub fn new(data: Vec<T>, pagination: CursorPagination) -> Self {
        Self { data, pagination }
    }

    /// Get the data items
    pub fn data(&self) -> &[T] {
        &self.data
    }

    /// Get the pagination metadata
    pub fn pagination(&self) -> &CursorPagination {
        &self.pagination
    }
}
