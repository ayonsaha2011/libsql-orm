//! Procedural macros for libsql-orm
//!
//! This crate provides derive macros and attribute macros for the libsql-orm library,
//! enabling automatic implementation of ORM traits and convenient model definitions.
//!
//! # Derive Macros
//!
//! ## `#[derive(Model)]`
//!
//! Automatically implements the `Model` trait for a struct, providing all CRUD operations
//! and ORM functionality.
//!
//! ```rust
//! use libsql_orm::Model;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Model, Serialize, Deserialize)]
//! struct User {
//!     pub id: Option<i64>,
//!     pub name: String,
//!     pub email: String,
//! }
//! ```
//!
//! # Attribute Macros
//!
//! ## `#[table_name("custom_name")]`
//!
//! Specifies a custom table name for the model. By default, the table name is derived
//! from the struct name converted to lowercase.
//!
//! ```rust
//! use libsql_orm::Model;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Model, Serialize, Deserialize)]
//! #[table_name("custom_users")]
//! struct User {
//!     pub id: Option<i64>,
//!     pub name: String,
//! }
//! ```
//!
//! ## `#[orm_column(...)]`
//!
//! Specifies custom column properties for database fields.
//!
//! ```rust
//! use libsql_orm::Model;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Model, Serialize, Deserialize)]
//! struct User {
//!     #[orm_column(type = "INTEGER PRIMARY KEY AUTOINCREMENT")]
//!     pub id: Option<i64>,
//!     
//!     #[orm_column(not_null, unique)]
//!     pub email: String,
//!     
//!     #[orm_column(type = "TEXT DEFAULT 'active'")]
//!     pub status: String,
//! }
//! ```
//!
//! # Function-like Macros
//!
//! ## `generate_migration!(Model)`
//!
//! Generates a database migration from a model definition.
//!
//! ```rust
//! use libsql_orm::{generate_migration, MigrationManager};
//!
//! let migration = generate_migration!(User);
//! let manager = MigrationManager::new(db);
//! manager.execute_migration(&migration).await?;
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Field, Fields, Lit, Type};

/// Column attribute macro for defining SQL column properties
///
/// This attribute allows you to specify custom SQL column properties for struct fields.
///
/// # Supported attributes:
/// - `type = "SQL_TYPE"` - Custom SQL type definition
/// - `not_null` - Add NOT NULL constraint
/// - `unique` - Add UNIQUE constraint  
/// - `primary_key` - Mark as PRIMARY KEY
/// - `auto_increment` - Add AUTOINCREMENT (for INTEGER PRIMARY KEY)
///
/// # Examples:
///
/// ```rust
/// #[derive(Model)]
/// struct User {
///     #[orm_column(type = "INTEGER PRIMARY KEY AUTOINCREMENT")]
///     pub id: Option<i64>,
///     
///     #[orm_column(not_null, unique)]
///     pub email: String,
///     
///     #[orm_column(type = "TEXT DEFAULT 'active'")]
///     pub status: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn orm_column(_args: TokenStream, input: TokenStream) -> TokenStream {
    // For now, just return the input unchanged
    // We'll parse the column attributes in the Model macro
    input
}

