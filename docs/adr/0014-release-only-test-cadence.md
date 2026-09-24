# ADR 0014: Automated tests run at the versioned release gate

**Status:** Accepted for the experiment

**Date:** 2026-09-24

## Context

Running Rust, frontend, browser, E2E, soak, and full CI tests at every small issue consumes minutes and interrupts the economical-agent workflow. BrainRoot already uses one draft PR per batch, a version change at batch end, and required latest-head CI before merge. The maintainer requests automated tests only when preparing a release/version, not on every microstep.

## Decision

From B17 onward, implementation microsteps add or update the tests that cover their behavior but do not execute automated unit, contract, integration, E2E, soak, or full CI tests. Their local micro-gate is restricted to fast non-test checks: allowlisted diff, whitespace/format, secret scan, and applicable schema/documentation validation. An issue closes as `IMPLEMENTED_UNVERIFIED`; its deferred test list appears in the issue and PR.

At the batch's single version/release boundary, the draft PR becomes Ready and required CI runs the complete suite on the latest head. Platform/manual probes and performance measurements that CI cannot cover also belong to this release gate. A failed gate creates a remediation issue and another final-head run. The submitting task never waits for CI; a later task checks once. No merge, tag, or release occurs without required latest-head CI, review, and documented evidence.

## Alternatives considered

- Run the full suite after every microstep: earlier failures, rejected for repeated minutes of latency and duplicated execution.
- Run targeted automated tests on every microstep: catches more defects early, rejected for this experiment because the maintainer explicitly prioritizes short microstep turns.
- Eliminate tests entirely: rejected; it would make browser/security/resource claims untrustworthy.

## Consequences

Defects can surface late and may require reopening completed issues or creating remediation issues. Closing an implementation issue no longer means its runtime behavior has been verified. PR/Wiki wording must distinguish `IMPLEMENTED_UNVERIFIED` from release-verified. Test code must be committed alongside each capability so the release gate remains complete.

## Performance implications

Shorter individual agent turns and fewer repeated full-suite runs are expected, not yet measured. Benchmark the saved CI time and any increased release rework before claiming a net productivity improvement. The product's runtime performance budgets do not change.

## Security implications

No security boundary is relaxed. Privileged and browser changes remain unverified until the release gate and must not be merged or advertised as safe before it passes. Fast secret scanning and scope review remain per microstep. A release gate failure cannot be waived merely for speed.

## Reversibility

High. Reinstate targeted per-issue tests if late failures increase total lead time or risk, without changing product architecture. Historical B00–B16 issue evidence remains as recorded.

## References

- [BrainRoot execution plan](../18-mvp-execution-plan.md)
- [BrainRoot testing strategy](../13-testing-strategy.md)
- [GitHub protected branch checks](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)
