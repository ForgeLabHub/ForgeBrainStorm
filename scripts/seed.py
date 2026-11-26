"""Create demo users and play a few scripted games.

Usage: API=http://127.0.0.1:5000 python3 scripts/seed.py
"""
import asyncio
import json
import os

import requests
import websockets

API = os.environ.get("API", "http://127.0.0.1:5000")
PASSWORD = os.environ.get("SEED_PASSWORD", "supersecret123")
USERS = ["uitester", "wshost", "anna", "maxim", "kirill"]
GAMES = [
    (["uitester", "anna", "maxim"], "История России"),
    (["wshost", "kirill", "anna"], "Космос и планеты"),
    (["uitester", "wshost"], "Кино и сериалы"),
]


def login(username: str) -> str:
    requests.post(f"{API}/api/auth/register", json={"username": username, "password": PASSWORD}, timeout=15)
    session = requests.Session()
    session.post(f"{API}/api/auth/login", json={"username": username, "password": PASSWORD}, timeout=15)
    cookie = session.cookies.get("brainstorm_session")
    if not cookie:
        raise RuntimeError(f"cannot log in as {username}")
    return cookie


async def play(players: list[str], cookies: dict[str, str], topic: str) -> str:
    room = requests.post(
        f"{API}/api/rooms",
        json={"topic": topic, "question_count": 5, "timer_secs": 10, "num_options": 4},
        cookies={"brainstorm_session": cookies[players[0]]},
        timeout=20,
    ).json()["room"]["code"]

    sockets = []
    for player in players:
        ws = await websockets.connect(f"ws://127.0.0.1:5000/ws",
                                      additional_headers={"Cookie": f"brainstorm_session={cookies[player]}"})
        await ws.send(json.dumps({"type": "join_room", "data": {"code": room}}))
        sockets.append(ws)
    await asyncio.sleep(0.8)
    await sockets[0].send(json.dumps({"type": "start_game", "data": {}}))

    asked = 0
    while True:
        message = json.loads(await asyncio.wait_for(sockets[0].recv(), timeout=90))
        if message["type"] == "question_emitted":
            asked += 1
            for index, ws in enumerate(sockets):
                await ws.send(json.dumps({"type": "submit_answer",
                                          "data": {"answer_index": (asked + index) % 4}}))
        if message["type"] == "game_over" or asked >= 5:
            break
    for ws in sockets:
        await ws.close()
    await asyncio.sleep(1.5)
    return room


async def main() -> None:
    cookies = {user: login(user) for user in USERS}
    for players, topic in GAMES:
        code = await play(players, cookies, topic)
        print(f"seeded game {code}: {topic} ({', '.join(players)})")


asyncio.run(main())
