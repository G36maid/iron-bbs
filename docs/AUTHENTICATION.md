# Authentication Guide

## Overview

Iron BBS implements session-based authentication with Argon2 password hashing for the web interface. This document covers the authentication system architecture, usage, and security considerations.

## Features

### Implemented
- User registration with email and password
- Session-based authentication (7-day expiration)
- Argon2 password hashing
- HTTP-only cookies (XSS protection)
- Protected routes (require authentication)
- Session management (login/logout)
- Login IP tracking
- Duplicate username/email prevention

### Security Features
- **Password Hashing**: Argon2 algorithm (memory-hard, resistant to GPU attacks)
- **HTTP-only Cookies**: Prevents JavaScript access to session tokens
- **Session Expiration**: Automatic logout after 7 days of inactivity
- **SQL Injection Prevention**: Parameterized queries via SQLx
- **Input Validation**: Username (min 3 chars), password (min 8 chars)
- **Unique Constraints**: Username and email must be unique

## Database Schema

### Users Table
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_ip INET,
    last_login_at TIMESTAMPTZ
);
```

### Sessions Table
```sql
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(255) UNIQUE NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

## Usage

### User Registration

**Browser:**
1. Navigate to `http://localhost:3000/register`
2. Fill in username, email, and password
3. Submit form → Automatically logged in → Redirected to homepage

**API:**
```bash
curl -c cookies.txt -X POST http://localhost:3000/register \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=testuser&email=test@example.com&password=securepass123"
```

### User Login

**Browser:**
1. Navigate to `http://localhost:3000/login`
2. Enter username and password
3. Submit form → Session created → Redirected to homepage

**API:**
```bash
curl -c cookies.txt -X POST http://localhost:3000/login \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=testuser&password=securepass123"
```

### Creating Posts (Protected Route)

**Browser:**
1. Login first
2. Click "New Post" button in navigation
3. Fill in title and content
4. Optionally check "Publish immediately"
5. Submit → Post created

**API:**
```bash
curl -b cookies.txt -X POST http://localhost:3000/new \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "title=My Post&content=Post content&published=on"
```

### Logout

**Browser:**
Click "Logout" button in navigation

**API:**
```bash
curl -b cookies.txt -X POST http://localhost:3000/logout
```

## Implementation Details

### Authentication Flow

1. **Registration:**
   - Validate username (3+ chars), email format, password (8+ chars)
   - Check for duplicate username/email
   - Hash password with Argon2
   - Insert user into database
   - Generate session token (UUID v4)
   - Set session cookie (HTTP-only, 7-day expiration)
   - Redirect to homepage

2. **Login:**
   - Lookup user by username
   - Verify password with Argon2
   - Generate new session token
   - Set session cookie
   - Update last login IP and timestamp
   - Redirect to homepage

3. **Protected Routes:**
   - Extract session cookie
   - Validate token against database
   - Check expiration timestamp
   - Fetch user data
   - Allow access or redirect to login

4. **Logout:**
   - Delete session from database
   - Remove session cookie
   - Redirect to homepage

### Session Management

**Session Token Generation:**
```rust
use uuid::Uuid;

pub fn generate_session_token() -> String {
    Uuid::new_v4().to_string()
}
```

**Session Expiration:**
- Default: 7 days from creation
- Configurable via `Duration::days(7)`
- Checked on every request

**Cookie Configuration:**
```rust
let mut cookie = Cookie::new("session_id", token);
cookie.set_path("/");
cookie.set_http_only(true);
// Optional: cookie.set_secure(true); // HTTPS only
```

### Password Hashing

**Hashing (Registration):**
```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

let salt = SaltString::generate(&mut OsRng);
let argon2 = Argon2::default();
let password_hash = argon2
    .hash_password(password.as_bytes(), &salt)?
    .to_string();
```

**Verification (Login):**
```rust
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};

let parsed_hash = PasswordHash::new(&user.password_hash)?;
Argon2::default().verify_password(password.as_bytes(), &parsed_hash)?;
```

## Security Considerations

### Current Limitations
- No CSRF protection
- No rate limiting on authentication endpoints
- No email verification
- No password reset functionality
- No account lockout after failed attempts
- No "remember me" option
- Sessions tied to cookies only (no token-based auth)

### Production Recommendations

1. **CSRF Protection:**
   ```bash
   cargo add tower-csrf
   ```
   Add CSRF tokens to all forms

2. **Rate Limiting:**
   ```bash
   cargo add tower-governor
   ```
   Limit login attempts (e.g., 5 per minute per IP)

3. **Email Verification:**
   - Add `email_verified` column to users table
   - Generate verification tokens
   - Send verification emails via SMTP

4. **Password Requirements:**
   - Enforce stronger passwords (uppercase, lowercase, numbers, symbols)
   - Check against common password lists
   - Implement password strength meter

