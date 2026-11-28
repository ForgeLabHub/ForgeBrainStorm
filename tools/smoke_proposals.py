"""End-to-end smoke test for lobby proposals and the room AI provider.

Talks to the real backend over HTTP + WebSocket. Run with the dev server up:

    python3 tools/smoke_proposals.py [http://localhost:5000]

The first player is the host: it creates the room, accepts the proposal and
starts the game. The second one proposes and votes.
"""

import asyncio
import json
import sys
import time

import requests
import websockets

BASE = (sys.argv[1] if len(sys.argv) > 1 else "http://localhost:5000").rstrip("/")
WS = BASE.replace("https://", "wss://").replace("http://", "ws://") + "/ws"
STAMP = str(int(time.time()))[-6:]
PASSWORD = "proposal-password-1234"


def cookie_header(session):
    return "; ".join(f"{name}={value}" for name, value in session.cookies.items())


def register(username):
    session = requests.Session()
    response = session.post(
        f"{BASE}/api/auth/register",
        json={"username": username, "password": PASSWORD},
        timeout=10,
    )
    assert response.status_code == 201, (response.status_code, response.text)
    # The session cookie is `Secure`, so `requests` refuses to send it over
    # plain http; pin it as a header to keep the local smoke test honest.
    session.headers["Cookie"] = cookie_header(session)
    return session


class Peer:
    def __init__(self, name, session):
        self.name = name
        self.session = session
        self.ws = None
        self.events = []

    async def connect(self):
        self.ws = await websockets.connect(
            WS, additional_headers={"Cookie": self.session.headers["Cookie"]}
        )
        asyncio.create_task(self._pump())

    async def _pump(self):
        try:
            async for raw in self.ws:
                self.events.append(json.loads(raw))
        except Exception:
            pass

    async def send(self, kind, data=None):
        await self.ws.send(json.dumps({"type": kind, "data": data or {}}))

    async def wait_proposals(self, predicate, timeout=30):
        """Wait for a `proposals_list` snapshot that satisfies `predicate`.

        The list is broadcast on join and after every change, so matching on
        content is the only way to tell the snapshots apart.
        """
        deadline = time.time() + timeout
        while time.time() < deadline:
            for event in list(self.events):
                if event.get("type") == "proposals_list":
                    proposals = event["data"]["proposals"]
                    if predicate(proposals):
                        self.events.remove(event)
                        return proposals
            await asyncio.sleep(0.05)
        raise AssertionError(f"{self.name}: no matching proposals_list")

    async def wait_for(self, kind, timeout=30):
        deadline = time.time() + timeout
        while time.time() < deadline:
            for event in list(self.events):
                if event.get("type") == kind:
                    self.events.remove(event)
                    return event.get("data") or {}
            await asyncio.sleep(0.05)
        raise AssertionError(f"{self.name}: no '{kind}' event; got {self.events[-5:]}")


async def main():
    host_session = register(f"prop_host_{STAMP}")
    player_session = register(f"prop_play_{STAMP}")
    host = Peer(f"prop_host_{STAMP}", host_session)
    player = Peer(f"prop_play_{STAMP}", player_session)
    print(f"accounts: {host.name}, {player.name}")

    providers = host_session.get(f"{BASE}/api/ai/providers", timeout=10).json()
    assert providers.get("ok"), providers
    available = providers["providers"]
    print(f"providers: {available or 'none (offline bank)'}")

    # Rooms enforce the same 5..30 range as the proposal validator, so 5 is
    # the smallest round the API will actually accept.
    payload = {"topic": "Космос", "mode": "classic", "question_count": 5}
    if available:
        payload["ai_provider"] = available[0]
    created = host_session.post(f"{BASE}/api/rooms", json=payload, timeout=10)
    assert created.status_code == 201, (created.status_code, created.text)
    code = created.json()["room"]["code"]
    print(f"room: {code} (provider={payload.get('ai_provider', 'auto')})")

    rejected = host_session.post(
        f"{BASE}/api/rooms",
        json={"topic": "Космос", "ai_provider": "no-such-provider"},
        timeout=10,
    )
    assert rejected.status_code == 400, (rejected.status_code, rejected.text)
    print("unknown provider rejected by the API")

    await host.connect()
    await player.connect()
    await host.send("join_room", {"code": code})
    assert (await host.wait_for("room_joined"))["is_host"] is True
    await player.send("join_room", {"code": code})
    await player.wait_for("room_joined")
    print("both joined; lobby proposals panel is live")

    # --- propose ----------------------------------------------------------
    await player.send(
        "propose",
        {"proposal_type": "topic", "value": "История", "description": "Тема → История"},
    )
    proposals = await host.wait_proposals(
        lambda items: any(item["value"] == "История" for item in items)
    )
    proposal = [item for item in proposals if item["value"] == "История"][-1]
    assert proposal["vote_count"] == 1, proposal  # the proposer starts with a vote
    print(f"proposal {proposal['id']}: {proposal['value']} (1 vote)")

    # --- invalid values never reach the list ------------------------------
    await player.send("propose", {"proposal_type": "question_count", "value": "999"})
    error = await player.wait_for("error")
    assert error["code"] == "proposal_invalid", error
    print(f"out-of-range proposal refused: {error['message']}")

    # --- vote / unvote ----------------------------------------------------
    await host.send("vote_proposal", {"proposal_id": proposal["id"]})
    await player.wait_proposals(
        lambda items: any(
            item["id"] == proposal["id"] and item["vote_count"] == 2 for item in items
        )
    )
    print("host vote counted")

    await host.send("unvote_proposal", {"proposal_id": proposal["id"]})
    await player.wait_proposals(
        lambda items: any(
            item["id"] == proposal["id"] and item["vote_count"] == 1 for item in items
        )
    )
    print("host vote withdrawn")

    # The author cannot drop their own vote: the server refuses to remove it,
    # and a refused change broadcasts nothing.
    await asyncio.sleep(0.5)  # let the previous broadcast land before resetting
    player.events = [event for event in player.events if event["type"] != "proposals_list"]
    await player.send("unvote_proposal", {"proposal_id": proposal["id"]})
    await asyncio.sleep(0.6)
    assert not any(event["type"] == "proposals_list" for event in player.events), (
        "author unvote should be a no-op"
    )
    print("author keeps their own vote")

    # --- only the host accepts --------------------------------------------
    await player.send("accept_proposal", {"proposal_id": proposal["id"]})
    error = await player.wait_for("error")
    assert error["code"] == "not_host", error
    print("non-host cannot accept")

    await host.send("accept_proposal", {"proposal_id": proposal["id"]})
    accepted = await player.wait_for("proposal_accepted")
    assert accepted["change"] == {"setting": "topic", "value": "История"}, accepted
    print(f"accepted: {accepted['change']}")
    await player.wait_proposals(
        lambda items: any(
            item["id"] == proposal["id"] and item["active"] is False for item in items
        )
    )
    print("accepted proposal is no longer active")

    # --- the game starts with the accepted topic --------------------------
    await host.send("start_game")
    question = await host.wait_for("question_emitted", timeout=90)
    assert question["question"]["options"], question
    assert question["total"] == 5, question
    print(
        f"question from {question['source']}: "
        f"{question['question']['question'][:60]!r} ({len(question['question']['options'])} options)"
    )

    print("SMOKE OK")


asyncio.run(main())
