# Iron BBS

[![CI](https://github.com/G36_maid/iron-bbs/workflows/CI/badge.svg)](https://github.com/G36_maid/iron-bbs/actions)

A high-performance **monolithic blogging platform** built in Rust featuring a unique **dual-interface architecture** that serves content via both **HTTP** (web browser) and **SSH** (terminal).

## Features

- 🌐 **Dual Interface**: Access via web browser or SSH terminal
- 🔒 **Authentication**: Session-based login with Argon2 password hashing
- ⚡ **Fast & Safe**: Built on Tokio with compile-time SQL verification
- 🐳 **Docker Ready**: One command deployment
- 📝 **Modern Stack**: Axum, SQLx, Russh, PostgreSQL

## Quick Start

```bash
# Clone and start
git clone <repository-url>
cd iron-bbs
docker-compose up -d

# Access
# Web:  http://localhost:3000
# SSH:  ssh -p 2222 localhost
```

## Usage

### Web Interface

**Register & Login:**
```
http://localhost:3000/register
http://localhost:3000/login
```

**Create Posts:**
- Login → Click "New Post" → Fill form → Submit

### SSH Interface

```bash
ssh -p 2222 localhost

# Available commands:
list              # List recent posts
view <id>         # View post details
delete <id>       # Delete post
help              # Show commands
quit              # Exit
```

## Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Rust |
| Web Framework | Axum |
| SSH Server | Russh |
| Database | PostgreSQL |
| ORM | SQLx |
| Templates | Askama |
| Runtime | Tokio |

## Documentation

- [Authentication Guide](docs/AUTHENTICATION.md) - User auth, sessions, security
- [Development Guide](docs/DEVELOPMENT.md) - Setup, workflow, contributing
- [Docker Deployment](docs/DOCKER_DEPLOYMENT.md) - Production deployment
- [SSH Troubleshooting](docs/SSH_TROUBLESHOOTING.md) - SSH connection issues
- [Agent Development](AGENTS.md) - AI agent coding standards

## API Endpoints

```
GET  /                  - Homepage
GET  /login             - Login page
POST /login             - Authenticate
GET  /register          - Registration
POST /register          - Create account
POST /logout            - Logout
GET  /new               - Create post (requires auth)
POST /new               - Submit post (requires auth)
GET  /api/posts         - List posts (JSON)
POST /api/posts         - Create post (JSON)
```

## Development

### Prerequisites

- Rust 1.75+
- Docker & Docker Compose
- sqlx-cli: `cargo install sqlx-cli --no-default-features --features postgres`

### Local Setup

```bash
# Start database
docker-compose up -d postgres

# Run migrations
export DATABASE_URL="postgresql://iron_bbs:iron_bbs@localhost:5432/iron_bbs"
sqlx migrate run

# Run application
cargo run
```

### Building

```bash
cargo build --release
```

### Testing

```bash
cargo test
cargo clippy
cargo fmt
```

## Project Structure

```
iron-bbs/
├── src/
│   ├── main.rs         # Entry point
│   ├── auth.rs         # Authentication
│   ├── web/            # HTTP server
│   └── ssh/            # SSH server
├── templates/          # HTML templates
├── migrations/         # Database migrations
├── .sqlx/             # SQLx offline data
└── docs/              # Documentation
```

## Security Features

✅ Argon2 password hashing  
✅ HTTP-only session cookies  
✅ Session expiration (7 days)  
✅ SQL injection prevention  
✅ Input validation  

⚠️ No CSRF protection (add for production)  
⚠️ No rate limiting (add for production)  
⚠️ SSH accepts all connections (demo only)  

## Configuration

Environment variables (`.env`):

```env
DATABASE_URL=postgresql://iron_bbs:iron_bbs@postgres:5432/iron_bbs
WEB_ADDRESS=0.0.0.0:3000
SSH_ADDRESS=0.0.0.0:2222
RUST_LOG=info
```

## Docker Deployment

```bash
# Build and run
docker-compose up -d

# View logs
docker-compose logs -f

# Stop
docker-compose down
```

**After modifying database queries:**

```bash
cargo sqlx prepare
git add .sqlx/
docker-compose build --no-cache app
```

## Troubleshooting

### Database connection failed
```bash
docker-compose up -d postgres
```

### Port already in use
Change `WEB_ADDRESS` or `SSH_ADDRESS` in `.env`

### Docker build fails
```bash
cargo sqlx prepare
docker-compose build --no-cache app
```

See [docs/](docs/) for detailed troubleshooting.

## Architecture

```mermaid
graph TD
    subgraph Binary [Iron BBS - Single Rust Binary]
        Web[Web Interface<br>Port 3000]
        SSH[SSH Interface<br>Port 2222]
        Pool[DB Pool]
    end
    
    Web --> Pool
    SSH --> Pool
```

## Why Iron BBS?

- **Monolithic**: Single binary, simple deployment, no microservice overhead
- **Dual Interface**: Unique SSH access for terminal enthusiasts
- **Type Safety**: Compile-time SQL verification prevents runtime errors
- **Performance**: Rust + Tokio = blazing fast async I/O
- **Developer Experience**: Hot reload, instant feedback, clear errors

## Contributing

1. Fork repository
2. Create feature branch
3. Make changes
4. Run tests: `cargo test`
5. Submit pull request

See [DEVELOPMENT.md](docs/DEVELOPMENT.md) for guidelines.

## License

MIT License

## Credits

Built with:
- [Tokio](https://tokio.rs/) - Async runtime
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Russh](https://github.com/Eugeny/russh) - SSH protocol
- [SQLx](https://github.com/launchbadge/sqlx) - SQL toolkit
- [Askama](https://github.com/djc/askama) - Templates
