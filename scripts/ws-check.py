#!/usr/bin/env python3
"""End-to-end check of the gameplay WebSocket protocol with two players.

Speaks the same protocol as the browser client against a running server, so it
catches breakage that unit tests cannot see: a hand that is never dealt, an
event addressed to the wrong player, a wrong-answer path that never scores.

    pip install websocket-client
    python3 scripts/ws-check.py [--base http://127.0.0.1:5000]

With AI providers configured the questions are generated, so their answers are
unknown to this script: score assertions are then skipped and only the protocol
plumbing is checked. With AI disabled the questions come from the offline bank,
whose wording it knows, so exact scores are asserted.
"""

import argparse
import http.cookiejar
import json
import sys
import time
import urllib.error
import urllib.request

import websocket

# Fallback bank wording -> correct option text. Order-proof: options may move.
KNOWN_ANSWERS = {
    "Сколько планет в Солнечной системе?": "8",
    "Какая планета ближе всего к Солнцу?": "Меркурий",
    "Столица России?": "Москва",
}

BASE = "http://127.0.0.1:5000"
FAILURES: list[str] = []


class Client:
    def __init__(self, name, base):
        self.name = name
        self.ws = websocket.create_connection(
            base.replace("http://", "ws://").replace("https://", "wss://") + "/ws",
            header=[f"Cookie: {self.session()}"],
            timeout=120,
        )
        self.msgs = []

    def session(self):
        jar = http.cookiejar.CookieJar()
        opener = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(jar))
        creds = {"username": self.name, "password": "supersecret123"}
        body = json.dumps(creds).encode()
        for path in ("/api/auth/register", "/api/auth/login"):
            request = urllib.request.Request(
                BASE + path, data=body, headers={"content-type": "application/json"}
            )
            try:
                opener.open(request, timeout=30).close()
                break
            except urllib.error.HTTPError:
                if path == "/api/auth/login":
                    raise
        # The session cookie is `Secure`, and urllib will not resend it over
        # plain http — which is exactly what this local check uses.
        return "; ".join(f"{c.name}={c.value}" for c in jar)

    def send(self, kind, data):
        self.ws.send(json.dumps({"type": kind, "data": data}))

    def wait(self, kind, timeout=30):
        for msg in list(self.msgs):
            if msg["type"] == kind:
                self.msgs.remove(msg)
                return msg
        end = time.time() + timeout
        while time.time() < end:
            self.ws.settimeout(max(1, end - time.time()))
            try:
                msg = json.loads(self.ws.recv())
            except websocket.WebSocketTimeoutException:
                break
            if msg["type"] == kind:
                return msg
            self.msgs.append(msg)
        raise AssertionError(
            f"{self.name}: no {kind!r}; buffered={[m['type'] for m in self.msgs]}"
        )

    def silence(self, kind, seconds=3):
        """Drain the socket for a while; True when `kind` never shows up."""
        end = time.time() + seconds
        while time.time() < end:
            self.ws.settimeout(max(0.2, end - time.time()))
            try:
                msg = json.loads(self.ws.recv())
            except websocket.WebSocketTimeoutException:
                break
            if msg["type"] == kind:
                return False
            self.msgs.append(msg)
        return True

    def drain(self):
        self.msgs.clear()


