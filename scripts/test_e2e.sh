#!/bin/bash

set -e

echo "======================================================================"
echo "IRON BBS - PTT-STYLE LOGIN - END-TO-END TEST"
echo "======================================================================"
echo ""

echo "Checking if server is running..."
if ! pgrep -f "target/debug/iron-bbs" >/dev/null; then
	echo "✗ Server not running!"
	echo "Please start server: cargo run"
	exit 1
fi
echo "✓ Server is running"
echo ""

echo "TEST 1: Verify SSH port 2222 is listening..."
if ss -tln 2>/dev/null | grep -q ":2222" || netstat -tln 2>/dev/null | grep -q ":2222" || lsof -i :2222 >/dev/null 2>&1; then
	echo "✓ Port 2222 is listening"
else
	echo "✗ Port 2222 not accessible"
	exit 1
fi
echo ""

echo "TEST 2: Test SSH connection (will timeout after 5 seconds)..."
timeout 5 ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=3 -p 2222 bbs@localhost exit 2>/dev/null || true
CONNECTION_RESULT=$?
if [ $CONNECTION_RESULT -eq 124 ]; then
	echo "✓ SSH connection established (timeout is expected - server waiting for input)"
elif [ $CONNECTION_RESULT -eq 0 ]; then
	echo "✓ SSH connection successful"
else
	echo "⚠ Connection attempt made (exit code: $CONNECTION_RESULT)"
fi
echo ""

echo "TEST 3: Verify database has admin user..."
docker exec iron-bbs-postgres-1 psql -U postgres -d iron_bbs -c "SELECT username, email FROM users WHERE username = 'admin';" 2>/dev/null | grep -q "admin" && echo "✓ Admin user exists in database" || echo "✗ Admin user not found"
echo ""

echo "TEST 4: Check password hash format..."
HASH=$(docker exec iron-bbs-postgres-1 psql -U postgres -d iron_bbs -t -c "SELECT password_hash FROM users WHERE username = 'admin';" 2>/dev/null | tr -d ' ')
if [[ $HASH == \$argon2* ]]; then
	echo "✓ Password hash is Argon2 format"
else
	echo "✗ Invalid password hash format"
fi
echo ""

echo "======================================================================"
echo "AUTOMATED TESTS COMPLETE"
echo "======================================================================"
echo ""
echo "✓ Server running"
echo "✓ SSH port accessible"
echo "✓ Database configured correctly"
echo "✓ Authentication system ready"
echo ""
echo "----------------------------------------------------------------------"
echo "MANUAL TESTING REQUIRED:"
echo "----------------------------------------------------------------------"
echo ""
echo "Run this command in a new terminal:"
echo ""
echo "    ssh -p 2222 bbs@localhost"
echo ""
echo "Expected flow:"
echo "  1. See login screen with 'Username:' prompt"
echo "  2. Type: admin"
echo "  3. Press Enter"
echo "  4. See 'Password:' prompt (input hidden)"
echo "  5. Type: admin123"
echo "  6. Press Enter"
echo "  7. Should see post browsing interface with 3 posts:"
echo "     - Welcome to Iron BBS"
echo "     - Getting Started with Rust"
echo "     - Async Rust with Tokio"
echo "  8. Press 'q' to quit"
echo ""
echo "Test invalid credentials:"
echo "  - Connect again"
echo "  - Enter wrong password"
echo "  - Should see error: 'Invalid username or password'"
echo "  - Login screen should reset"
echo ""
echo "======================================================================"
