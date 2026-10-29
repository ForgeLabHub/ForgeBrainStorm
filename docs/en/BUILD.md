# Build: from a clean machine to a running game

English version of [../../BUILD.md](../../BUILD.md).

## Pinned versions

| Tool | Version | Source of truth |
| --- | --- | --- |
| Python | 3.12 | `Dockerfile`, `ci.yml`, `scripts/doctor.sh` |
| Python dependencies | `requirements.lock.txt` (37 packages, exact versions) | `make install` |
| Update ranges | `requirements.txt` | `make lock` |
| Dev tools | `pytest`, `pytest-cov`, `ruff` | `ci.yml` |
| Runtime server | gunicorn + eventlet (`gunicorn.conf.py`) | production run |

## Level A — source code

Neither Docker, nor GigaChat, nor an external database is needed: data lives in SQLite,
AI is off.

```bash
make doctor       # tools, versions, .env not in git, dependencies
make install-dev  # pinned dependencies + pytest/ruff
make lint         # ruff
make test         # 190 tests
make docs-check
```

## Level B — artefacts

A Python application is not compiled: the artefact is the image.

```bash
make build-image  # docker build -t brainstorm:local .
```

## Level C — integration environment

```bash
cp .env.example .env    # minimum: SECRET_KEY; AI is not required
make compose-up
make smoke-test
```

## Level D — container check

CI starts the container and polls `/health` (`ci.yml`, job `docker`). Locally:

```bash
make build-image
docker run -d -p 5000:5000 -e SECRET_KEY=dev-secret-key-that-is-long-enough \
  -e ADMIN_SECRET_KEY=1777 -e CHEAT_TESTER_CODE=19112009 brainstorm:local
./scripts/smoke-test.sh http://localhost:5000
```

## Level E — clean deployment

```bash
docker compose down -v          # deliberate: removes the data volume
make compose-up
make smoke-test
# volume backup and restore — see docs/database/storage.md
```

## Level F — documentation as artefacts

```bash
make docs-pdf     # Markdown -> PDF for both languages, output in docs/pdf/
```

`tools/docs_pdf.py --check` (part of `make docs-check`) fails when a document has no
translation or no PDF, so the bilingual set cannot silently drift.
