//! NOTE: current implementation assume that the [crdt::ListCrdt::OpUnit] is cheap to clone
//!
//!
//!
pub mod crdt;
mod dumb_common;
pub mod fugue;
pub mod rga;
pub mod woot;
pub mod yata;

#[cfg(feature = "fuzzing")]
pub mod fugue_dumb_impl;
#[cfg(feature = "fuzzing")]
pub mod rga_dumb_impl;
#[cfg(feature = "fuzzing")]
pub mod test;
#[cfg(feature = "fuzzing")]
pub mod woot_dumb_impl;
#[cfg(feature = "fuzzing")]
pub mod yata_dumb_impl;
