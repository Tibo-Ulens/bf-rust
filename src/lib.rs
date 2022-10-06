#[macro_use]
extern crate thiserror;

pub mod error;
pub mod interpret;
pub mod link;
pub mod optimise;
pub mod transpile;

use transpile::Instruction;

pub struct UnlinkedInstructions(Vec<Instruction>);
pub struct LinkedInstructions(Vec<Instruction>);
