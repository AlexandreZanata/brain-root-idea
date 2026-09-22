// The provider-neutral contract, the deterministic fake, the bounded executor,
// and the normalizer are consumed by the credential boundary and the
// integration test in the following B02 microsteps (S05-S06); until those
// consumers exist the items are intentionally unused. Remove these allowances
// when the integration lands.
#[allow(dead_code)]
pub mod contract;
#[allow(dead_code)]
pub mod execution;
#[allow(dead_code)]
pub mod fake;
#[allow(dead_code)]
pub mod normalize;
