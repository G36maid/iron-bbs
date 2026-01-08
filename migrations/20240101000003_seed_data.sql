INSERT INTO users (id, username, email, password_hash)
VALUES 
    ('550e8400-e29b-41d4-a716-446655440000'::uuid, 'admin', 'admin@example.com', '$argon2id$v=19$m=19456,t=2,p=1$VE0rM09wSHNsWmVPejJnaw$iOyQaJQp7nXzZqK/0dHGKIUEqZIk8nzHZ3GQwNcMTbU');

INSERT INTO posts (title, content, author_id, published)
VALUES
    ('Welcome to Iron BBS', 'This is a high-performance blogging platform built with Rust. It features a unique dual-interface architecture serving content via both HTTP and SSH!', '550e8400-e29b-41d4-a716-446655440000'::uuid, true),
    ('Getting Started with Rust', 'Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.', '550e8400-e29b-41d4-a716-446655440000'::uuid, true),
    ('Async Rust with Tokio', 'Tokio is an asynchronous runtime for the Rust programming language. It provides the building blocks for writing networking applications.', '550e8400-e29b-41d4-a716-446655440000'::uuid, true);
