use crate::error::Error;
use crate::instruction::{Instruction, LinkedInstructions, UnlinkedInstructions};

bitflags! {
	pub struct Optimisations: u8 {
		const COMBINE_CLEARS     = 0b00000001;
		const GROUP_INSTRUCTIONS = 0b00000010;
	}
}

impl Optimisations {
	pub fn from_strings(strs: &[String]) -> Self {
		let mut opts = Self::empty();

		for s in strs {
			match s.as_str() {
				"all" => opts.set(Self::all(), true),
				"combine-clears" => opts.set(Self::COMBINE_CLEARS, true),
				"group-instructions" => opts.set(Self::GROUP_INSTRUCTIONS, true),
				_ => (),
			}
		}

		opts
	}
}

impl UnlinkedInstructions {
	/// Apply all requested optimisations
	///
	/// List of optimisations:
	///  - Clear pattern combination: `[-]` patterns get combined into a `clear`
	/// instruction
	///  - Instruction grouping: repeated sequences of add/sub and left/right
	/// instructions get combined into a single instruction
	pub fn optimise(self, opts: Optimisations) -> Result<LinkedInstructions, Error> {
		let mut optimised_insts = self.link()?;
		if opts.contains(Optimisations::COMBINE_CLEARS) {
			optimised_insts = optimised_insts.combine_clears().link()?;
		}
		if opts.contains(Optimisations::GROUP_INSTRUCTIONS) {
			optimised_insts = optimised_insts.group_instructions().link()?;
		}

		Ok(optimised_insts)
	}
}

impl LinkedInstructions {
	/// Combine `[-]` into a `clear` instruction
	fn combine_clears(&self) -> UnlinkedInstructions {
		let mut optimised_insts = Vec::with_capacity(self.0.len());

		let mut inst_iter = self.0.iter().peekable();
		while let Some(inst) = inst_iter.next() {
			let optimised_instruction = match inst {
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

			optimised_insts.push(optimised_instruction);
		}

		UnlinkedInstructions(optimised_insts)
	}

	/// Group repeated sequences of add/sub and left/right instructions
	fn group_instructions(&self) -> UnlinkedInstructions {
		let mut optimised_insts = Vec::with_capacity(self.0.len());

		let mut inst_iter = self.0.iter().peekable();
		while let Some(inst) = inst_iter.next() {
			let optimised_inst = match inst {
				Instruction::Right(_) => {
					let mut total = 1;

					while let Some(Instruction::Right(_)) = inst_iter.peek() {
						total += 1;
						inst_iter.next();
					}

					Instruction::Right(total)
				},
				Instruction::Left(_) => {
					let mut total = 1;

					while let Some(Instruction::Left(_)) = inst_iter.peek() {
						total += 1;
						inst_iter.next();
					}

					Instruction::Left(total)
				},
				Instruction::Add(_) => {
					let mut total: u8 = 1;

					while let Some(Instruction::Add(_)) = inst_iter.peek() {
						total = total.wrapping_add(1);
						inst_iter.next();
					}

					Instruction::Add(total)
				},
				Instruction::Sub(_) => {
					let mut total: u8 = 1;

					while let Some(Instruction::Sub(_)) = inst_iter.peek() {
						total = total.wrapping_add(1);
						inst_iter.next();
					}

					Instruction::Sub(total)
				},
				inst => inst.to_owned(),
			};

			optimised_insts.push(optimised_inst);
		}

		UnlinkedInstructions(optimised_insts)
	}
}
