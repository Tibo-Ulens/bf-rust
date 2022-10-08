use super::{Instruction, LinkedInstructions, UnlinkedInstructions};
use crate::error::Error;

impl UnlinkedInstructions {
	/// Transpile a slice of bytes into abstract, unlinked instructions
	pub fn from_text(bytes: &[u8]) -> UnlinkedInstructions {
		let mut unlinked_insts = Vec::with_capacity(bytes.len());

		for byte in bytes.iter() {
			unlinked_insts.push(match byte {
				b'>' => Instruction::IncrDp { amount: 1 },
				b'<' => Instruction::IncrDp { amount: -1 },
				b'+' => Instruction::Incr { amount: 1, offset: 0 },
				b'-' => Instruction::Incr { amount: -1, offset: 0 },
				b'[' => Instruction::BranchIfZero { destination: 0 },
				b']' => Instruction::BranchIfNotZero { destination: 0 },
				b',' => Instruction::Read,
				b'.' => Instruction::Write,
				_ => continue,
			});
		}

		UnlinkedInstructions(unlinked_insts)
	}

	/// Set the jump targets for corresponding `[` and `]` instructions
	pub fn link(mut self) -> Result<LinkedInstructions, Error> {
		let mut jump_stack: Vec<usize> = Vec::with_capacity(5);

		for i in 0..self.0.len() {
			match &self.0[i] {
				// [
				Instruction::BranchIfZero { .. } => {
					jump_stack.push(i);
				},
				// ]
				Instruction::BranchIfNotZero { .. } => {
					let opening_idx = match jump_stack.pop() {
						Some(op_idx) => op_idx,
						None => {
							return Err(Error::MissingOpeningBracket(i));
						},
					};

					// The current instruction (]) needs to point to the opening [
					self.0[i] = Instruction::BranchIfNotZero { destination: opening_idx as u64 };
					// The corresponding [ needs to point to the current instruction
					self.0[opening_idx] = Instruction::BranchIfZero { destination: i as u64 };
				},
				_ => (),
			}
		}

		if !(jump_stack.is_empty()) {
			return Err(Error::MissingClosingBracket(jump_stack.pop().unwrap()));
		}

		Ok(LinkedInstructions(self.0))
	}
}
