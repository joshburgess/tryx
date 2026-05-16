#![no_std]
#![feature(try_trait_v2)]

pub use core::ops::{ControlFlow, FromResidual, Try};

/// Marker for residual types that belong to this workspace's outcome family.
pub trait TryxResidual {}
