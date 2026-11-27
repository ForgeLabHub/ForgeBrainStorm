"""End-to-end smoke test for chat, emoji and the extended cheater powers.

Talks to the real backend over HTTP + WebSocket, so it exercises the same paths
the Svelte client uses. Run with the dev server up:

    python3 tools/smoke_chat_cheat.py [http://localhost:5000]
"""

import asyncio
import json
import sys
import time

import requests
import websockets

BASE = (sys.argv[1] if len(sys.argv) > 1 else "http://localhost:5000").rstrip("/")
WS = BASE.replace("https://", "wss://").replace("http://", "ws://") + "/ws"
CHEAT_CODE = "1778"
ADMIN_CODE = "19112009"
STAMP = str(int(time.time()))[-6:]


def register(username, password):
    session = requests.Session()
    response = session.post(
        f"{BASE}/api/auth/register",
        json={"username": username, "password": password},
        timeout=10,
    )
    assert response.status_code == 201, (response.status_code, response.text)
    # The session cookie is `Secure`, so `requests` refuses to send it over
    # plain http; pin it as a header to keep the local smoke test honest.
    session.headers["Cookie"] = cookie_header(session)
    return session


def create_room(session, topic="Космос"):
    response = session.post(
        f"{BASE}/api/rooms",
        json={"topic": topic, "mode": "classic", "question_count": 3, "timer_secs": 60},
        timeout=10,
    )
    assert response.status_code in (200, 201), (response.status_code, response.text)
    body = response.json()
    return body.get("code") or body.get("room", {}).get("code")


def admin_call(session, path, payload=None):
    url = f"{BASE}/api/admin{path}"
    if payload is None:
        response = session.get(url, timeout=10)
    else:
        response = session.post(url, json=payload, timeout=10)
    return response


def cookie_header(session):
    return "; ".join(f"{name}={value}" for name, value in session.cookies.items())


class Peer:
    def __init__(self, name, session):
        self.name = name
        self.session = session
        self.ws = None
        self.events = []
        self.powers = {}

    async def connect(self):
        self.ws = await websockets.connect(
            WS, additional_headers={"Cookie": cookie_header(self.session)}
        )
        asyncio.create_task(self._pump())

    async def _pump(self):
        try:
            async for raw in self.ws:
                event = json.loads(raw)
                if event.get("type") == "cheat_state":
                    self.powers = event["data"]["powers"]
                self.events.append(event)
        except Exception:
            pass

    async def send(self, kind, data=None):
        await self.ws.send(json.dumps({"type": kind, "data": data or {}}))

    async def wait_score(self, minimum, timeout=15):
        """Wait until this player's score reaches `minimum`."""
        deadline = time.time() + timeout
        while time.time() < deadline:
            for event in list(self.events):
                if event.get("type") != "players_updated":
                    continue
                for player in event["data"]["players"]:
                    if player["name"] == self.name and player["score"] >= minimum:
                        return player["score"]
            await asyncio.sleep(0.05)
        raise AssertionError(f"{self.name}: score never reached {minimum}")

    async def wait_for(self, kind, timeout=15):
        deadline = time.time() + timeout
        while time.time() < deadline:
            for event in self.events:
                if event.get("type") == kind:
                    self.events.remove(event)
                    return event.get("data") or {}
            await asyncio.sleep(0.05)
        raise AssertionError(f"{self.name}: no '{kind}' event; got {self.events[-5:]}")


