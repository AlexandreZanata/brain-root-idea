// The provider-neutral contract and the deterministic fake are consumed by the
// execution layer and the integration test in the following B02 microsteps
// (S03-S06); until those consumers exist the items are intentionally unused.
// Remove these allowances when the executor lands.
#[allow(dead_code)]
pub mod contract;
#[allow(dead_code)]
pub mod fake;
