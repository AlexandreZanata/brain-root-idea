// The provider-neutral contract is consumed by the deterministic fake and the
// execution layer in the following B02 microsteps (S02-S06); until those
// consumers exist the items are intentionally unused. Remove this allowance
// when the fake provider lands.
#[allow(dead_code)]
pub mod contract;
