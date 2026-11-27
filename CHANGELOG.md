# Changelog

All notable changes to BrainStorm are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- CI jobs for the newer stacks: `rust` (`cargo check --all-targets`, `cargo test`) and
  `frontend` (`npm ci`, `svelte-check`, `npm run build`). Nothing compiled them before.
- README section describing the Flask monolith, the Axum server and the SvelteKit client
  side by side, in both languages.
- Power-ups (`double_points`, `freeze_time`, `remove_two_options`): every player gets a
  three-card hand at game start, spends it over the WebSocket protocol and sees private
  events (their own hand, the two crossed-out options) that other players cannot read.
  Scoring moved to `Player::award`, so the "correct answer = 100, doubled = 200" rule is
  unit-tested instead of living in the socket handler.
- «На выживание» works: every player starts with three lives, a wrong answer costs
  one, the third knocks you out (your answers are refused and the room keeps playing
  without you), and the game ends as soon as nobody is left. The final board marks
  who fell. «Команды» is marked as coming soon instead of silently behaving like
  classic.
- «Играть снова» works: a finished room can start another game in place — scores,
  streaks, lives and hands reset, a new question set is generated, everyone stays
  connected. Only the host can do it, and only from the finished screen; a mid-game
  call is refused so the host cannot wipe a score that is still being played for.
- A room now plays exactly the number of questions the host asked for: providers
  routinely answer with ten regardless of the request, so the count is trimmed before
  the first question goes out (`fit_count`, unit-tested).
- `make ws-check` / `scripts/ws-check.py`: two browsers' worth of protocol against a running
  instance — host-only start, per-player hands, doubling, hidden options, and the fact that
  neither a hand nor another player's crossed-out options ever reach the wrong socket.
  With AI disabled it also asserts the exact scores, since then the answers are known.
- `AGENTS.md`: rules for AI agents (atomic steps, `make agent-verify` before claiming
  success, no `unwrap()`, two-strike stop rule, project map for all three stacks).
- `make agent-verify` / `rust-verify` / `frontend-check`: fmt, `cargo check`, clippy with
  `-D warnings`, tests and the frontend build as one gate; CI runs the same checks.
- Community files: `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `CHANGELOG.md`.

### Changed
- `rust-server`: zero compiler and clippy warnings. `AppError` and `ConfigError` keep their
  heavy payloads boxed (sqlx/figment), dead code removed, `tracing-opentelemetry` dropped.
- `rust-server` reads `config/default.toml` with `include_str!`, so the binary no longer
  silently loses `SECRET_KEY` when started outside `rust-server/`.
- Rate limiting is compiled behind the `rate-limit` feature instead of sitting unused:
  keying by `ConnectInfo` is wrong behind Caddy.

### Fixed
- A page reload no longer costs you the game: the room keeps your seat (score, streak,
  lives, elimination, hand) while it is playing or finished, and the socket replays the
  live question or the final board on reconnect. Players who dropped are shown dimmed
  with an «ОФФЛАЙН» badge and their seats are cleared when the next game starts.
- The room header shows the real mode («На выживание», «FFA» …) instead of always
  «Классика»: `room_joined` carries `mode`.
- The final screen says «Никто не дожил до конца» when survival wipes out every player
  and marks each knocked-out player in the board instead of crowning the last one.
- `./forge.sh verify` printed "локальная матрица пройдена" even when lint failed; it now
  stops at the first failing check.
- `scripts/docs-check.sh` walked `node_modules/`, so a local run reported dozens of broken
  links from third-party packages while CI (clean checkout) stayed green.
- Secret scan gate: two false positives (the RFC 6455 sample WebSocket nonce and a test
  fixture that looked like a key) turned the blocking gitleaks step red on every branch.
  Build output is now allowlisted so a local `gitleaks dir .` is usable too.
- Weak example credentials (`ADMIN_SECRET_KEY=1777`, `CHEAT_TESTER_CODE=19112009`) in
  `.env.example` and `BUILD.md` replaced with placeholders; `1777` failed production
  validation anyway.
- README badges pointed at the upstream mirror and at a `tests.json` that does not exist.
- GitHub issue forms (bug report, feature request) and a pull request template.
- `.editorconfig` and `.gitattributes` for consistent cross-editor / cross-OS behaviour.

## [2.0.0] — 2026-10-07

Full, from-scratch refactor into a production-ready, layered application.

### Added
- Layered architecture: `config` → `domain` → `services` → `realtime` / `http_api`,
  with cross-cutting `security`, `validation`, `errors` and `logging_setup`.
- GigaChat integration with JSON-schema structured output, a stricter system prompt,
  meta-option rejection and a built-in offline question bank as fallback.
- Accounts (XP, levels, coins, cosmetics), UGC with a moderation queue, a 12-level
  single-player campaign, achievements, leaderboard and room history.
- Six game modes (Classic, FFA, Team draft, Lives, Co-op, Svoya Igra), adaptive
  difficulty, jokers, hints and question rephrasing.
- Admin panel and tester/cheat mode with a suite of abilities; `/api/cheat/room_stats`.
- PWA (installable, offline shell), health/readiness probes and structured logging.
- Typed settings with production fail-fast validation; no hardcoded secrets.
- Dockerfile, `docker-compose.yml`, Render blueprint, GitHub Actions CI (lint + tests
  on Python 3.10–3.12 + image build).
- Test suite grown to 190 tests; documentation: bilingual `README.md`, `ROADMAP.md`.

### Fixed
- Security: removed hardcoded secret defaults; constant-time key comparison; rate limits
  on activation/auth/AI endpoints; `X-Forwarded-For` only trusted when `TRUST_PROXY=true`.
- Realtime: restored ~22 missing/misnamed client→server event handlers (chat, cheat panel,
  admin panel, restart, presentation mode); consistent `join_room` / `new_question` payloads.
- Logic: re-entrancy guard (`resolved_q`) prevents a question from being resolved twice
  (timer vs. fast answer vs. cheat skip); timer honours the host setting.
- AI: correct 0-based/1-based answer index handling, deduplication and robust JSON parsing.
- Auth: passwords upgraded to PBKDF2-HMAC-SHA256 with legacy-hash migration.

[Unreleased]: https://github.com/geff1778/Brainstorm-test/compare/v2.0.0...HEAD
[2.0.0]: https://github.com/geff1778/Brainstorm-test/releases/tag/v2.0.0
