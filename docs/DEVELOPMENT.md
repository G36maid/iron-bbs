# Development Guide

## Prerequisites

- Rust 1.75 or later
- Docker & Docker Compose
- PostgreSQL (via Docker)
- sqlx-cli (for database operations)

## Setup

### 1. Clone and Configure

```bash
git clone <repository-url>
cd iron-bbs
cp .env.example .env
```

### 2. Start Database

```bash
docker-compose up -d postgres
```

### 3. Install SQLx CLI

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

### 4. Run Migrations

```bash
export DATABASE_URL="postgresql://iron_bbs:iron_bbs@localhost:5432/iron_bbs"
sqlx migrate run
```

### 5. Run Application

```bash
cargo run
```

Application will be available at:
- Web: http://localhost:3000
- SSH: ssh -p 2222 localhost

## Development Workflow

### Code Style

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check compilation
cargo check
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Database Operations

**Create Migration:**
```bash
sqlx migrate add migration_name
```

**Run Migrations:**
```bash
sqlx migrate run
```

**Revert Migration:**
```bash
sqlx migrate revert
```

**Update SQLx Offline Data (Required after query changes):**
```bash
export DATABASE_URL="postgresql://iron_bbs:iron_bbs@localhost:5432/iron_bbs"
cargo sqlx prepare
git add .sqlx/
```

## Project Structure

```
iron-bbs/
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Module exports
│   ├── config.rs         # Configuration
│   ├── error.rs          # Error types
│   ├── auth.rs           # Authentication service
│   ├── db.rs             # Database connection
│   ├── models.rs         # Data models
│   ├── web/              # HTTP server
│   │   ├── mod.rs
│   │   ├── routes.rs     # Route definitions
│   │   └── handlers.rs   # Request handlers
│   └── ssh/              # SSH server
│       ├── mod.rs
│       ├── server.rs     # SSH implementation
│       ├── terminal.rs   # Terminal handling
│       └── ui.rs         # TUI rendering
├── templates/            # Askama HTML templates
├── migrations/           # SQL migrations
├── .sqlx/               # SQLx offline query data
└── docs/                # Documentation
```

## Adding Features

### 1. Database Changes

```bash
# Create migration
sqlx migrate add add_feature_table

# Edit migrations/<timestamp>_add_feature_table.sql
# Run migration
sqlx migrate run

# Update offline data
cargo sqlx prepare
```

### 2. Add Model

Edit `src/models.rs`:
```rust
#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Feature {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
```

### 3. Add Web Handler

Edit `src/web/handlers.rs`:
```rust
pub async fn get_feature(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Feature>> {
    let feature = sqlx::query_as!(
        Feature,
        "SELECT * FROM features WHERE id = $1",
        id
    )
    .fetch_one(&state.db)
    .await?;
    
    Ok(Json(feature))
}
```

### 4. Add Route

Edit `src/web/routes.rs`:
```rust
.route("/features/:id", get(handlers::get_feature))
```

### 5. Add Template (if needed)

Create `templates/feature.html`:
```html
{% extends "base.html" %}

{% block title %}{{ feature.name }}{% endblock %}

{% block content %}
<h1>{{ feature.name }}</h1>
{% endblock %}
```

## Docker Development

### Build Image

```bash
docker-compose build app
```

### Run with Docker Compose

```bash
docker-compose up -d
```

### View Logs

```bash
docker-compose logs -f app
```

### Rebuild After Changes

```bash
# Update SQLx offline data first
cargo sqlx prepare

# Rebuild image
docker-compose build --no-cache app
docker-compose up -d
```

## Common Tasks

### Reset Database

```bash
docker-compose down -v
docker-compose up -d postgres
sqlx migrate run
```

### Generate Test Data

```bash
# Add to migrations or create a seed script
sqlx migrate add seed_test_data
```

### Debug SQLx Queries

```bash
# Check prepared queries
ls -la .sqlx/

# Regenerate if stale
cargo sqlx prepare --check
```

### Profile Performance

```bash
cargo build --release
cargo flamegraph --bin iron-bbs
```

## Troubleshooting

### Compilation Errors

**SQLx offline data mismatch:**
```bash
cargo sqlx prepare
```

**Dependency issues:**
```bash
cargo clean
cargo update
cargo build
```

### Database Issues

**Connection refused:**
```bash
docker-compose up -d postgres
docker-compose ps
```

**Migration conflicts:**
```bash
sqlx migrate revert
sqlx migrate run
```

### Docker Issues

**Build fails:**
```bash
# Ensure .sqlx/ is up to date
cargo sqlx prepare

# Clean build
docker-compose down
docker-compose build --no-cache app
```

**Container exits immediately:**
```bash
docker-compose logs app
```

## Code Guidelines

See [AGENTS.md](../AGENTS.md) for detailed coding standards including:
- Code style and formatting
- Error handling patterns
- Async patterns
- Database query patterns
- Authentication implementation
- Testing practices

## Contributing

1. Create feature branch
2. Make changes
3. Run tests and linters
4. Update documentation
5. Submit pull request

## Resources

- [SQLx Documentation](https://docs.rs/sqlx)
- [Axum Documentation](https://docs.rs/axum)
- [Russh Documentation](https://docs.rs/russh)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
