// The provider-neutral contract, the deterministic fake, the bounded executor,
// the normalizer, and the credential boundary are consumed by the integration
// test in the following B02 microstep (S06); until that consumer exists the
// items are intentionally unused. Remove these allowances when the integration
// lands.
#[allow(dead_code)]
pub mod contract;
#[allow(dead_code)]
pub mod credential;
#[allow(dead_code)]
pub mod execution;
#[allow(dead_code)]
pub mod fake;
#[allow(dead_code)]
pub mod normalize;

#[cfg(test)]
mod integration;