5. **Account Security:**
   - Implement account lockout (e.g., 15 minutes after 5 failed attempts)
   - Add two-factor authentication (TOTP)
   - Log security events (failed logins, IP changes)

6. **Session Security:**
   - Use `Secure` flag on cookies (HTTPS only)
   - Implement session fingerprinting (User-Agent, IP)
   - Add "Remember Me" with longer expiration
   - Allow users to view/revoke active sessions

7. **HTTPS:**
   - Use reverse proxy (nginx/caddy) for TLS termination
   - Redirect HTTP to HTTPS
   - Set HSTS header

## Testing Authentication

### Manual Testing

**Test Registration:**
```bash
# Success case
curl -c cookies.txt -X POST http://localhost:3000/register \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=testuser&email=test@example.com&password=testpass123"

# Should return HTTP 303 and set session cookie
```

**Test Login:**
```bash
# Success case
curl -c cookies.txt -X POST http://localhost:3000/login \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=testuser&password=testpass123"

# Failed case (wrong password)
curl -X POST http://localhost:3000/login \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=testuser&password=wrongpassword"
# Should return error message
```

**Test Protected Route:**
```bash
# Without authentication (should redirect to login)
curl -L http://localhost:3000/new

# With authentication (should show form)
curl -b cookies.txt http://localhost:3000/new
```

**Test Logout:**
```bash
curl -b cookies.txt -X POST http://localhost:3000/logout
# Session should be deleted from database
```

### Automated Testing

Add unit tests in `src/auth.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let password = "testpassword123";
        let hash = AuthService::hash_password(password).unwrap();
        
        assert!(AuthService::verify_password(password, &hash).is_ok());
        assert!(AuthService::verify_password("wrongpassword", &hash).is_err());
    }

    #[test]
    fn test_session_token_unique() {
        let token1 = AuthService::generate_session_token();
        let token2 = AuthService::generate_session_token();
        
        assert_ne!(token1, token2);
    }
}
```

## Troubleshooting

### Session not persisting
**Symptom:** User logs in but is immediately logged out

**Solutions:**
- Check browser cookie settings (cookies must be enabled)
- Verify `cookie.set_path("/")` is set correctly
- Check database sessions table for expired entries
- Ensure `expires_at` is in the future

### Password verification fails
**Symptom:** Correct password rejected at login

**Solutions:**
- Verify password hash format in database
- Check Argon2 version compatibility
- Ensure password wasn't truncated during storage
- Verify character encoding (UTF-8)

### Duplicate username/email errors
**Symptom:** Registration fails with constraint violation

**Solutions:**
- Check if username already exists: `SELECT * FROM users WHERE username = 'x'`
- Check if email already exists: `SELECT * FROM users WHERE email = 'x'`
- Add better error messages to registration form

### Session cookie not set
**Symptom:** No session cookie after login

**Solutions:**
- Check response headers for `Set-Cookie`
- Verify cookie domain matches request origin
- Ensure `HttpOnly` flag is compatible with client
- Check for conflicts with other middleware

## API Reference

### Endpoints

| Method | Path | Description | Auth Required |
|--------|------|-------------|---------------|
| GET | `/register` | Registration form | No |
| POST | `/register` | Create new user | No |
| GET | `/login` | Login form | No |
| POST | `/login` | Authenticate user | No |
| POST | `/logout` | End session | Yes |
| GET | `/new` | Post creation form | Yes |
| POST | `/new` | Submit new post | Yes |

### Request/Response Examples

**POST /register**
```
Request:
  Content-Type: application/x-www-form-urlencoded
  Body: username=testuser&email=test@example.com&password=pass123

Response:
  Status: 303 See Other
  Location: /
  Set-Cookie: session_id=<uuid>; HttpOnly; Path=/
```

**POST /login**
```
Request:
  Content-Type: application/x-www-form-urlencoded
  Body: username=testuser&password=pass123

Response (Success):
  Status: 303 See Other
  Location: /
  Set-Cookie: session_id=<uuid>; HttpOnly; Path=/

Response (Failed):
  Status: 200 OK
  Body: <HTML with error message>
```

**POST /logout**
```
Request:
  Cookie: session_id=<uuid>

Response:
  Status: 303 See Other
  Location: /
  Set-Cookie: session_id=; Max-Age=0
```

## Future Enhancements

- [ ] CSRF protection
- [ ] Rate limiting
- [ ] Email verification
- [ ] Password reset via email
- [ ] Two-factor authentication (TOTP)
- [ ] OAuth2 integration (Google, GitHub)
- [ ] Session management dashboard
- [ ] Account lockout policy
- [ ] Password history (prevent reuse)
- [ ] Security event logging
- [ ] IP-based restrictions
- [ ] Remember me functionality
