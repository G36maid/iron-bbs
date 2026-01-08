# Rusty BBS

A high-performance, **monolithic blogging platform** built in Rust featuring a unique **dual-interface architecture** that serves identical content via both **HTTP/HTTPS** (modern web browser) and **SSH** (terminal interface).

## Architecture

```mermaid
graph TD
    subgraph Binary [Rusty BBS - Single Rust Binary]
        direction TB
        Web[Web Interface<br>Axum HTTP<br>Port 3000]
        SSH[SSH Interface<br>Russh Server<br>Port 2222]
        Pool[Shared DB Connection Pool<br>Arc PgPool]
    end

    Web --> Pool
    SSH --> Pool
```

## Features

### Core
- **Dual Interface**: Access the same blog content via web browser or SSH terminal
- **Monolithic Design**: Single binary, shared database, unified state
- **Async Runtime**: Built on Tokio for high concurrency
- **Type-Safe SQL**: Compile-time query verification with SQLx
- **Template-Based Rendering**: Askama templates with Tailwind CSS

### Web Interface (HTTP)
- Browse posts with modern, responsive UI
- REST API for CRUD operations
- JSON endpoints for programmatic access
- Tailwind CSS styling

### SSH Interface (Terminal)
- Text-based command interface
- Real-time blog browsing via SSH
- CRUD operations from terminal
- ASCII art formatting

### Available Commands (SSH)
```
list            - List recent published posts
view <id>       - Display full post content
delete <id>     - Delete a post by ID
help            - Show available commands
quit/exit       - Close SSH connection
```

### API Endpoints (Web)
```
GET  /                  - Homepage with post listing
GET  /posts/:id         - View individual post
GET  /api/posts         - List all posts (JSON)
POST /api/posts         - Create new post
PUT  /api/posts/:id     - Update post
DELETE /api/posts/:id   - Delete post
GET  /health            - Health check
```

## Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Rust |
| Async Runtime | Tokio |
| Web Framework | Axum |
| SSH Server | Russh |
| Templates | Askama |
| Database | PostgreSQL |
| ORM | SQLx (compile-time checked) |
| Styling | Tailwind CSS |
| Containerization | Docker Compose |

## Project Structure

```
rusty-bbs/
├── Cargo.toml              # Dependencies and project config
├── docker-compose.yml      # PostgreSQL container
├── .env.example            # Environment variable template
├── migrations/             # Database migrations
│   ├── 20240101000001_create_users.sql
│   ├── 20240101000002_create_posts.sql
│   └── 20240101000003_seed_data.sql
├── templates/              # Askama HTML templates
│   ├── index.html
│   └── post.html
└── src/
    ├── main.rs            # Entry point, spawns both servers
    ├── lib.rs             # Module exports
    ├── config.rs          # Configuration loading
    ├── error.rs           # Error types
    ├── db/                # Database connection
    │   └── mod.rs
    ├── models/            # Data models
    │   ├── mod.rs
    │   ├── user.rs
    │   └── post.rs
    ├── web/               # HTTP server
    │   ├── mod.rs
    │   ├── routes.rs
    │   └── handlers.rs
    └── ssh/               # SSH server
        ├── mod.rs
        └── server.rs
```

## Prerequisites

