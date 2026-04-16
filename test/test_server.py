import asyncio
import json
import os
import pprint
import websockets

# Bypass any corporate/system HTTP proxy for local connections
os.environ.pop("HTTP_PROXY", None)
os.environ.pop("http_proxy", None)
os.environ.pop("HTTPS_PROXY", None)
os.environ.pop("https_proxy", None)

GAME_URL = "ws://127.0.0.1:3000/ws"


async def send_and_receive(ws, message):
    """Send a JSON message and collect the responses (direct + broadcast state)."""
    await ws.send(json.dumps(message))
    responses = []
    # Read responses until we get a 'state' broadcast
    while True:
        raw = await asyncio.wait_for(ws.recv(), timeout=5.0)
        data = json.loads(raw)
        responses.append(data)
        if data.get("type") == "state":
            break
    return responses


async def send_only(ws, message):
    """Send a JSON message and return the first response."""
    await ws.send(json.dumps(message))
    raw = await asyncio.wait_for(ws.recv(), timeout=5.0)
    return json.loads(raw)


async def recv_until_state(ws):
    """Read messages from the websocket until we get a 'state' message."""
    while True:
        raw = await asyncio.wait_for(ws.recv(), timeout=5.0)
        data = json.loads(raw)
        if data.get("type") == "state":
            return data


async def small_test():
    async with websockets.connect(GAME_URL) as ws:
        # Join
        print("\n--- Joining ---")
        responses = await send_and_receive(ws, {"action": "join", "name": "Jakob"})
        join_result = next(r for r in responses if r.get("type") == "join_result")
        token = join_result["token"]
        print(f"name: Jakob, token: {token}")

        # Get state
        print("\n--- State ---")
        responses = await send_and_receive(ws, {"action": "get_state"})
        state = next(r for r in responses if r.get("type") == "state")
        pprint.pprint(state)

        # Start round
        print("\n--- Starting Round ---")
        responses = await send_and_receive(ws, {"action": "start"})
        for r in responses:
            print(r)

        # Buzz
        print("\n--- Buzzing ---")
        responses = await send_and_receive(ws, {"action": "buzz", "token": token})
        for r in responses:
            print(r)

        # Get state
        print("\n--- State After Buzz ---")
        responses = await send_and_receive(ws, {"action": "get_state"})
        state = next(r for r in responses if r.get("type") == "state")
        pprint.pprint(state)

        # Leave
        print("\n--- Leaving ---")
        responses = await send_and_receive(ws, {"action": "leave", "token": token})
        for r in responses:
            print(r)

        # Rejoin with token
        print("\n--- Rejoining with token ---")
        responses = await send_and_receive(ws, {"action": "join", "name": "Jakob", "token": token})
        for r in responses:
            print(r)

        # Final state
        print("\n--- Final State ---")
        responses = await send_and_receive(ws, {"action": "get_state"})
        state = next(r for r in responses if r.get("type") == "state")
        pprint.pprint(state)


async def concurrent_buzz_test():
    n_players = 10
    tokens = []

    print("\n--- Joining Players ---")
    for i in range(n_players):
        async with websockets.connect(GAME_URL) as ws:
            responses = await send_and_receive(ws, {"action": "join", "name": f"Player{i+1}"})
            join_result = next(r for r in responses if r.get("type") == "join_result")
            token = join_result["token"]
            tokens.append(token)
            print(f"Player{i+1} joined with token {token}")

    print("\n--- Starting Round ---")
    async with websockets.connect(GAME_URL) as ws:
        responses = await send_and_receive(ws, {"action": "start"})
        for r in responses:
            print(r)

    await asyncio.sleep(0.5)

    print("\n--- Buzzing Concurrently ---")

    async def buzz_player(token, name):
        async with websockets.connect(GAME_URL) as ws:
            responses = await send_and_receive(ws, {"action": "buzz", "token": token})
            buzz_result = next(r for r in responses if r.get("type") == "buzz_result")
            print(f"Buzz {name}: success={buzz_result['success']}")
            return buzz_result

    tasks = []
    for i, token in enumerate(tokens):
        tasks.append(buzz_player(token, f"Player{i+1}"))

    results = await asyncio.gather(*tasks)

    print("\nBuzz results:")
    pprint.pprint(results)

    print("\n--- Final Game State ---")
    async with websockets.connect(GAME_URL) as ws:
        responses = await send_and_receive(ws, {"action": "get_state"})
        state = next(r for r in responses if r.get("type") == "state")
        pprint.pprint(state)


if __name__ == "__main__":
    asyncio.run(concurrent_buzz_test())