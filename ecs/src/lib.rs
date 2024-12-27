#![feature(allocator_api)]
#![no_std]

#[cfg(any(test, feature = "std"))]
extern crate std;

extern crate alloc;

pub mod archetype;
pub mod bundle;
pub mod entity;
pub mod query;
pub mod world;
