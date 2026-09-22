# ADR 0010: Apache-2.0 with BrainRoot NOTICE attribution

**Status:** Accepted  
**Date:** 2026-09-22

## Context

The project lead requires BrainRoot to be usable for any purpose, including modification, redistribution, and commercial use, provided redistributed work retains the BrainRoot project name and reference. The project also requires a standard OSI-approved license rather than an ambiguous custom license.

## Decision

License BrainRoot under the unmodified Apache License, Version 2.0 (`SPDX-License-Identifier: Apache-2.0`). Include a root `NOTICE` containing the BrainRoot name and canonical reference `https://github.com/AlexandreZanata/brain-root-idea`.

Under Section 4(d), redistributed derivative works that include relevant BrainRoot content must preserve a readable copy of the NOTICE attribution in a NOTICE file, provided documentation/source, or an appropriate generated display. This is attribution preservation, not a requirement to place branding in the primary application UI.

Release artifacts must contain both `LICENSE` and `NOTICE`. Dependency/license tooling must treat Apache-2.0 as the project license and preserve any compatible third-party notices separately.

## Alternatives considered

- **MIT:** permissive and simple, but its notice mechanism is less explicit for the requested project name/reference attribution and lacks Apache's express patent grant.
- **BSD-2-Clause/BSD-3-Clause:** permissive attribution licenses, but Apache's NOTICE mechanism maps more directly to a durable project reference in derivative distributions.
- **Custom attribution license:** rejected because custom wording adds legal ambiguity and could undermine OSI compatibility.
- **Copyleft license:** would impose broader source-sharing conditions than the project lead requested.

## Consequences

Commercial and noncommercial use, modification, and redistribution are permitted under Apache-2.0. Distributors must comply with its license, change-notice, attribution, and NOTICE conditions. The project name may be used for reasonable origin attribution, but the license does not grant trademark endorsement rights.

## Performance implications

None at runtime. Packaging/CI must verify the small LICENSE and NOTICE files are included.

## Security implications

Apache-2.0 includes an express contributor patent grant and termination clause. Licensing does not replace security review, warranty, privacy, or responsible disclosure policy.

## Reversibility

Low for already published contributions and releases: their granted license cannot simply be withdrawn. Future relicensing would require rights analysis and a new ADR. Therefore the canonical license text must remain unmodified.

## References

[Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0), [Apache guidance for applying the license](https://www.apache.org/legal/apply-license.html), and [OSI approved licenses](https://opensource.org/licenses).
