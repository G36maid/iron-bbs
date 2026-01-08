# Multi-stage build for optimized image size
FROM rust:1.91-slim as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations
COPY templates ./templates
COPY .sqlx ./.sqlx

ENV SQLX_OFFLINE=true
RUN cargo build --release

# Runtime stage - minimal image
FROM debian:trixie-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -U -s /bin/bash iron_bbs

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/iron-bbs /app/iron-bbs

# Copy templates directory
COPY --from=builder /app/templates /app/templates

# Copy migrations directory
COPY --from=builder /app/migrations /app/migrations

# Create data directory for persistent SSH keys
RUN mkdir -p /app/data && chown iron_bbs:iron_bbs /app/data

# Change ownership
RUN chown -R iron_bbs:iron_bbs /app

# Switch to non-root user
USER iron_bbs

# Expose ports
EXPOSE 3000 2222

# Set default environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=postgresql://iron_bbs:iron_bbs@postgres:5432/iron_bbs
ENV WEB_PORT=3000
ENV SSH_PORT=2222

# Run the application
CMD ["/app/iron-bbs"]
