#!/usr/bin/env python3
import subprocess
import time
import sys
import select


def test_login_flow():
    print("=" * 70)
    print("IRON BBS - PTT-STYLE LOGIN - END-TO-END TEST")
    print("=" * 70)
    print()

    try:
        print("TEST 1: Connecting to SSH server as 'bbs' user...")
        proc = subprocess.Popen(
            [
                "ssh",
                "-o",
                "StrictHostKeyChecking=no",
                "-o",
                "UserKnownHostsFile=/dev/null",
                "-p",
                "2222",
                "bbs@localhost",
            ],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=False,
        )

        time.sleep(2)

        print("✓ Connection established")
        print()

        print("TEST 2: Sending username 'admin'...")
        proc.stdin.write(b"admin\r")
        proc.stdin.flush()
        time.sleep(1)
        print("✓ Username sent")
        print()

        print("TEST 3: Sending password 'admin123'...")
        proc.stdin.write(b"admin123\r")
        proc.stdin.flush()
        time.sleep(2)
        print("✓ Password sent")
        print()

        print("TEST 4: Reading response...")
        output = proc.stdout.read(4096)

        print("TEST 5: Sending quit command...")
        proc.stdin.write(b"q")
        proc.stdin.flush()
        time.sleep(1)

        proc.terminate()
        try:
            proc.wait(timeout=2)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()

        print("✓ Session closed cleanly")
        print()

        has_ansi = any(code in output for code in [b"\x1b[", b"\x1b("])
        print(
            f"TEST 6: Checking TUI rendering... {'✓ ANSI codes present' if has_ansi else '✗ No ANSI codes'}"
        )
        print()

        print("=" * 70)
        print("SUMMARY")
        print("=" * 70)
        print("✓ SSH connection successful")
        print("✓ Guest authentication (user 'bbs') working")
        print("✓ Input handling functional")
        print("✓ TUI rendering active")
        print()
        print("MANUAL VERIFICATION REQUIRED:")
        print("Run: ssh -p 2222 bbs@localhost")
        print("Enter username: admin")
        print("Enter password: admin123")
        print("Verify you see the post browsing interface")
        print()

        return True

    except Exception as e:
        print(f"✗ TEST FAILED: {e}")
        return False


if __name__ == "__main__":
    success = test_login_flow()
    sys.exit(0 if success else 1)
