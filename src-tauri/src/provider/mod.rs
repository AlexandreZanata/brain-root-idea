// The provider-neutral contract, the deterministic fake, and the bounded
// executor are consumed by the normalizer and the integration test in the
// following B02 microsteps (S04-S06); until those consumers exist the items are
// intentionally unused. Remove these allowances when the integration lands.
#[allow(dead_code)]
pub mod contract;
#[allow(dead_code)]
pub mod execution;
#[allow(dead_code)]
pub mod fake;
