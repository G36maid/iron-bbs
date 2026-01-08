#!/bin/bash

set -e

echo "=== Iron BBS Login Flow Test ==="
echo ""
echo "Test 1: Connect as guest (bbs user)"
echo "Expected: See login screen, enter credentials"
echo ""

# Test with known credentials
echo "admin" >/tmp/test_input
echo "admin123" >>/tmp/test_input
echo "quit" >>/tmp/test_input
sleep 1
echo "q" >>/tmp/test_input

timeout 8 ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -p 2222 bbs@localhost </tmp/test_input 2>/dev/null || true

echo ""
echo "=== Test Complete ==="
echo ""
echo "Manual test instructions:"
echo "1. Run: ssh -p 2222 bbs@localhost"
echo "2. Enter username: admin"
echo "3. Enter password: admin123"
echo "4. Should see post browsing interface"
echo "5. Press 'q' to quit"

rm -f /tmp/test_input
