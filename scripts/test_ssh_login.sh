#!/bin/bash

echo "Testing SSH connection to Iron BBS..."
echo ""
echo "Attempting to connect: ssh -p 2222 bbs@localhost"
echo "Expected: Login screen should appear"
echo ""
echo "Press Ctrl+C to exit after testing"
echo ""

ssh-keygen -R "[localhost]:2222" 2>/dev/null
ssh -o StrictHostKeyChecking=no -p 2222 bbs@localhost
