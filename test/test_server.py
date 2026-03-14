import requests
import json
import concurrent.futures as futures
import pprint
import time

GAME_URL = "http://localhost:3000"


def join_game(username: str):
    # Join without a token initially
    player_req = {"name": username}
    res = requests.post(f"{GAME_URL}/join", json=player_req)
    data = res.json()
    print(f"Join response: {data}")
    # Return the token for future actions
    return data["token"]

def join_game_token(username: str, token):
    # Join without a token initially
    player_req = {"name": username, "token": token}
    res = requests.post(f"{GAME_URL}/join", json=player_req)
    data = res.json()
    print(f"Join response: {data}")
    # Return the token for future actions
    return data["token"]

def start_game():
    res = requests.post(f"{GAME_URL}/start")
    print(f"Start response: {res.status_code}")


def buzz(player_token: str, username: str):
    player_req = {"token": player_token}
    res = requests.post(f"{GAME_URL}/buzz", json=player_req)
    print(f"Buzz {username}: {res.status_code}")
    return res.status_code


def get_state():
    res = requests.get(f"{GAME_URL}/state")
    return res.json()

def leave_game(player_token: str):
    player_req = {"token": player_token}
    res = requests.post(f"{GAME_URL}/leave", json=player_req)
    print(f"Leave {res.status_code}")

def small_test():
    player = "Jakob"
    token = join_game(player)
    print(f"name: {player} , token: {token}")
    state = get_state()
    pprint.pprint(state)
    start_game()
    buzz(token, player)
    state = get_state()
    pprint.pprint(state)
    leave_game(token)
    pprint.pprint(state)
    token = join_game_token(player, token)
    state = get_state()
    pprint.pprint(state)


def main():
    n_players = 10
    players = []  # List of dicts: { "name": ..., "token": ... }

    print("\n--- Joining Players ---")
    with futures.ThreadPoolExecutor() as executor:
        tasks = []
        for i in range(n_players):
            username = f"Player{i+1}"
            tasks.append(executor.submit(join_game, username))

        for t in tasks:
            token, name = t.result()
            players.append({"name": name, "token": token})

    print("\n--- Starting Round ---")
    start_game()

    time.sleep(0.5)  # small delay

    print("\n--- Buzzing Concurrently ---")
    with futures.ThreadPoolExecutor() as executor:
        tasks = []
        for p in players:
            tasks.append(executor.submit(buzz, p["token"], p["name"]))

        results = [t.result() for t in tasks]

    print("\nBuzz results:")
    pprint.pprint(results)

    print("\n--- Final Game State ---")
    state = get_state()
    pprint.pprint(state)


if __name__ == "__main__":
    small_test()