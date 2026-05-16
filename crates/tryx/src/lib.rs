//! Nightly outcome types for Rust's `?` operator.
//!
//! `tryx` re-exports feature-gated outcome crates from one user-facing entry
//! point.

#[cfg(feature = "cancel")]
pub use tryx_cancel as cancel;

#[cfg(feature = "checked")]
pub use tryx_checked as checked;

#[cfg(feature = "derive")]
pub use tryx_derive as derive;

#[cfg(feature = "parsing")]
pub use tryx_parsing as parsing;

#[cfg(feature = "stage")]
pub use tryx_stage as stage;
