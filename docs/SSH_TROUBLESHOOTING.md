# SSH Login Troubleshooting Guide

## Issue: SSH Login Failed

### Common Cause: Host Key Mismatch

**Problem:** When you restart the Docker container, a new SSH host key is generated, causing your SSH client to reject the connection with:

```
@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
@    WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!     @
@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
```

### Solution 1: Remove Old Host Key (Quick Fix)

```bash
ssh-keygen -R "[localhost]:2222"
ssh -p 2222 bbs@localhost
```

### Solution 2: Persistent SSH Keys (Permanent Fix)

The updated `docker-compose.yml` now includes a persistent volume for SSH keys:

```yaml
volumes:
  - ssh_keys:/app/data

environment:
  SSH_HOST_KEY_PATH: /app/data/ssh_host_key
```

This ensures the SSH host key remains the same across container restarts.

## Testing SSH Login

### Method 1: Automated Test Script

```bash
./scripts/test_ssh_login.sh
```

### Method 2: Manual Connection

```bash
ssh -p 2222 bbs@localhost
```

**Expected behavior:**
1. Connection establishes
2. PTY-style login screen appears
3. You can enter username and password

**Test credentials:**
- Username: `admin`
- Password: `admin123`

### Method 3: Bypass Host Key Check (Development Only)

```bash
ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -p 2222 bbs@localhost
```

**⚠️ Warning:** Only use this for testing. Never in production.

## Troubleshooting Steps

### 1. Check if SSH server is running

```bash
docker-compose ps
```

Expected output should show `iron-bbs-app-1` as `Up`.

### 2. Check SSH server logs

```bash
docker-compose logs app | grep -i ssh
```

Look for:
```
INFO iron_bbs::ssh::server: SSH server listening on 0.0.0.0:2222 (TUI mode)
```

### 3. Test port connectivity

```bash
nc -zv localhost 2222
```

Should return: `Connection to localhost 2222 port [tcp/*] succeeded!`

### 4. Verbose SSH connection

```bash
ssh -vvv -p 2222 bbs@localhost
```

This shows detailed connection steps and helps identify where it fails.

## Common Issues

### Issue: Connection Refused

**Cause:** App container not running or SSH server failed to start

**Fix:**
```bash
docker-compose restart app
docker-compose logs -f app
```

### Issue: Connection Timeout

**Cause:** Firewall blocking port 2222

**Fix:**
```bash
sudo ufw allow 2222/tcp
```

### Issue: Authentication Fails

**Cause 1:** Wrong username (must be "bbs" for guest login)

**Fix:** Use `ssh -p 2222 bbs@localhost` (not `admin@localhost`)

**Cause 2:** Database not initialized

**Fix:**
```bash
docker-compose down
docker-compose up -d
docker-compose logs app | grep migration
```

### Issue: Terminal Not Rendering Properly

**Cause:** Non-interactive SSH or missing PTY

**Fix:** Ensure you're connecting from an actual terminal, not piping through scripts

**Don't do this:**
```bash
echo "test" | ssh -p 2222 bbs@localhost
```

**Do this:**
```bash
ssh -p 2222 bbs@localhost
```

## Understanding PTY Requirements

The Iron BBS SSH interface is TUI-based (Terminal User Interface) and **requires a PTY** (pseudo-terminal).

**Will work:**
- `ssh -p 2222 bbs@localhost` (from interactive terminal)
- `ssh -t -p 2222 bbs@localhost` (explicit PTY request)

**Won't work:**
- `ssh -T -p 2222 bbs@localhost` (explicitly disables PTY)
- `echo "command" | ssh -p 2222 bbs@localhost` (stdin redirection)
- Scripts without PTY allocation

## Verifying the Fix

After applying solutions, verify SSH login works:

```bash
./scripts/test_ssh_login.sh
```

You should see:
1. Connection successful (no host key warnings)
2. Login prompt appears
3. Can enter credentials and interact with the BBS

## Persisted SSH Key Verification

To verify SSH keys persist across restarts:

```bash
# Get current host key fingerprint
ssh-keyscan -p 2222 localhost 2>/dev/null | ssh-keygen -lf -

# Restart container
docker-compose restart app

# Check fingerprint again
ssh-keyscan -p 2222 localhost 2>/dev/null | ssh-keygen -lf -
```

Both fingerprints should match if persistence is working correctly.
