"""End-to-end game-mode check over HTTP + WebSocket.

Needs a running backend and `pip install httpx websockets`.

Plays a full game in team mode and in sandbox mode, answering every question,
then asserts the payloads and the persistence rules.
"""

import asyncio
import json
import os
import sys

import httpx
import websockets

API = os.environ.get("BRAINSTORM_API", "http://127.0.0.1:5000")
WS = os.environ.get("BRAINSTORM_WS", "ws://127.0.0.1:5000/ws")
PASSWORD = "Passw0rd!23456"


async def login(client: httpx.AsyncClient, username: str) -> None:
    response = await client.post(
        f"{API}/api/auth/register",
        json={"username": username, "password": PASSWORD, "email": f"{username}@example.com"},
    )
    if response.status_code not in (200, 201):
        response = await client.post(
            f"{API}/api/auth/login", json={"username": username, "password": PASSWORD}
        )
        response.raise_for_status()
    jar = response.cookies or client.cookies
    cookie = jar.get("brainstorm_session")
    assert cookie, response.text
    client.headers["Cookie"] = f"brainstorm_session={cookie}"


async def me(client: httpx.AsyncClient) -> dict:
    response = await client.get(f"{API}/api/auth/me")
    response.raise_for_status()
    return response.json()["user"]


class Player:
    def __init__(self, name: str, cookie: str, code: str, host: bool, answer_index: int = 0):
        self.name = name
        self.cookie = cookie
        self.code = code
        self.host = host
        self.ws = None
        self.events: list[dict] = []
        self.game_over: dict | None = None
        self.room_joined: dict | None = None
        self.started = False
        self.answer_index = answer_index
        self.time_limits: list[int] = []

    async def run(self) -> None:
        async with websockets.connect(WS, additional_headers={"Cookie": self.cookie}) as ws:
            self.ws = ws
            await self.send("join_room", {"code": self.code})
            async for raw in ws:
                message = json.loads(raw)
                kind, data = message.get("type"), message.get("data") or {}
                self.events.append(message)
                if kind == "room_joined":
                    self.room_joined = data
                    if self.host:
                        await self.send("start_game", {})
                elif kind == "question_emitted":
                    self.time_limits.append(data.get("time_limit"))
                    await asyncio.sleep(0.05)
                    await self.send("submit_answer", {"answer_index": self.answer_index})
                elif kind == "game_over":
                    self.game_over = data
                    return
                elif kind == "error":
                    raise AssertionError(f"{self.name}: server error {data}")

    async def send(self, kind: str, data: dict) -> None:
        await self.ws.send(json.dumps({"type": kind, "data": data}))


async def play(mode: str, timer: int, count: int, answer_index: int = 0) -> tuple[dict, dict, dict]:
    host_client = httpx.AsyncClient(timeout=30)
    guest_client = httpx.AsyncClient(timeout=30)
    try:
        await login(host_client, "host_mode")
        await login(guest_client, "guest_mode")
        before_host = await me(host_client)
        before_guest = await me(guest_client)

        created = await host_client.post(
            f"{API}/api/rooms",
            json={
                "topic": "Режимы",
                "mode": mode,
                "question_count": count,
                "num_options": 3,
                "timer_seconds": timer,
            },
        )
        created.raise_for_status()
        room = created.json()["room"]
        assert room["mode"] == mode, room
        code = room["code"]

        host = Player("host", host_client.headers["Cookie"], code, host=True, answer_index=answer_index)
        guest = Player("guest", guest_client.headers["Cookie"], code, host=False, answer_index=answer_index)
        await asyncio.wait_for(asyncio.gather(host.run(), guest.run()), timeout=120)

        await asyncio.sleep(2)  # stats are written in a spawned task after game_over
        after_host = await me(host_client)
        after_guest = await me(guest_client)
        return {
            "room": room,
            "host_joined": host.room_joined,
            "guest_joined": guest.room_joined,
            "game_over": host.game_over,
            "time_limits": host.time_limits,
            "host_delta": after_host["games_played"] - before_host["games_played"],
            "guest_delta": after_guest["games_played"] - before_guest["games_played"],
        }
    finally:
        await host_client.aclose()
        await guest_client.aclose()


async def main() -> int:
    failures: list[str] = []

    team = await play("team", 12, 5)
    over = team["game_over"]
    print("team game_over:", json.dumps(over, ensure_ascii=False)[:400])
    if over.get("mode") != "team":
        failures.append("team: mode missing in game_over")
    teams = over.get("teams") or []
    if len(teams) != 2:
        failures.append(f"team: expected 2 team totals, got {teams}")
    if any(player.get("team") is None for player in over["scores"]):
        failures.append("team: player without a team seat")
    if team["host_delta"] != 1 or team["guest_delta"] != 1:
        failures.append(f"team: stats not persisted {team['host_delta']}/{team['guest_delta']}")
    if team["host_joined"].get("mode_title") != "Команды":
        failures.append(f"team: mode_title {team['host_joined'].get('mode_title')}")

    lives = await play("lives", 5, 6, answer_index=99)
    eliminated = [p for p in lives["game_over"]["scores"] if p["eliminated"]]
    print("lives game_over:", json.dumps(lives["game_over"], ensure_ascii=False)[:300])
    if not eliminated:
        failures.append("lives: three wrong answers must eliminate a player")

    blitz = await play("blitz", 90, 3)
    if blitz["game_over"].get("mode") != "blitz":
        failures.append("blitz: mode missing in game_over")
    if any(limit != 10 for limit in blitz["time_limits"]):
        failures.append(f"blitz: clock must be forced to 10s, got {blitz['time_limits']}")
    if any(limit != 12 for limit in team["time_limits"]):
        failures.append(f"team: host timer 12s ignored, got {team['time_limits']}")

    sandbox = await play("sandbox", 12, 3)
    if sandbox["game_over"].get("persisted") is not False:
        failures.append("sandbox: game_over should report persisted=false")
    if sandbox["host_delta"] != 0 or sandbox["guest_delta"] != 0:
        failures.append(f"sandbox: stats changed {sandbox['host_delta']}/{sandbox['guest_delta']}")

    print("failures:", failures or "none")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))
