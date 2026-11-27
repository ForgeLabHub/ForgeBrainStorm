# Security Policy

## Supported versions

The `main` branch is the only supported version. Fixes are released there.

| Version | Supported |
| ------- | --------- |
| `main`  | ✅        |
| older tags | ❌    |

## Reporting a vulnerability

Please **do not** open a public issue for security problems. Instead, report privately using
GitHub's [private vulnerability reporting](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability)
(Security tab → *Report a vulnerability*) or contact the maintainer directly via
[@geff1778](https://github.com/geff1778).

Please include:

- a description of the issue and its impact,
- steps to reproduce (a proof of concept if possible),
- affected version/commit,
- any suggested mitigation.

We aim to acknowledge reports within 72 hours and to provide a remediation timeline after
triage. Please give us reasonable time to fix the issue before public disclosure.

## Security model & hardening notes

BrainStorm is a self-hosted application. A few design decisions matter for deployments:

- **Secrets** — `SECRET_KEY` (session signing) is the only required secret and has **no default**.
  In `ENV=production` the app refuses to start if it is shorter than 32 bytes or still the
  development value. Keep it in `.env` (git-ignored) or your platform's secret store.
- **Reverse proxy** — `X-Forwarded-For` is only trusted when `TRUST_PROXY=true`. Enable it only
  behind a trusted proxy, otherwise client IPs (and therefore rate limits) are spoofable.
- **Cookies** — set `SESSION_COOKIE_SECURE=true` when serving over HTTPS.
- **CORS** — set `CORS_ORIGINS` to explicit origins in production instead of `*`.
- **Rate limiting** — auth and room-creation endpoints are rate limited in-process. For
  multi-worker deployments back this with Redis (see [ROADMAP.md](ROADMAP.md)).
- **Cheat/admin mode** — no privilege lives in the client: the role is stored in
  `users.role`, `/api/admin/*` sits behind `admin_middleware`, and cheat powers are granted
  and spent by the server. The codes `1778` (cheater) and `19112009` (admin) are
  registration password codes, not separate configuration secrets.
- **Password storage** — Argon2id hashes with a per-password salt.

If you find a gap in any of the above, please report it as described above.