- **Rust**: 1.75 or later ([Install Rust](https://www.rust-lang.org/tools/install))
- **Docker & Docker Compose**: For PostgreSQL ([Install Docker](https://docs.docker.com/get-docker/))
- **SSH Client**: OpenSSH or compatible (usually pre-installed on Linux/macOS)

## Quick Start

### 1. Clone and Setup

```bash
git clone <repository-url>
cd rusty-bbs
cp .env.example .env
```

### 2. Start PostgreSQL

```bash
docker-compose up -d
```

### 3. Run the Application

```bash
cargo run
```

The application will:
- Run database migrations automatically
- Start web server on `http://localhost:3000`
- Start SSH server on `ssh://localhost:2222`

## Usage

### Web Interface

Open your browser:
```
http://localhost:3000
```

Create a post via API:
```bash
curl -X POST http://localhost:3000/api/posts \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Hello World",
    "content": "My first post!",
    "author_id": "550e8400-e29b-41d4-a716-446655440000",
    "published": true
  }'
```

### SSH Interface

Connect via SSH:
```bash
ssh -p 2222 localhost
```

Example session:
```
╔════════════════════════════════════════════╗
║   Welcome to Rusty BBS (SSH Interface)  ║
╚════════════════════════════════════════════╝

Commands:
  list - List recent posts
  view <id> - View post details
  delete <id> - Delete a post
  help - Show this help
  quit - Exit

> list
╔══════════════════════════════════════════════════════════════════════╗
║                          RECENT POSTS                                ║
╚══════════════════════════════════════════════════════════════════════╝

1. Welcome to Rusty BBS
   ID: f2f49db6-9891-4c05-9d89-6ff293e09f34
   This is a high-performance blogging platform...

> view f2f49db6-9891-4c05-9d89-6ff293e09f34
╔══════════════════════════════════════════════════════════════════════╗
║ Welcome to Rusty BBS                                                 ║
╚══════════════════════════════════════════════════════════════════════╝

ID: f2f49db6-9891-4c05-9d89-6ff293e09f34
Author: 550e8400-e29b-41d4-a716-446655440000
Published: Yes
Created: 2026-01-08 07:32:27

[Full post content displays here...]

> quit
Goodbye!
```

## Configuration

Environment variables (`.env` file):

```env
DATABASE_URL=postgres://postgres:postgres@localhost:5432/rustybbs
WEB_ADDRESS=0.0.0.0:3000
SSH_ADDRESS=0.0.0.0:2222
RUST_LOG=info
```

## Database Schema

### Users Table
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Posts Table
```sql
CREATE TABLE posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    author_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published BOOLEAN NOT NULL DEFAULT false
);
```

## Development

### Build Release Binary
```bash
cargo build --release
```

### Run Tests
```bash
cargo test
```

### Check Code
```bash
cargo check
cargo clippy
```

### Database Migrations

Create a new migration:
```bash
sqlx migrate add <migration_name>
```

Run migrations:
```bash
sqlx migrate run
```

Revert last migration:
```bash
sqlx migrate revert
```

## Deployment

### Docker Build (Optional)
```bash
docker build -t rusty-bbs .
docker run -p 3000:3000 -p 2222:2222 rusty-bbs
```

### Production Considerations
- Set `RUST_LOG=warn` in production
- Use proper PostgreSQL credentials
- Configure firewall rules for ports 3000 and 2222
- Consider using a reverse proxy (nginx/caddy) for HTTPS
- Change default SSH port if needed
- Implement rate limiting
- Add authentication (currently accepts all connections)

## Security Notes

⚠️ **Current Implementation**:
- SSH server accepts **all connections without authentication**
- No user authentication on web interface
- Suitable for **demonstration purposes only**

For production use, implement:
- SSH public key authentication
- Web session management
- Password hashing (Argon2)
- CSRF protection
- Rate limiting
- Input validation

## Design Decisions

### Why Monolithic?
- **Simplicity**: Single binary, single deployment
- **Performance**: No network overhead between services
- **Development Speed**: Shared types, single codebase
- **Operational Simplicity**: One process to monitor

### Why Dual Interface?
- **Demonstration**: Showcases Rust's versatility
- **Accessibility**: Terminal access for SSH-only environments
- **Uniqueness**: Novel approach to blogging platforms

### Why Russh?
- Overcame trait lifetime issues present in older versions
- Uses simpler `&mut self` signatures
- Better maintained and more stable

## Known Limitations

- No authentication system (SSH/Web)
- No comment system
- No user roles/permissions
- No file upload support
- No full-text search
- No Ratatui TUI (planned but not implemented)
- SSH interface is command-line based, not interactive TUI

## Future Enhancements

- [ ] Authentication and authorization
- [ ] User registration and profiles
- [ ] Comment system
- [ ] Rich TUI with Ratatui
- [ ] Full-text search
- [ ] Markdown support
- [ ] Image uploads
- [ ] RSS feed
- [ ] Admin dashboard

## License

MIT License

## Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## Troubleshooting

### Database Connection Failed
```
Error: Failed to connect to database
```
**Solution**: Ensure PostgreSQL is running via `docker-compose up -d`

### Port Already in Use
```
Error: Address already in use
```
**Solution**: Change `WEB_ADDRESS` or `SSH_ADDRESS` in `.env`

### SSH Connection Refused
```
ssh: connect to host localhost port 2222: Connection refused
```
**Solution**: Verify application is running and port 2222 is accessible

### Compilation Errors
```
error: failed to compile
```
**Solution**: Ensure Rust 1.75+ is installed: `rustc --version`

## Credits

Built with:
- [Tokio](https://tokio.rs/) - Asynchronous runtime
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Russh](https://github.com/Eugeny/russh) - SSH protocol
- [SQLx](https://github.com/launchbadge/sqlx) - SQL toolkit
- [Askama](https://github.com/djc/askama) - Type-safe templates