def api(path, payload, cookie):
    request = urllib.request.Request(
        BASE + path,
        data=json.dumps(payload).encode(),
        headers={"content-type": "application/json", "cookie": cookie},
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.load(response)


def check(label, condition, detail=""):
    print(f"  [{'ok  ' if condition else 'FAIL'}] {label}" + (f" — {detail}" if detail else ""))
    if not condition:
        FAILURES.append(label)


def scores_of(client, timeout=30):
    try:
        players = client.wait("players_updated", timeout=timeout)["data"]["players"]
    except AssertionError:
        return None
    return {p["name"]: p["score"] for p in players}


def score_reaches(client, expected, timeout=30):
    """Wait for a player's score to reach `expected`.

    A snapshot from the start of the round is always in flight, so the first
    `players_updated` after an answer can still carry the old score.
    """
    end = time.time() + timeout
    seen = []
    while time.time() < end:
        scores = scores_of(client, timeout=max(1, end - time.time()))
        if scores is None:
            break
        seen.append(scores.get(client.name))
        if scores.get(client.name) == expected:
            return True, seen
    return False, seen


def answer_index(question):
    text = KNOWN_ANSWERS.get(question["question"])
    if text and text in question["options"]:
        return question["options"].index(text)
    return None


def main():
    global BASE
    parser = argparse.ArgumentParser()
    parser.add_argument("--base", default=BASE)
    args = parser.parse_args()
    BASE = args.base.rstrip("/")

    host = Client("wscheck", BASE)
    room = api(
        "/api/rooms",
        {
            "topic": "Общие знания",
            "question_count": 5,
            "options": 4,
            "timer_seconds": 20,
            "difficulty": "easy",
            "mode": "classic",
        },
        cookie=host.session(),
    )
    code = room["room"]["code"]
    print(f"room {code}")

    host.send("join_room", {"code": code})
    host.wait("room_joined")
    check("hand is empty before the game", host.wait("power_ups")["data"]["items"] == [])

    guest = Client("wscheck2", BASE)
    guest.send("join_room", {"code": code})
    check(
        "guest joins as a non-host",
        guest.wait("room_joined")["data"]["is_host"] is False,
    )
    check(
        "guest gets no hand before the game",
        guest.wait("power_ups")["data"]["items"] == [],
    )
    players, deadline = [], time.time() + 15
    while time.time() < deadline:
        players = host.wait("players_updated")["data"]["players"]
        if len(players) == 2:
            break
    check(
        "host sees both players",
        sorted(p["name"] for p in players) == ["wscheck", "wscheck2"],
        players,
    )

    # ── Only the host may start ──
    guest.send("start_game", {})
    error = guest.wait("error")["data"]["message"]
    check("guest cannot start the game", "хост" in error.lower(), error)

    host.send("start_game", {})
    host.wait("game_preparing")
    hand = host.wait("power_ups")["data"]["items"]
    check(
        "a full hand is dealt at start",
        sorted(hand) == ["double_points", "freeze_time", "remove_two_options"],
        hand,
    )
    check(
        "guest gets its own hand",
        sorted(guest.wait("power_ups")["data"]["items"])
        == ["double_points", "freeze_time", "remove_two_options"],
    )

    # ── 1. DoublePoints, with the hand kept per player ──
    first = host.wait("question_emitted", timeout=60)["data"]
    question, source = first["question"], first.get("source", "")
    guest.wait("question_emitted")
    host.drain()
    host.send("use_power_up", {"power_up": "double_points"})
    used = host.wait("power_up_used")["data"]
    check(
        "double_points is announced",
        used["player"] == "wscheck" and used["power_up"] == "double_points",
        used,
    )
    check("double_points left the hand", "double_points" not in host.wait("power_ups")["data"]["items"])
    # Hands are private: the guest sees the announcement, never the host's deck.
    check("the guest sees the announcement", guest.wait("power_up_used")["data"]["player"] == "wscheck")
    check("the host's hand never reaches the guest", guest.silence("power_ups", seconds=2))

    index = answer_index(question)
    if index is None:
        print(f"  [skip] score assertions: answer unknown (source={source!r})")
    else:
        host.send("submit_answer", {"answer_index": index})
        result = host.wait("question_result")["data"]
        reported = result.get("correct_index", result.get("correctIndex"))
        check("answer accepted as correct", reported == index, f"sent={index} got={reported}")
        ok, seen = score_reaches(host, 200)
        check("correct answer doubled to 200", ok, f"seen={seen}")
    host.drain()

    # ── 2. RemoveTwoOptions is visible only to the player who paid ──
    second = host.wait("question_emitted", timeout=60)["data"]["question"]
    guest.wait("question_emitted")
    host.drain()
    host.send("use_power_up", {"power_up": "remove_two_options"})
    hidden = host.wait("options_hidden")["data"]["indexes"]
    check("exactly two options hidden", len(hidden) == 2 and len(set(hidden)) == 2, hidden)
    check(
        "hidden indexes are inside the option list",
        all(0 <= i < len(second["options"]) for i in hidden),
        hidden,
    )
    check("the guest never sees the hidden options", guest.silence("options_hidden", seconds=3))
    index = answer_index(second)
    if index is not None:
        check("the correct option is not hidden", index not in hidden, (index, hidden))
        host.send("submit_answer", {"answer_index": index})
        ok, seen = score_reaches(host, 300)
        check("a normal correct answer adds 100", ok, f"seen={seen}")
    host.drain()

    # ── 3. FreezeTime, and the hand empties ──
    host.wait("question_emitted", timeout=60)
    host.drain()
    host.send("use_power_up", {"power_up": "freeze_time"})
    used = host.wait("power_up_used")["data"]
    check("freeze adds 10 seconds", used["seconds"] == 10, used)
    check("hand is empty after three uses", host.wait("power_ups")["data"]["items"] == [])

    # ── 4. Spending an empty hand ──
    host.send("use_power_up", {"power_up": "freeze_time"})
    error = host.wait("error")["data"]["message"]
    check("spending an empty hand is refused", "нет в запасе" in error, error)

    if FAILURES:
        print(f"ws-check: {len(FAILURES)} failure(s): {FAILURES}")
        return 1
    print("ws-check: all checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
