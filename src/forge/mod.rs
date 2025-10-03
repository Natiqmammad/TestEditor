//! Forge Standard Library for ApexForge NightScript
//! 
//! This module provides the standard library implementation for AFNS.

pub mod math;
pub mod collections;
pub mod structs;
pub mod types;
pub mod concurrency;
pub mod os;
pub mod syscall;
pub mod ffi;
pub mod io;
pub mod error;
pub mod memory;
pub mod pointer;
pub mod special;
pub mod flutter;

/// Re-export commonly used items
pub use math::*;
pub use collections::*;
pub use structs::*;
pub use types::*;
pub use concurrency::*;
pub use os::*;
pub use syscall::*;
pub use ffi::*;
pub use io::*;
pub use error::*;
pub use memory::*;
pub use pointer::*;
pub use special::*;
pub use flutter::*;

