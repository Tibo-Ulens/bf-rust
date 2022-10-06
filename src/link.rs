use crate::error::Error;
use crate::transpile::Instruction;
use crate::{LinkedInstructions, UnlinkedInstructions};

/// Set the jump targets for corresponding `[` and `]` instructions
pub fn link(mut insts: UnlinkedInstructions) -> Result<LinkedInstructions, Error> {
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
				insts.0[i] = Instruction::BranchIfNotZero(opening_idx as u64);
				// The corresponding [ needs to point to the current instruction
				insts.0[opening_idx] = Instruction::BranchIfZero(i as u64);
			},
			_ => (),
		}
	}

	if !(jump_stack.is_empty()) {
		return Err(Error::MissingClosingBracket(jump_stack.pop().unwrap()));
	}

	Ok(LinkedInstructions(insts.0))
}
