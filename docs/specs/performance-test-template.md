# Performance report: <scenario>

**Status:** Planned | Measured | Superseded  
**Date/commit:** <ISO date and revision>  
**Owner:** <owner>  
**Related budget:** <link and metric>

## Claim

State the question and label the result TARGET, MEASURED, or UNKNOWN. Never generalize beyond the tested environment.

## Environment

- BrainRoot build/profile and lockfile:
- OS, version, architecture, patches:
- CPU, RAM, storage, display:
- Power/thermal mode and background workload:
- WebView/runtime versions:
- Project fixture and size:
- Network condition:
- Measurement tools and versions:

## Procedure

Define start/end events, warm/cold conditions, settling time, sampling interval, repetitions, cleanup between runs, and process-tree inclusion. Include exact reproducible commands where safe.

## Raw results

Link or include bounded machine-readable samples. Report sample count, median, p95, minimum, maximum, and variability. Separate Core/UI, WebViews, agent, project processes, language tools, and automation.

## Result against budget

- Budget:
- Measured result:
- Pass / warn / fail / unknown:
- Comparison revision and statistical caveats:

## Resource cleanup

Record state after cancellation/close, orphan process count, memory return, remaining handles/artifacts, and repeated-cycle behavior.

## Interpretation

Explain what the data supports, what it does not support, suspected causes, regressions, and next experiment.