async def main():
    cheater = Peer(f"cheat_{STAMP}", register(f"cheat_{STAMP}", CHEAT_CODE))
    victim = Peer(f"player_{STAMP}", register(f"player_{STAMP}", "player-password-1234"))
    print(f"accounts: {cheater.name} (cheater), {victim.name}")

    code = create_room(cheater.session)
    print(f"room: {code}")

    await cheater.connect()
    await victim.connect()
    await cheater.send("join_room", {"code": code})
    joined = await cheater.wait_for("room_joined")
    assert joined["is_host"] is True
    await victim.send("join_room", {"code": code})
    await victim.wait_for("room_joined")
    print("both joined")

    await cheater.send("start_game")
    question = await cheater.wait_for("question_emitted", timeout=60)
    await victim.wait_for("question_emitted", timeout=60)
    print(f"question: {question['question']['question'][:50]!r}")

    # --- peek, then put points on the table ------------------------------
    # Leech and swap need somebody with a score, and the victim is the only
    # other player, so this comes before the curses that zero their answers.
    await cheater.wait_for("cheat_state")
    await cheater.send("cheat_power", {"power": "peek"})
    reveal = await cheater.wait_for("cheat_reveal")
    await victim.send("submit_answer", {"answer_index": reveal["correct_index"]})
    score = await victim.wait_score(1)
    print(f"peek ok: correct index {reveal['correct_index']}, victim at {score} points")

    # --- leech: 25 from every other player -------------------------------
    await cheater.send("cheat_power", {"power": "leech"})
    leeched = await cheater.wait_for("cheat_used")
    assert leeched["power"] == "leech" and leeched["points"] > 0, leeched
    print(f"leech ok: {leeched['points']} points from {leeched['victims']} players")

    # --- swap: trade places with the leader ------------------------------
    await cheater.send("cheat_power", {"power": "swap"})
    swapped = await cheater.wait_for("cheat_used")
    assert swapped["power"] == "swap", swapped
    assert swapped["to"] > swapped["from"], swapped
    print(f"swap ok: {swapped['from']} -> {swapped['to']} points")

    # --- chat -------------------------------------------------------------
    await victim.send("chat_send", {"text": "  привет   всем  "})
    chat = await cheater.wait_for("chat_message")
    assert chat["text"] == "привет всем", chat
    assert chat["from"] == victim.name, chat
    print(f"chat ok: {chat['from']}: {chat['text']}")

    # flood control: two messages back to back
    await victim.send("chat_send", {"text": "ещё"})
    error = await victim.wait_for("error")
    assert error["code"] == "chat_too_fast", error
    print("flood control ok")

    # --- emoji ------------------------------------------------------------
    await asyncio.sleep(0.5)  # the 400 ms gap between two messages is per player
    await victim.send("emoji", {"emoji": "🔥", "target": cheater.name})
    throw = await cheater.wait_for("emoji")
    assert throw["emoji"] == "🔥", throw
    print(f"emoji ok: {throw['from']} -> {throw['target']}")

    await asyncio.sleep(0.5)
    await victim.send("emoji", {"emoji": "💥"})
    error = await victim.wait_for("error")
    assert error["code"] == "emoji_not_allowed", error
    print("emoji palette enforced")

    # --- cheater powers ---------------------------------------------------
    await cheater.send("cheat_power", {"power": "mute", "target": victim.name})
    muted = await victim.wait_for("muted")
    assert muted["by"] == cheater.name, muted
    print("mute ok")

    await victim.send("chat_send", {"text": "мне можно?"})
    error = await victim.wait_for("error")
    assert error["code"] == "chat_muted", error
    print("muted player is silenced")

    # a repeated mute must not burn a second use
    await cheater.wait_for("cheat_state")
    before = cheater.powers["mute"]
    await cheater.send("cheat_power", {"power": "mute", "target": victim.name})
    error = await cheater.wait_for("error")
    assert error["code"] == "cheat_no_effect", error
    await cheater.wait_for("cheat_state")
    assert cheater.powers["mute"] == before, (before, cheater.powers)
    print("no-op power refunded")

    await cheater.send("cheat_power", {"power": "jinx", "target": victim.name})
    await victim.wait_for("jinxed")
    print("jinx ok")

    await cheater.send(
        "cheat_power",
        {
            "power": "rewrite",
            "payload": {
                "question": "Кто здесь читер?",
                "options": ["Я", "Ты", "Он"],
                "correct_index": 0,
            },
        },
    )
    rewritten = await victim.wait_for("question_emitted")
    assert rewritten["question"]["question"] == "Кто здесь читер?", rewritten
    print("rewrite ok")

    # --- blind: the victim cannot answer while it lasts -------------------
    await cheater.send("cheat_power", {"power": "blind", "target": victim.name})
    blinded = await victim.wait_for("blinded")
    assert blinded["seconds"] >= 1, blinded
    await victim.send("submit_answer", {"answer_index": 0})
    error = await victim.wait_for("error")
    assert error["code"] == "blinded", error
    print(f"blind ok: {blinded['seconds']} s, answer refused")

    # --- hint: two wrong options disappear for the cheater only -----------
    await cheater.send("cheat_power", {"power": "hint"})
    hidden = await cheater.wait_for("options_hidden")
    assert len(hidden["indexes"]) == 2, hidden
    print(f"hint ok: options {hidden['indexes']} hidden")

    # a mistyped victim must not burn the budget
    await cheater.wait_for("cheat_state")
    before = cheater.powers["hijack"]
    await cheater.send("cheat_power", {"power": "hijack", "target": "нет-такого"})
    error = await cheater.wait_for("error")
    assert error["code"] == "cheat_no_target", error
    print("unknown victim rejected")

    await cheater.send("cheat_power", {"power": "hijack", "target": victim.name})
    hijacked = await victim.wait_for("hijacked")
    assert hijacked["by"] == cheater.name, hijacked
    await asyncio.sleep(0.3)
    after = cheater.powers["hijack"]
    assert before == 1 and after == 0, (before, after)
    print(f"hijack ok: {hijacked['points']} points taken")

    # --- admin panel ------------------------------------------------------
    admin = register(f"admin_{STAMP}", ADMIN_CODE)
    detail = admin_call(admin, f"/rooms/detail?code={code}")
    assert detail.status_code == 200, (detail.status_code, detail.text)
    detail = detail.json()
    assert detail["code"] == code and len(detail["players"]) >= 1, detail
    print(f"admin room detail ok: {len(detail['players'])} players, {detail['state']}")

    announce = admin_call(admin, "/rooms/announce", {"code": code, "message": "тест объявления"})
    assert announce.status_code == 200, announce.text
    await cheater.wait_for("announcement")
    print("admin room announcement ok")

    locked = admin_call(admin, "/rooms/chat_lock", {"code": code, "locked": True})
    assert locked.status_code == 200, locked.text
    await asyncio.sleep(0.3)
    await cheater.send("chat_send", {"text": "а теперь?"})
    error = await cheater.wait_for("error")
    assert error["code"] == "chat_locked", error
    unlocked = admin_call(admin, "/rooms/chat_lock", {"code": code, "locked": False})
    assert unlocked.status_code == 200, unlocked.text
    print("chat lock ok: message refused while frozen")

    cleared = admin_call(admin, "/rooms/chat_clear", {"code": code})
    assert cleared.status_code == 200, cleared.text
    await cheater.wait_for("chat_cleared")
    print("chat clear ok")

    kick = admin_call(admin, "/rooms/kick", {"code": code, "username": victim.name})
    assert kick.status_code == 200, kick.text
    await victim.wait_for("kicked")
    detail = admin_call(admin, f"/rooms/detail?code={code}").json()
    assert all(player["name"] != victim.name for player in detail["players"]), detail
    print("kick ok: player removed from the room")

    # --- ban: the ban must reach an already-open session ------------------
    ban = admin_call(admin, "/ban", {"identifier": victim.name, "reason": "smoke test"})
    assert ban.status_code == 200, ban.text
    assert ban.json()["sessions_revoked"] >= 1, ban.json()
    blocked = victim.session.get(f"{BASE}/api/profile/games", timeout=10)
    assert blocked.status_code in (401, 403), (blocked.status_code, blocked.text)
    print("ban ok: existing session rejected")

    unban = admin_call(admin, "/unban", {"identifier": victim.name})
    assert unban.status_code == 200, unban.text
    print("unban ok")

    print("\nALL CHECKS PASSED")


if __name__ == "__main__":
    asyncio.run(main())
