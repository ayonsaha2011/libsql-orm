# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of libsql-orm
- Core ORM functionality with Model trait
- Database connection wrapper for libsql
- Query builder with fluent API
- Comprehensive filtering and search capabilities
- Pagination support (offset-based and cursor-based)
- Migration system with auto-generation
- Bulk operations (create, update, delete)
- Aggregation functions (COUNT, SUM, AVG, MIN, MAX)
- Full Cloudflare Workers compatibility
- WebAssembly (WASM) optimization
- Derive macros for automatic model generation
- Type-safe database operations
- Async/await support throughout
- Comprehensive error handling
- **Custom Table Names**: `#[table_name("custom")]` attribute support
- **Boolean Type Safety**: Automatic SQLite integer ↔ Rust boolean conversion
- **Column Attributes**: `#[orm_column(...)]` for column customization
- Full API documentation
- Examples and usage guides

### Features
- **Model Derive Macro**: Automatic implementation of ORM traits
- **Query Builder**: Fluent API for complex SQL generation
- **Pagination**: Both offset and cursor-based pagination
- **Search & Filtering**: Advanced text search and filtering
- **Bulk Operations**: Efficient batch processing
- **Aggregations**: Statistical functions and grouping
- **Type Safety**: Compile-time guarantees with Rust's type system
- **WASM Ready**: Optimized for edge computing environments
- **Cloudflare Workers**: First-class integration support
- **Custom Table Names**: Override default naming with `#[table_name("custom")]`
- **Boolean Type Conversion**: Seamless SQLite integer (0/1) to Rust bool conversion
- **Column Customization**: Fine-grained control with `#[orm_column(...)]` attributes

### Technical Details
- Built on libsql v0.9.14 with Cloudflare features
- Compatible with wasm32-unknown-unknown target
- Async-first design with tokio compatibility
- Zero-copy deserialization where possible
- Minimal memory footprint for edge deployments

## [0.1.0] - 2024-07-14

### Added
- Initial public release
- Complete ORM implementation for libsql
- Cloudflare Workers integration
- Comprehensive documentation and examples
- MIT license
- Ready for crates.io publication

---

For more details about upcoming features and known issues, please check our [GitHub repository](https://github.com/ayonsaha2011/libsql-orm).