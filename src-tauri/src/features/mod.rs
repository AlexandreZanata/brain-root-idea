//! BrainRoot product features.
//!
//! Each feature owns its state and exposes a narrow interface; the private
//! submodules and `scripts/check-modules.sh` keep the boundary enforceable.

pub mod agent_host;
pub mod canvas_host;
pub mod conversation;
pub mod deck;
pub mod governor;
pub mod health;
pub mod human_browser;
pub mod preview;
