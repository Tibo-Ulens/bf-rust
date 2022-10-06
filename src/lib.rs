#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate thiserror;

pub mod error;
pub mod instruction;
pub mod interpret;
pub mod link;
pub mod optimise;
