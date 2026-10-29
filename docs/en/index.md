# BrainStorm documentation

English version. The canonical (Russian) set is [../index.md](../index.md); both are kept
in step and both ship as PDF in [../pdf/](../pdf/).

| Question | Document |
| --- | --- |
| Run it from scratch | [BUILD.md](BUILD.md) (Russian: [../../BUILD.md](../../BUILD.md)) |
| Understand the design | [architecture/system-context.md](architecture/system-context.md) |
| Where the data lives | [database/storage.md](database/storage.md) |
| API and errors | [api/conventions.md](api/conventions.md) |
| What a module does / what to borrow | [modules/](modules/) |
| Deployment | [operations/deployment.md](operations/deployment.md) |
| Monitoring | [operations/monitoring.md](operations/monitoring.md) |
| **Secrets and the key incident** | [security/secrets.md](security/secrets.md) |
| What proves it works | [verification/matrix.md](verification/matrix.md), CI |
| What the audit changed | [audit/portability-audit-2026-10-29.md](audit/portability-audit-2026-10-29.md) |
| Contributing, security policy | [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md) |

## Rules

1. A claim without a check is a promise — see [verification/matrix.md](verification/matrix.md).
2. A new module without a specification following
   [modules/module-template.md](modules/module-template.md) is not considered done
   (`make docs-check`).
3. Secrets come from the environment only; `gitleaks` and `make doctor` enforce it.
4. Dependency versions are pinned in `requirements.lock.txt`; `make lock` is a deliberate
   action, not automation.

## Development

- [development/code-style.md](development/code-style.md) — lint, tool versions, style debt.

## Project control

- `forge.sh` / `forge.bat` start, stop, tail logs, run tests, manage the database and
  containers in one command; see the README section.
