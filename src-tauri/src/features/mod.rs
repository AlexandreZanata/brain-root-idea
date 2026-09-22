//! BrainRoot product features.
//!
//! Each feature owns its state and exposes a narrow interface; the private
//! submodules and `scripts/check-modules.sh` keep the boundary enforceable.

pub mod conversation;
pub mod health;
