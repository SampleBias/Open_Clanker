#!/usr/bin/env python3
"""
Test script for Open Clanker WebSocket API.
Requires: pip install websockets
Usage: python3 scripts/test_websocket.py [message]
"""
import asyncio
import json
import sys

try:
    import websockets
except ImportError:
    print("Install websockets first. Options:")
    print("  • Use project venv: .venv/bin/python3 scripts/test_websocket.py")
    print("  • Or install: python3 -m venv .venv && .venv/bin/pip install websockets")
    sys.exit(1)


async def test(host="127.0.0.1", port=18789, message="Say hello in one word"):
    uri = f"ws://{host}:{port}/ws"
    print(f"Connecting to {uri}...")
    async with websockets.connect(uri) as ws:
        # Receive welcome
        msg = await ws.recv()
        print(f"Connected. Welcome: {msg[:100]}...")
        # Send message
        payload = {
            "type": "send_message",
            "data": {
                "channel_id": "test",
                "channel_type": "telegram",
                "message": message,
            },
        }
        await ws.send(json.dumps(payload))
        print(f"Sent: {message}")
        # Get AI response
        resp = await asyncio.wait_for(ws.recv(), timeout=60)
        data = json.loads(resp)
        if data.get("type") == "send_response":
            d = data.get("data", {})
            if d.get("success") and d.get("content"):
                print(f"AI reply: {d['content']}")
            else:
                print(f"Error: {d.get('error', 'Unknown')}")
        else:
            print(f"Response: {resp[:300]}")


if __name__ == "__main__":
    msg = sys.argv[1] if len(sys.argv) > 1 else "Say hello in one word"
    asyncio.run(test(message=msg))
