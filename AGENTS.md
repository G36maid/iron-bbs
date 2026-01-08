# Iron BBS - Agent Development Guide

## Build Commands

### Development
```bash
# Run application (starts both web and SSH servers)
cargo run

# Check compilation
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Build release binary
cargo build --release
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test <test_name>

# Run tests in specific module
cargo test <module_name>::tests

# Run tests with output
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored
```

## Code Style Guidelines

### Imports
- Organize in three blocks: `std`, external crates, local modules
- Use grouped imports with curly braces for clarity
```rust
use std::sync::Arc;
use tokio::net::TcpListener;
use crate::{models::Post, Error, Result};
```

### Type Annotations
- **Explicit**: Struct fields, function parameters, return types
- **Implicit**: Local variables when type is obvious from context
```rust
pub async fn create_post(State(state): State<Arc<AppState>>) -> Result<Json<Post>>
```

### Naming Conventions
- **snake_case**: Variables, functions, modules (`db_pool`, `get_post`)
- **PascalCase**: Structs, enums, traits (`AppState`, `User`, `Error`)
- **SCREAMING_SNAKE_CASE**: Environment variables (`DATABASE_URL`)

### Error Handling
- Centralized error enum in `src/error.rs` using `thiserror`
- Custom `Result<T>` type alias crate-wide
- Never suppress errors with `.expect()` outside main.rs
- Use `#[from]` for automatic error conversions
- Implement `IntoResponse` for web error handling

```rust
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found")]
    NotFound,
}
```

### Async Patterns
- Use `#[tokio::main]` for entry points
- Use `#[tokio::test]` for async tests
- Use `.await?` for error propagation in async contexts
- Concurrent execution with `tokio::select!` for multiple futures

### Database (SQLx)
- Use `sqlx::query_as!()` for compile-time checked queries when possible
- Use `sqlx::query_as::<_, Model>()` for dynamic queries
- Bind parameters with `.bind()` - NEVER interpolate strings
- All queries must be type-safe at compile time
- **IMPORTANT**: After modifying queries, regenerate offline metadata for Docker builds

```rust
let post = sqlx::query_as::<_, Post>("SELECT * FROM posts WHERE id = $1")
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(Error::NotFound)?;
```

**SQLx Offline Mode (Required for Docker Builds):**
```bash
# Install sqlx-cli if not already installed
cargo install sqlx-cli --no-default-features --features postgres

# Set DATABASE_URL
export DATABASE_URL="postgresql://iron_bbs:iron_bbs@localhost:5432/iron_bbs"

# Ensure database is running
docker-compose up -d postgres

# Generate offline query metadata
cargo sqlx prepare

# This creates/updates the .sqlx/ directory
# MUST commit this directory to git for Docker builds to work
git add .sqlx/
git commit -m "Update sqlx offline query metadata"
```

### Web (Axum)
- Handlers accept `State<Arc<AppState>>` for shared state
- Return `Result<Json<T>>` for JSON responses
- Return `Result<Response>` for HTML/custom responses
- Use Axum extractors: `Path`, `State`, `Json`, `Query`, `Form`, `Cookies`
- Protected routes should check authentication via `check_auth()` helper
- Use `Redirect::to()` for navigation after form submissions

**Authentication Pattern:**
```rust
pub async fn protected_route(
    cookies: Cookies,
    State(state): State<Arc<AppState>>,
) -> Result<Response> {
    let user = check_auth(&cookies, &state.db).await;
    if user.is_none() {
        return Ok(Redirect::to("/login").into_response());
    }
    // Handle authenticated request
}
```

### SSH (Russh)
- Implement `server::Server` and `server::Handler` traits
- Use `&mut self` pattern (Russh 0.56+)
- Manage client sessions with `Arc<Mutex<HashMap<>>>`
- Handle window resize events properly

### Models
- Use `#[derive(sqlx::FromRow)]` for database models
- Use `#[derive(Serialize, Deserialize)]` for API models
- Keep models in `src/models.rs`
- Use UUID for primary keys

### Testing Patterns
- Unit tests inline in `#[cfg(test)] mod tests`
- Use standard `assert!`, `assert_eq!`, `assert_ne!`
- Add tests to `src/auth.rs` for new auth logic
- Currently minimal - expand coverage when adding features

### State Management
- Shared state (DB pool) wrapped in `Arc`
- Pass `Arc<AppState>` to handlers via Axum's `State` extractor
- Clone `Arc` references for SSH threads

### Logging
- Use `tracing` for instrumentation
- Use `tracing::info!`, `tracing::error!`, `tracing::warn!`
- Never use `println!` or `eprintln!` in library code
- Configure log level via `RUST_LOG` env var

## Project Structure

```
src/
├── main.rs          # Entry point, spawns web/SSH servers
├── lib.rs           # Module exports (Config, Error, Result)
├── config.rs        # Configuration loading
├── db.rs            # Database connection pool
├── error.rs         # Centralized error handling
├── models.rs        # Data models (User, Post, Session)
├── auth.rs          # Authentication logic
├── web/             # HTTP server (Axum)
│   ├── mod.rs
│   ├── routes.rs    # Router definition
│   └── handlers.rs  # Request handlers
└── ssh/             # SSH server (Russh)
    ├── mod.rs
    ├── server.rs    # SSH server implementation
    ├── terminal.rs  # Terminal handling
    └── ui.rs        # TUI rendering
```

## Configuration

- Config loaded from `.env` file via `config` crate
- Required env vars: `DATABASE_URL`, `WEB_ADDRESS`, `SSH_ADDRESS`
- Log level: `RUST_LOG` (default: `info`)

## Key Dependencies

- **Runtime**: `tokio` (async runtime)
- **Web**: `axum` (HTTP server), `tower-http` (middleware)
- **SSH**: `russh` (SSH protocol)
- **Database**: `sqlx` (PostgreSQL, compile-time checked)
- **Templates**: `askama` (type-safe HTML templates)
- **Error**: `thiserror` (error enums), `anyhow` (error contexts)
- **TUI**: `ratatui`, `crossterm` (terminal UI)
- **Logging**: `tracing`, `tracing-subscriber`

## Before Committing Changes

1. Run `cargo fmt` to format code
2. Run `cargo clippy` - fix all warnings
3. Run `cargo test` - ensure all tests pass
4. Run `cargo check` - verify compilation
5. Check LSP diagnostics on changed files

## Common Patterns

### Adding a new API endpoint
1. Add route in `src/web/routes.rs`
2. Implement handler in `src/web/handlers.rs`
3. Return appropriate `Result<Json<T>>` or `Result<Response>`
4. Handle errors via `IntoResponse` trait

### Adding a new SSH command
1. Add command parsing in `src/ssh/server.rs` (data handler)
2. Implement command logic
3. Update TUI rendering in `src/ssh/ui.rs` if needed
4. Handle terminal I/O via `TerminalHandle`

### Database migration
```bash
# Create migration
sqlx migrate add <name>

# Run migrations (automatic on startup)
# Or manually:
sqlx migrate run
```

## Notes

- No authentication system currently (SSH accepts all connections)
- No CI/CD configured
- Uses default `rustfmt` and `clippy` settings (no custom config)
- Single binary monolith - shared state between web and SSH
- Always use type-safe SQLx queries - never string interpolation
