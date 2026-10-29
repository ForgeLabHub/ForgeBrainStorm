# Module specification: template

English version of [../../modules/module-template.md](../../modules/module-template.md).

Copy it to `docs/modules/<name>.md` and fill in all 11 sections — `make docs-check` verifies
them in CI.

## 1. Purpose and boundaries
What it does, what it does not do, where its responsibility ends.

## 2. Use scenarios
User or system → what happens → what is observable, including failures.

## 3. Contract
HTTP routes, Socket.IO events, response schemas, error codes.

## 4. Data
Which SQLite files or tables the module writes (it is the only writer) and what it reads.

## 5. Access rules
Who has access; what happens on refusal; how it is verified.

## 6. Dependencies
Incoming (who calls it) and outgoing (whom it calls); the port to an external system.

## 7. Configuration
Environment variables with defaults and the consequences of a wrong value.

## 8. Build and debug
How to run just this module and how to reproduce the behaviour with a test.

## 9. Tests
Files and test names: what exactly is proven.

## 10. Observability
Logs, events, error codes, operator actions on failure.

## 11. Limitations and readiness
Known limitations, what was deliberately left out, readiness criteria.
