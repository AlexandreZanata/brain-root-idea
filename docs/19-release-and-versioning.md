# Experimental release and versioning policy

## Decision

BrainRoot uses Semantic Versioning 2.0.0 and remains below `1.0.0` throughout the experiment. The informal range “0.0001–0.01” is represented professionally as `0.0.1` prereleases through the `0.0.x` line, with `0.1.0` reserved for a substantially broader experimental product milestone.

Versions such as `0.01` or `0.0001` are not valid SemVer: normal versions have exactly `X.Y.Z`, and numeric components do not have leading zeroes. Early iterations therefore use forms such as `0.0.1-alpha.1`.

## Initial sequence

```text
v0.0.1-alpha.1  governance and delivery controls
v0.0.1-alpha.2  minimal measured Linux shell
v0.0.1-alpha.3  provider contract and deterministic fake
v0.0.1-alpha.4  OpenCode Go live transport
v0.0.1-alpha.5  minimal Linux conversation experience
v0.0.1          Linux MVP-0 experimental test release
```

These tags describe planned batch outcomes, not promises that an arbitrary commit deserves the version. If a batch is split, continue the prerelease sequence rather than renumbering published tags.

## Version meaning before 1.0

- `0.0.1-alpha.N`: incomplete, internal/public experimental batch artifact; compatibility can change.
- `0.0.1`: first reproducible Linux MVP-0 test foundation; still experimental and incomplete.
- `0.0.PATCH`: compatible fixes, security patches, packaging corrections, or small hardening within the MVP-0 contract.
- `0.MINOR.0`: a materially larger experimental capability contract. `0.1.0` is reserved for the complete Linux product loop or an explicitly accepted equivalent milestone.
- `1.0.0`: forbidden during the current experiment. It requires a future ADR defining stable public contracts, supported platforms, migration policy, security posture, and maintenance commitment.

Breaking changes before 1.0 are allowed only when documented in `CHANGELOG.md`, the release notes, Wiki history, and any relevant migration instructions. “Experimental” does not excuse silent data loss or secret exposure.

## When a version changes

A microstep issue never changes the version. A batch changes the version exactly once, in its finalization issue, after all implementation issues close and before final CI. Emergency fixes after a published release use a dedicated hotfix batch and the next valid patch or prerelease.

Do not publish two different artifacts with the same version. Tags and release artifacts are immutable. A failed release candidate gets a new prerelease identifier; never move a published tag.

## Single source of truth

B00 chooses one machine-readable root version source. Generated or ecosystem-required copies—for example Cargo package, frontend package, and Tauri bundle versions—must be synchronized by a deterministic script/check. Agents change the root source only; final CI fails on drift.

Implemented in B00-S05: the root `VERSION` file is the single machine-readable source. It contains `UNRELEASED` until a batch finalization sets a SemVer value (B00-S06 sets `0.0.1-alpha.1`). `CHANGELOG.md` is the human-readable record, and `scripts/check-version-consistency.sh` fails on an invalid or multi-line source, a missing changelog section, or a mismatch in any present `Cargo.toml`, `package.json`, or `src-tauri/tauri.conf.json`. Agents change only `VERSION`; ad-hoc version fields remain forbidden.

## Tags and releases

- Git tags use `v<semver>`, for example `v0.0.1-alpha.4`.
- Tags are annotated; signing is preferred once maintainer signing is configured.
- Tag only the batch merge commit on `main`, never an unmerged branch head.
- Prerelease tags produce GitHub pre-releases, not stable releases.
- Release title: `BrainRoot <version> — <capability>`.
- Include supported Linux environment, artifact checksum, dependency/runtime requirements, privacy warning, known limitations, related batch PR/issues/Wiki page, and performance status labels.

## Changelog

Use a human-readable `CHANGELOG.md` with `Unreleased` and one section per published version. Allowed categories are Added, Changed, Fixed, Security, Performance, Documentation, Removed, and Known limitations. Write user outcomes, not commit messages. Link the batch PR and Wiki release page.

The project history remains more detailed than the changelog. Do not paste raw logs or secrets into either.

## Artifact identity

Every build report records version, Git commit, dirty/clean status, build profile, target triple, Linux distribution/reference environment, WebKit/runtime version, and dependency lockfile hash. Local dirty builds display a development suffix in diagnostics but are never tagged or uploaded as official artifacts.

## Release gate

Before tagging:

- From B17 onward, this versioned gate is when automated unit, contract, integration, E2E, soak, and full CI tests execute; microstep issues only wrote the tests and ran fast non-test checks (ADR 0014);

- all batch issues closed with evidence;
- latest batch head passed required full Linux CI;
- review approved and conversations resolved;
- version copies and lockfiles consistent;
- changelog, repository history, Wiki batch/release pages synchronized;
- license/security/dependency checks passed;
- packaged `LICENSE` and `NOTICE` preserve Apache-2.0 and the BrainRoot attribution/reference;
- artifact produced from clean source and checksum recorded;
- install/run/close/uninstall or removal path verified on the supported environment;
- known limitations and external provider variability stated clearly.

## Reference

[Semantic Versioning 2.0.0](https://semver.org/) defines `X.Y.Z`, major-zero initial development, and prerelease identifiers.
