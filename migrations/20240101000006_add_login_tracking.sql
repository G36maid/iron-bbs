ALTER TABLE users
ADD COLUMN last_login_ip TEXT,
ADD COLUMN last_login_at TIMESTAMPTZ;
