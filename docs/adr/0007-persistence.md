# ADR 0007: SQLite global state plus minimal project metadata

**Status:** Accepted  
**Date:** 2026-09-22

## Context

BrainRoot must resume projects, tasks, layouts, permissions, and checkpoint references without a local database server or proprietary project format. Secrets and browser identity require different storage.

## Decision

Use one core-owned SQLite database in OS app data for global structured state. Use a small versioned `.brainroot/` directory only for nonsecret project-portable metadata when needed. Store secrets in OS credential facilities, large temporary artifacts in cache with retention, and source in ordinary project files. Decide SQLite crate, migration library, and journal mode during implementation; do not assume WAL by default.

## Alternatives considered

- **JSON files only:** readable but weak transactions, migration, and concurrent structured queries.
- **Postgres/Redis/local service:** rejected as unnecessary idle and operational cost.
- **Everything in `.brainroot/`:** rejected due secrets, churn, device-specific state, and lock-in.
- **Everything global:** rejected because selected metadata should travel with a project.

## Consequences

Schema migrations, backups, corruption recovery, artifact retention, and export/delete tools become responsibilities. Data classification must be explicit.

## Performance implications

SQLite supports incremental queries and avoids parsing a state pile at startup. Keep one controlled access layer; measure query/startup cost. WAL is enabled only with evidence because it adds lifecycle files/checkpoint behavior.

## Security implications

SQLite is not secret storage. Minimize and redact histories, restrict file permissions, and store only credential references. Treat imported/newer databases as untrusted input.

## Reversibility

Moderate. Versioned repository interfaces and exportable schemas permit migration. Project source remains independent.

## References

[SQLite application file format](https://www.sqlite.org/appfileformat.html), [appropriate uses](https://www.sqlite.org/whentouse.html), [WAL](https://www.sqlite.org/wal.html).

