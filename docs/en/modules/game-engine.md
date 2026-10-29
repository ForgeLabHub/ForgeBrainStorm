# Module: game engine

English version of [../../modules/game-engine.md](../../modules/game-engine.md).

## 1. Purpose and boundaries

Game rules: round generation, answer checking, scoring, modes (classic, learning,
campaign). A domain layer without Flask and network.
It does not send events to clients (that is `realtime`) and does not store the leaderboard
(that is `services.leaderboard`).

## 2. Use scenarios

- Round start: a set of questions from the bank, a time limit.
- Player answer: checking, scoring, recording the result.
- Round end: totals are computed and handed to the leaderboard.
- Failure: not enough questions → a round configuration error, not an empty round.

## 3. Contract

Internal functions of `app/game_logic.py` and `app/domain/*`; the outside world sees
Socket.IO events and `/api/*` responses. Error codes — `app/errors.py`.

## 4. Data

Reads the question bank (built-in/offline or AI-generated), writes the round result through
the leaderboard service. Has no tables of its own.

## 5. Access rules

Game actions are available anonymously; the cheat mode (`/api/cheat`) requires
`CHEAT_TESTER_CODE` and is checked separately (`tests/test_privileged.py`).

## 6. Dependencies

Outgoing: `app/domain`, `app/ai_client` (question generation, optional). Incoming:
`app/http_api/rooms.py`, `app/realtime/socket_events.py`.

## 7. Configuration

`CHEAT_TESTER_CODE` (access to the test mode). Question count and round time come from the
room data.

## 8. Build and debug

```bash
make test
python -m pytest tests/test_domain.py tests/test_services.py -q
```

## 9. Tests

`tests/test_domain.py` (rules and scoring), `tests/test_services.py`,
`tests/test_validation.py` (input boundaries), `tests/test_privileged.py` (access to
privileged actions).

## 10. Observability

Round and generation error logs; room state is visible through Socket.IO events.
Errors: `invalid_answer`, `round_not_ready`, `privileged_required`.

## 11. Limitations and readiness

- Rules live in one process: several replicas will not share room state.
- Round state is not persisted: a restart interrupts the game.
- Readiness: the rules are covered by unit tests; the privileged mode is closed by a code.
