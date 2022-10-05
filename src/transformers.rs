//! Functions and types that transform code into its final form
//!
//! Transformations in order of execution:
//! - Parse: Converts bytes to unlinked instructions
//! - Optimise: Converts unlinked instructions to optimised instructions
//! - Link: Converts optimised instructions to linked instructions

use std::fmt;

use crate::error::Error;

#[derive(Clone, Copy, Debug)]
pub enum Instruction {
	/// >
	Right(usize),
	/// <
	Left(usize),
	/// +
	Add(u8),
	/// -
	Sub(u8),
	/// [
	BranchIfZero(usize),
	/// ]
	BranchIfNotZero(usize),
	/// ,
	Read,
	/// .
	Write,
	/// [-]
	Clear,
}

impl fmt::Display for Instruction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Right(n) => ">".repeat(*n),
				Self::Left(n) => "<".repeat(*n),
				Self::Add(n) => "+".repeat(*n as usize),
				Self::Sub(n) => "-".repeat(*n as usize),
				Self::BranchIfZero(_) => "[".to_owned(),
				Self::BranchIfNotZero(_) => "]".to_owned(),
				Self::Read => ",".to_owned(),
				Self::Write => ".".to_owned(),
				Self::Clear => "[-]".to_owned(),
			}
		)
	}
}

pub struct UnlinkedInstructions(Vec<Instruction>);
pub struct LinkedInstructions(pub Vec<Instruction>);

pub struct Transformer;

impl Transformer {
	pub fn transform(bytes: &[u8]) -> Result<LinkedInstructions, Error> {
		let unlinked_insts = Self::parse(bytes);
		let mut optimised_insts = Self::optimise(&unlinked_insts);

		Self::link(&mut optimised_insts)
	}

	fn parse(bytes: &[u8]) -> UnlinkedInstructions {
		let mut unlinked_insts = Vec::with_capacity(bytes.len());

		for &byte in bytes.iter() {
			unlinked_insts.push(match byte {
				b'>' => Instruction::Right(1),
				b'<' => Instruction::Left(1),
				b'+' => Instruction::Add(1),
				b'-' => Instruction::Sub(1),
				b'[' => Instruction::BranchIfZero(0),
				b']' => Instruction::BranchIfNotZero(0),
				b',' => Instruction::Read,
				b'.' => Instruction::Write,
				_ => continue,
			});
		}

		UnlinkedInstructions(unlinked_insts)
	}

	/// Optimise instructions to be more efficient
	///
	/// Optimisations:
	///  - Combination: repeated sequences of add/sub or left/right
	/// instructions get combined into a single instruction
	///  - Clear pattern: '\[-\]' patterns get combined into a 'clear' instruction
	fn optimise(insts: &UnlinkedInstructions) -> UnlinkedInstructions {
		let mut optimised_insts = Vec::with_capacity(insts.0.len());

		let mut inst_iter = insts.0.iter().peekable();
		while let Some(inst) = inst_iter.next() {
			let optimised_inst = match inst {
				// If there's a right instruction, see if it can be combined
				// with subsequent rights
				Instruction::Right(_) => {
					let mut total = 1;

					while let Some(Instruction::Right(_)) = inst_iter.peek() {
						total += 1;
						inst_iter.next();
					}

					Instruction::Right(total)
				},
				// If there's a left instruction, see if it can be combined
				// with subsequent lefts
				Instruction::Left(_) => {
					let mut total = 1;

					while let Some(Instruction::Left(_)) = inst_iter.peek() {
						total += 1;
						inst_iter.next();
					}

					Instruction::Left(total)
				},
				// If there's an add instruction, see if it can be combined
				// with subsequent adds
				Instruction::Add(_) => {
					let mut total: u8 = 1;

					while let Some(Instruction::Add(_)) = inst_iter.peek() {
						total = total.wrapping_add(1);
						inst_iter.next();
					}

					Instruction::Add(total)
				},
				// If there's a sub instruction, see if it can be combined
				// with subsequent subs
				Instruction::Sub(_) => {
					let mut total: u8 = 1;

					while let Some(Instruction::Sub(_)) = inst_iter.peek() {
						total = total.wrapping_add(1);
						inst_iter.next();
					}

					Instruction::Sub(total)
				},
				// Check if a loop is a clearing loop
				// TODO: make this less ugly
				Instruction::BranchIfZero(_) => {
					if let Some(Instruction::Sub(_)) = inst_iter.peek() {
						inst_iter.next();

						if let Some(Instruction::BranchIfNotZero(_)) = inst_iter.peek() {
							inst_iter.next();

							Instruction::Clear
						} else {
							optimised_insts.push(Instruction::BranchIfZero(0));
							Instruction::Sub(1)
						}
					} else {
						Instruction::BranchIfZero(0)
					}
				},
				inst => inst.to_owned(),
			};

			optimised_insts.push(optimised_inst);
		}

		UnlinkedInstructions(optimised_insts)
	}

	/// Set the jump targets for corresponding [ and ] instructions
	fn link(insts: &mut UnlinkedInstructions) -> Result<LinkedInstructions, Error> {
		let mut jump_stack: Vec<usize> = Vec::with_capacity(5);

		for i in 0..insts.0.len() {
			match insts.0[i] {
				Instruction::BranchIfZero(_) => {
					jump_stack.push(i);
				},
				Instruction::BranchIfNotZero(_) => {
					let opening_idx = match jump_stack.pop() {
						Some(op_idx) => op_idx,
						None => {
							return Err(Error::MissingOpeningBracket(i));
						},
					};

					// The current instruction (]) needs to point to the opening [
					insts.0[i] = Instruction::BranchIfNotZero(opening_idx);
					// The corresponding [ needs to point to the current instruction
					insts.0[opening_idx] = Instruction::BranchIfZero(i);
				},
				_ => (),
			}
		}

		if !(jump_stack.is_empty()) {
			return Err(Error::MissingClosingBracket(jump_stack.pop().unwrap()));
		}

		Ok(LinkedInstructions(insts.0.clone()))
	}
}
