# Changelog

All notable changes to BrainRoot are recorded here. The format is human-readable and uses the categories from `docs/19-release-and-versioning.md`. Versions follow Semantic Versioning 2.0.0 and stay below `1.0.0` during the experiment.

Batch B00 (governance and delivery controls) is tracked by [pull/2](https://github.com/AlexandreZanata/brain-root-idea/pull/2) and the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B00-Governance).

## Unreleased

No unreleased changes yet.

## 0.0.1-alpha.1

### Added

- Repository governance for the experimental delivery workflow: verified Apache-2.0 `LICENSE` and BrainRoot `NOTICE` with an offline artifact check ([#1](https://github.com/AlexandreZanata/brain-root-idea/issues/1)), a microstep issue form and batch pull request template with a structural check ([#3](https://github.com/AlexandreZanata/brain-root-idea/issues/3)), a repository rules runbook with `main` protection ([#4](https://github.com/AlexandreZanata/brain-root-idea/issues/4)), and the project history and Wiki structure ([#5](https://github.com/AlexandreZanata/brain-root-idea/issues/5)).
- A single machine-readable version source (`VERSION`) with a deterministic consistency check (`scripts/check-version-consistency.sh`).

### Documentation

- Batch B00 repository history mirror and the Wiki operations pages, including the flat page-naming convention.

### Known limitations

- No application code, CI, provider integration, or user-visible capability exists yet; the repository is the specification for MVP-0.
