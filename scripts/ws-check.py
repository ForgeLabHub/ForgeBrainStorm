#!/usr/bin/env python3
"""End-to-end check of the gameplay WebSocket protocol.

Speaks the same protocol as the browser client against a running server, so it
catches breakage that unit tests cannot see: a hand that is never dealt, an
event addressed to the wrong player, a wrong-answer path that never scores.

    pip install websocket-client
    python3 scripts/ws-check.py [--base http://127.0.0.1:5000]

With AI providers configured the questions are generated, so their answers are
unknown to this script: the score assertions are then skipped and only the
protocol plumbing is checked. With AI disabled the questions come from the
offline bank still, but the wording there changes over time, so the script
matches the answer by option *text* and skips when the text is unknown.
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

JAR = http.cookiejar.CookieJar()
OPENER = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(JAR))
COOKIE: list[str] = []
BASE = "http://127.0.0.1:5000"
FAILURES: list[str] = []


class Client:
    def __init__(self, cookie, ws_url):
        self.ws = websocket.create_connection(
            ws_url, header=[f"Cookie: {cookie}"], timeout=120
        )
        self.msgs = []

    def send(self, kind, data):
        self.ws.send(json.dumps({"type": kind, "data": data}))

    def wait(self, kind, timeout=120):
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
        raise AssertionError(f"no {kind!r}; buffered={[m['type'] for m in self.msgs]}")

    def drain(self):
        self.msgs.clear()


def api(path, payload=None):
    body = json.dumps(payload).encode() if payload is not None else None
    headers = {"content-type": "application/json"}
    if COOKIE:
        headers["cookie"] = COOKIE[0]
    request = urllib.request.Request(BASE + path, data=body, headers=headers)
    with OPENER.open(request, timeout=30) as response:
        data = json.load(response)
    # The session cookie is `Secure`, and urllib will not resend it over plain
    # http — which is exactly what this local check uses.
    COOKIE.clear()
    COOKIE.append("; ".join(f"{c.name}={c.value}" for c in JAR))
    return data


def check(label, condition, detail=""):
    print(f"  [{'ok  ' if condition else 'FAIL'}] {label}" + (f" — {detail}" if detail else ""))
    if not condition:
        FAILURES.append(label)


def score_reaches(client, expected, timeout=30):
    """Wait for the score to reach `expected`.

    A snapshot from the start of the round is always in flight, so the first
    `players_updated` after an answer can still carry the old score.
    """
    end = time.time() + timeout
    seen = []
    while time.time() < end:
        try:
            players = client.wait("players_updated", timeout=max(1, end - time.time()))["data"]["players"]
        except AssertionError:
            break
        seen.append(players[0]["score"])
        if players[0]["score"] == expected:
            return True, seen
    return False, seen


def main():
    global BASE
    parser = argparse.ArgumentParser()
    parser.add_argument("--base", default=BASE)
    parser.add_argument("--user", default="wscheck")
    parser.add_argument("--password", default="supersecret123")
    args = parser.parse_args()

    BASE = args.base.rstrip("/")
    ws_url = BASE.replace("http://", "ws://").replace("https://", "wss://") + "/ws"

    creds = {"username": args.user, "password": args.password}
    try:
        api("/api/auth/register", creds)
    except urllib.error.HTTPError:
        api("/api/auth/login", creds)

    room = api(
        "/api/rooms",
        {
            "topic": "Общие знания",
            "questions": 5,
            "options": 4,
            "time": 20,
            "difficulty": "easy",
            "mode": "classic",
        },
    )
    code = room["room"]["code"]
    print(f"room {code}")

    client = Client(COOKIE[0], ws_url)
    client.send("join_room", {"code": code})
    client.wait("room_joined")
    hand = client.wait("power_ups")["data"]["items"]
    check("hand is empty before the game", hand == [], hand)

    client.send("start_game", {})
    client.wait("game_preparing")
    hand = client.wait("power_ups")["data"]["items"]
    check(
        "a full hand is dealt at start",
        sorted(hand) == ["double_points", "freeze_time", "remove_two_options"],
        hand,
    )

    # ── 1. DoublePoints ──
    first = client.wait("question_emitted", timeout=180)["data"]
    question, source = first["question"], first.get("source", "")
    client.drain()
    client.send("use_power_up", {"power_up": "double_points"})
    used = client.wait("power_up_used")["data"]
    check("double_points is announced", used["power_up"] == "double_points", used)
    hand = client.wait("power_ups")["data"]["items"]
    check("double_points left the hand", "double_points" not in hand, hand)

    answer_text = KNOWN_ANSWERS.get(question["question"])
    if answer_text and answer_text in question["options"]:
        index = question["options"].index(answer_text)
        client.send("submit_answer", {"answer_index": index})
        result = client.wait("question_result")["data"]
        reported = result.get("correct_index", result.get("correctIndex"))
        check("answer accepted as correct", reported == index, f"sent={index} got={reported}")
        ok, seen = score_reaches(client, 200)
        check("correct answer doubled to 200", ok, f"seen={seen}")
    else:
        print(f"  [skip] score assertions: answer unknown (source={source!r})")
    client.drain()

    # ── 2. RemoveTwoOptions ──
    second = client.wait("question_emitted", timeout=60)["data"]
    question = second["question"]
    client.drain()
    client.send("use_power_up", {"power_up": "remove_two_options"})
    hidden = client.wait("options_hidden")["data"]["indexes"]
    check("exactly two options hidden", len(hidden) == 2 and len(set(hidden)) == 2, hidden)
    check(
        "hidden indexes are inside the option list",
        all(0 <= i < len(question["options"]) for i in hidden),
        hidden,
    )
    answer_text = KNOWN_ANSWERS.get(question["question"])
    if answer_text and answer_text in question["options"]:
        index = question["options"].index(answer_text)
        check("the correct option is not hidden", index not in hidden, (index, hidden))
        client.send("submit_answer", {"answer_index": index})
        ok, seen = score_reaches(client, 300)
        check("a normal correct answer adds 100", ok, f"seen={seen}")
    client.drain()

    # ── 3. FreezeTime ──
    client.wait("question_emitted", timeout=60)
    client.drain()
    client.send("use_power_up", {"power_up": "freeze_time"})
    used = client.wait("power_up_used")["data"]
    check("freeze adds 10 seconds", used["seconds"] == 10, used)
    hand = client.wait("power_ups")["data"]["items"]
    check("hand is empty after three uses", hand == [], hand)

    # ── 4. Spending an empty hand ──
    client.send("use_power_up", {"power_up": "freeze_time"})
    error = client.wait("error")["data"]["message"]
    check("spending an empty hand is refused", "нет в запасе" in error, error)

    if FAILURES:
        print(f"ws-check: {len(FAILURES)} failure(s): {FAILURES}")
        return 1
    print("ws-check: all checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
