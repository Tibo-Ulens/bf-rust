use itertools::Itertools;

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
	fn combine_clears(self) -> UnlinkedInstructions {
		let mut optimised_insts = Vec::with_capacity(self.0.len());

		let mut inst_iter = self.0.iter().peekable();
		while let Some(inst) = inst_iter.next() {
			let optimised_instruction = match inst {
				Instruction::BranchIfZero { .. } => {
					if let Instruction::Incr { amount: -1, .. } = inst_iter.peek().unwrap() {
						inst_iter.next();

						if let Instruction::BranchIfNotZero { .. } = inst_iter.peek().unwrap() {
							inst_iter.next();

							Instruction::Set { amount: 0, offset: 0 }
						} else {
							optimised_insts.push(Instruction::BranchIfZero { destination: 0 });
							Instruction::Incr { amount: -1, offset: 0 }
						}
					} else {
						Instruction::BranchIfZero { destination: 0 }
					}
				},
				inst => inst.to_owned(),
			};

			optimised_insts.push(optimised_instruction);
		}

		UnlinkedInstructions(optimised_insts)
	}

	/// Group repeated sequences of add/sub and left/right instructions
	fn group_instructions(self) -> UnlinkedInstructions {
		UnlinkedInstructions(
			self.0
				.into_iter()
				.coalesce(|prev, curr| {
					if let Instruction::Incr { amount: prev_amt, offset: prev_ofst } = prev {
						if let Instruction::Incr { amount, offset } = curr {
							if prev_ofst == offset {
								return Ok(Instruction::Incr { amount: prev_amt + amount, offset });
							}
						}
					}

					Err((prev, curr))
				})
				.coalesce(|prev, curr| {
					if let Instruction::IncrIp { amount: prev_amt } = prev {
						if let Instruction::IncrIp { amount } = curr {
							return Ok(Instruction::IncrIp { amount: prev_amt + amount });
						}
					}

					Err((prev, curr))
				})
				.filter(|i| {
					!(matches!(
						i,
						Instruction::Incr { amount: 0, .. } | Instruction::IncrIp { amount: 0 }
					))
				})
				.collect(),
		)
	}
}
