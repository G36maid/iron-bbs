#!/bin/bash
set -e

echo "🧪 Running iron-bbs integration tests..."

# Start PostgreSQL
echo "📦 Starting PostgreSQL..."
docker-compose up -d
sleep 2

# Wait for PostgreSQL to be healthy
echo "⏳ Waiting for PostgreSQL to be healthy..."
timeout 30 bash -c 'until docker-compose ps | grep -q "healthy"; do sleep 1; done'

# Build the application
echo "🔨 Building application..."
cargo build --release

# Start the application in background
echo "🚀 Starting iron-bbs..."
RUST_LOG=info ./target/release/iron-bbs >/tmp/iron-bbs-test.log 2>&1 &
APP_PID=$!
sleep 3

# Function to cleanup on exit
cleanup() {
	echo "🧹 Cleaning up..."
	kill $APP_PID 2>/dev/null || true
	docker-compose down
}
trap cleanup EXIT

# Test 1: Health endpoint
echo "✅ Test 1: Health endpoint"
HEALTH=$(curl -s http://localhost:3000/health)
if [ "$HEALTH" != "OK" ]; then
	echo "❌ Health check failed: $HEALTH"
	exit 1
fi

# Test 2: API posts endpoint
echo "✅ Test 2: API posts endpoint"
POST_COUNT=$(curl -s http://localhost:3000/api/posts | jq -r 'length')
if [ "$POST_COUNT" -lt 1 ]; then
	echo "❌ No posts found"
	exit 1
fi
echo "   Found $POST_COUNT posts"

# Test 3: Web homepage
echo "✅ Test 3: Web homepage"
TITLE=$(curl -s http://localhost:3000/ | grep -o '<title>.*</title>')
if [ -z "$TITLE" ]; then
	echo "❌ Homepage title not found"
	exit 1
fi

# Test 4: SSH port is open
echo "✅ Test 4: SSH server is running"
if ! nc -zv localhost 2222 2>&1 | grep -q "succeeded"; then
	echo "❌ SSH port not accessible"
	exit 1
fi

# Test 5: SSH authentication (should reject)
echo "✅ Test 5: SSH authentication"
if timeout 5 ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o BatchMode=yes localhost -p 2222 echo test 2>&1 | grep -q "Permission denied"; then
	echo "   ✓ Authentication correctly rejects unauthorized connections"
else
	echo "❌ SSH authentication not working as expected"
	exit 1
fi

# Test 6: Create a post via API
echo "✅ Test 6: Create post via API"
NEW_POST=$(curl -s -X POST http://localhost:3000/api/posts \
	-H "Content-Type: application/json" \
	-d '{
        "title": "Test Post",
        "content": "This is a test post created by integration tests",
        "author_id": "550e8400-e29b-41d4-a716-446655440000",
        "published": true
    }')
NEW_POST_ID=$(echo $NEW_POST | jq -r '.id')
if [ -z "$NEW_POST_ID" ] || [ "$NEW_POST_ID" = "null" ]; then
	echo "❌ Failed to create post"
	exit 1
fi
echo "   Created post with ID: $NEW_POST_ID"

# Test 7: Verify post was created
echo "✅ Test 7: Verify post exists"
POST_COUNT_AFTER=$(curl -s http://localhost:3000/api/posts | jq -r 'length')
if [ "$POST_COUNT_AFTER" -le "$POST_COUNT" ]; then
	echo "❌ Post count did not increase"
	exit 1
fi

# Test 8: Check logs for errors
echo "✅ Test 8: Check application logs"
if grep -i "panic\|fatal" /tmp/iron-bbs-test.log; then
	echo "❌ Found critical errors in logs"
	exit 1
fi

echo ""
echo "✅ All tests passed!"
echo "🎉 iron-bbs is working correctly"
