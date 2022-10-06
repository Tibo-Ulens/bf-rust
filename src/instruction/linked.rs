use super::{Instruction, LinkedInstructions};

impl LinkedInstructions {
	/// Convert the instructions into a stream of bytecode
	pub fn to_bytecode(&self) -> Vec<u8> {
		let mut bytes: Vec<u8> = vec![];
		for inst in self.0.iter() {
			bytes.extend_from_slice(&inst.to_bytecode());
		}

		bytes
	}

	/// Read bytecode into a series of instructions
	pub fn from_bytecode(bytes: &[u8]) -> Self {
		let mut instructions = Vec::with_capacity(bytes.len() / 2);

		let mut byte_iter = bytes.iter();
		while let Some(b) = byte_iter.next() {
			let inst = match b {
				0 => {
					let parts: [u8; 8] = [
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
					];
					let offset = u64::from_be_bytes(parts);

					Instruction::Right(offset)
				},
				1 => {
					let parts: [u8; 8] = [
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
					];
					let offset = u64::from_be_bytes(parts);

					Instruction::Left(offset)
				},
				2 => Instruction::Add(*byte_iter.next().unwrap()),
				3 => Instruction::Sub(*byte_iter.next().unwrap()),
				4 => {
					let parts: [u8; 8] = [
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
					];
					let offset = u64::from_be_bytes(parts);

					Instruction::BranchIfZero(offset)
				},
				5 => {
					let parts: [u8; 8] = [
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
						*byte_iter.next().unwrap(),
					];
					let offset = u64::from_be_bytes(parts);

					Instruction::BranchIfNotZero(offset)
				},
				6 => Instruction::Read,
				7 => Instruction::Write,
				8 => Instruction::Clear,
				_ => unreachable!(),
			};

			instructions.push(inst);
		}

		Self(instructions)
	}
}
