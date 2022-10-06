use crate::transpile::Instruction;
use crate::UnlinkedInstructions;

/// Wrapper struct to group optimising functions
///
/// List of optimisations:
///  - Clear pattern combination: `[-]` patterns get combined into a `clear`
/// instruction
///  - Instruction grouping: repeated sequences of add/sub and left/right
/// instructions get combined into a single instruction
pub struct Optimiser;

impl Optimiser {
	/// Combine `[-]` into a `clear` instruction
	pub fn combine_clears(insts: &UnlinkedInstructions) -> UnlinkedInstructions {
		let mut optimised_insts = Vec::with_capacity(insts.0.len());

		let mut inst_iter = insts.0.iter().peekable();
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
	pub fn group_instructions(insts: &UnlinkedInstructions) -> UnlinkedInstructions {
		let mut optimised_insts = Vec::with_capacity(insts.0.len());

		let mut inst_iter = insts.0.iter().peekable();
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