/// Derive macro for the Model trait
///
/// Automatically implements the `Model` trait for a struct, providing CRUD operations
/// and ORM functionality. The macro analyzes the struct fields to generate appropriate
/// SQL schema and conversion methods.
///
/// # Attributes:
/// - `#[table_name("custom_name")]` - Specify custom table name
/// - `#[orm_column(...)]` - Configure column properties
///
/// # Examples:
///
/// ```rust
/// use libsql_orm::Model;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Model, Serialize, Deserialize)]
/// #[table_name("users")]
/// struct User {
///     pub id: Option<i64>,
///     pub name: String,
///     pub email: String,
/// }
/// ```
#[proc_macro_derive(Model, attributes(table_name, orm_column))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Extract table name from attributes or use default
    let table_name =
        extract_table_name(&input.attrs).unwrap_or_else(|| name.to_string().to_lowercase());

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Extract field names and column metadata for columns
    let (field_names, column_definitions, boolean_field_names, boolean_flags) =
        if let Data::Struct(data) = &input.data {
            if let Fields::Named(fields) = &data.fields {
                let mut field_names = Vec::new();
                let mut column_defs = Vec::new();
                let mut bool_field_names = Vec::new();
                let mut bool_flags = Vec::new();

                for field in &fields.named {
                    let field_name = &field.ident;
                    let field_name_str = quote! { stringify!(#field_name) };
                    field_names.push(field_name_str);

                    // Parse column attributes to get SQL definition
                    let column_def = parse_column_definition(field);
                    column_defs.push(column_def);

                    // Extract field type information for conversion
                    let field_type = &field.ty;
                    let is_bool = is_boolean_type(field_type);
                    bool_field_names.push(quote! { stringify!(#field_name) });
                    bool_flags.push(is_bool);
                }

                (field_names, column_defs, bool_field_names, bool_flags)
            } else {
                (vec![], vec![], vec![], vec![])
            }
        } else {
            (vec![], vec![], vec![], vec![])
        };

    let expanded = quote! {
        impl #impl_generics libsql_orm::Model for #name #ty_generics #where_clause {
            fn table_name() -> &'static str {
                #table_name
            }

            fn get_primary_key(&self) -> Option<i64> {
                self.id
            }

            fn set_primary_key(&mut self, id: i64) {
                self.id = Some(id);
            }

            fn columns() -> Vec<&'static str> {
                vec![#(#field_names),*]
            }

            /// Generate SQL for creating the table
            fn migration_sql() -> String {
                let columns = vec![#(#column_definitions),*];
                format!(
                    "CREATE TABLE IF NOT EXISTS {} (\n    {}\n)",
                    Self::table_name(),
                    columns.join(",\n    ")
                )
            }

            fn to_map(&self) -> libsql_orm::Result<std::collections::HashMap<String, libsql_orm::Value>> {
                use serde_json;
                let json = serde_json::to_value(self)?;
                let map: std::collections::HashMap<String, serde_json::Value> = serde_json::from_value(json)?;

                let mut result = std::collections::HashMap::new();
                for (k, v) in map {
                    let value = match v {
                        serde_json::Value::Null => libsql_orm::Value::Null,
                        serde_json::Value::Bool(b) => libsql_orm::Value::Boolean(b),
                        serde_json::Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                libsql_orm::Value::Integer(i)
                            } else if let Some(f) = n.as_f64() {
                                libsql_orm::Value::Real(f)
                            } else {
                                libsql_orm::Value::Text(n.to_string())
                            }
                        }
                        serde_json::Value::String(s) => libsql_orm::Value::Text(s),
                        serde_json::Value::Array(_) => libsql_orm::Value::Text(serde_json::to_string(&v)?),
                        serde_json::Value::Object(_) => libsql_orm::Value::Text(serde_json::to_string(&v)?),
                    };
                    result.insert(k, value);
                }
                Ok(result)
            }

            fn from_map(map: std::collections::HashMap<String, libsql_orm::Value>) -> libsql_orm::Result<Self> {
                use serde_json;
                let mut json_map = serde_json::Map::new();

                for (k, v) in map {
                    let json_value = match v {
                        libsql_orm::Value::Null => serde_json::Value::Null,
                        libsql_orm::Value::Boolean(b) => serde_json::Value::Bool(b),
                        libsql_orm::Value::Integer(i) => {
                            // Convert integers to booleans for known boolean fields
                            let field_name = k.as_str();
                            let mut is_boolean_field = false;
                            #(
                                if field_name == #boolean_field_names {
                                    is_boolean_field = #boolean_flags;
                                }
                            )*

                            if is_boolean_field {
                                serde_json::Value::Bool(i != 0)
                            } else {
                                serde_json::Value::Number(serde_json::Number::from(i))
                            }
                        }
                        libsql_orm::Value::Real(f) => {
                            if let Some(n) = serde_json::Number::from_f64(f) {
                                serde_json::Value::Number(n)
                            } else {
                                serde_json::Value::String(f.to_string())
                            }
                        }
                        libsql_orm::Value::Text(s) => serde_json::Value::String(s),
                        libsql_orm::Value::Blob(b) => {
                            serde_json::Value::Array(b.into_iter().map(|byte| serde_json::Value::Number(serde_json::Number::from(byte))).collect())
                        }
                    };
                    json_map.insert(k, json_value);
                }

                let json_value = serde_json::Value::Object(json_map);
                let result: Self = serde_json::from_value(json_value)?;
                Ok(result)
            }
        }

        // Note: Clone is already derived in the struct definition
    };

    TokenStream::from(expanded)
}

