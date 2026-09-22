# Persistence and project state

## Principles

Persist only what improves resume, portability, recovery, or preferences. Separate project-portable state from device-global state and secrets. Use versioned schemas and transactional migrations. A project must remain useful when `.brainroot/` is removed.

## Data placement

| Class | Location | Examples | Rules |
|---|---|---|---|
| Project-portable | `<project>/.brainroot/` | project ID, nonsecret preferences, task/checkpoint metadata references, layout hints | small, documented, merge-friendly where practical; no secrets/cookies/raw credentials |
| Global structured | OS app-data SQLite | recent projects, task/session index, window state, provider metadata, grants, migration state | transactional; one core-owned connection layer |
| Sensitive | OS credential store | API tokens, refresh tokens, vault unlock material | store references only in SQLite; redact from logs/exports |
| Temporary | OS cache/temp | screenshots, traces, bounded logs, extracted artifacts | TTL/size policy; safe cleanup; not required for correctness |
| Project content | normal workspace files/Git | source, tests, configuration | owned by user; never made proprietary |

## Proposed `.brainroot/` v1

```text
.brainroot/
  project.json        # schema version, stable project ID, nonsecret settings
  tasks/              # optional high-level portable task records
  checkpoints/        # metadata only; content backend is separate
```

Do not create this directory merely on folder inspection; create it after explicit BrainRoot project initialization or the first durable feature that needs it. Generated/ephemeral data belongs in global app data or cache. Define recommended `.gitignore` entries after checkpoint design is proven.

## SQLite decision

SQLite fits local structured state without a server and supports atomic transactions. The core owns schema and migrations. Start with the default journal mode unless measured concurrency justifies WAL; WAL introduces `-wal`/`-shm` files and checkpoint behavior. Never open a global database from multiple ad-hoc frontend connections.

Sources: [SQLite application file format](https://www.sqlite.org/appfileformat.html), [threading modes](https://sqlite.org/threadsafe.html), and [WAL tradeoffs](https://www.sqlite.org/wal.html).

## Session and event persistence

Persist user messages, normalized high-level task events, decisions, artifact references, checkpoint IDs, final state, adapter/provider identity, and whether native resume is supported. Large stdout/stderr, screenshots, browser traces, and provider-private state use bounded artifacts with expiry. Do not store hidden chain-of-thought.

## Checkpoint storage contract

A checkpoint records baseline identity, included paths, created/modified/deleted files, preexisting user changes, content backend, task, timestamp, and integrity hash. Git may provide object storage for Git projects, but checkpoints are not user commits and cannot rewrite public history. Git's low-level commands are explicitly intended to support alternative porcelain, but the safe object/ref design still needs a spike. Non-Git folders require a bounded content-addressed fallback. Exact mechanics remain open. See [Git command documentation](https://git-scm.com/docs/git).

## Browser state

Store only reconstruction metadata by default: destination, allowed origin, display mode, viewport, and safe navigation hints. Human cookies and local storage remain in a separate platform WebView profile and are never serialized into project metadata. Agent automation profiles are ephemeral unless a user explicitly opts into a scoped test profile.

## Migrations and recovery

- Every persistent record has a schema version.
- Migrations are forward, transactional, backed up, and tested from every supported version.
- Unknown newer project metadata is not overwritten.
- Database corruption offers safe backup/export and rebuild of derivable indexes.
- Temporary artifacts have documented retention and a user-visible clear action.