/// Parse column definition from field attributes
fn parse_column_definition(field: &Field) -> proc_macro2::TokenStream {
    let field_name = &field.ident;
    let field_name_str = field_name.as_ref().unwrap().to_string();

    // Default column definitions based on field type
    let default_def = match &field.ty {
        Type::Path(type_path) => {
            let type_name = &type_path.path.segments.last().unwrap().ident;
            match type_name.to_string().as_str() {
                "i64" => format!("{field_name_str} INTEGER"),
                "i32" => format!("{field_name_str} INTEGER"),
                "f64" => format!("{field_name_str} REAL"),
                "f32" => format!("{field_name_str} REAL"),
                "bool" => format!("{field_name_str} BOOLEAN"),
                "String" => format!("{field_name_str} TEXT"),
                _ => format!("{field_name_str} TEXT"),
            }
        }
        _ => format!("{field_name_str} TEXT"),
    };

    // Check for orm_column attributes
    for attr in &field.attrs {
        if attr.path().is_ident("orm_column") {
            let mut column_type = None;
            let mut not_null = false;
            let mut unique = false;
            let mut primary_key = false;
            let mut auto_increment = false;

            // Parse the nested meta items
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("type") {
                    if let Ok(value) = meta.value() {
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(lit_str) = lit {
                            column_type = Some(lit_str.value());
                        }
                    }
                } else if meta.path.is_ident("not_null") {
                    not_null = true;
                } else if meta.path.is_ident("unique") {
                    unique = true;
                } else if meta.path.is_ident("primary_key") {
                    primary_key = true;
                } else if meta.path.is_ident("auto_increment") {
                    auto_increment = true;
                }
                Ok(())
            });

            let mut column_def = column_type.unwrap_or_else(|| default_def.clone());
            if primary_key {
                column_def = format!("{column_def} PRIMARY KEY");
            }
            if auto_increment {
                column_def = format!("{column_def} AUTOINCREMENT");
            }
            if not_null {
                column_def = format!("{column_def} NOT NULL");
            }
            if unique {
                column_def = format!("{column_def} UNIQUE");
            }
            return quote! { #column_def };
        }
    }
    // Return default definition
    quote! { #default_def }
}

/// Extract table name from struct attributes
fn extract_table_name(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("table_name") {
            if let Ok(Lit::Str(lit_str)) = attr.parse_args::<Lit>() {
                return Some(lit_str.value());
            }
        }
    }
    None
}

/// Check if a type is a boolean type
fn is_boolean_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let type_name = &segment.ident;
            return type_name == "bool";
        }
    }
    false
}

/// Macro to generate migration from a model
///
/// Creates a migration instance from a model's schema definition. The migration
/// will contain the SQL necessary to create the table for the model.
///
/// # Examples:
///
/// ```rust
/// use libsql_orm::{generate_migration, MigrationManager};
///
/// // Generate migration for User model
/// let user_migration = generate_migration!(User);
///
/// // Execute the migration
/// let manager = MigrationManager::new(db);
/// manager.execute_migration(&user_migration).await?;
/// ```
#[proc_macro]
pub fn generate_migration(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::Ident);

    let expanded = quote! {
        {
            let sql = #input::migration_sql();
            libsql_orm::MigrationManager::create_migration(
                &format!("create_table_{}", #input::table_name()),
                &sql
            )
        }
    };

    TokenStream::from(expanded)
}
