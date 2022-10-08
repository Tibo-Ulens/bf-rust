use std::slice::Iter;

use super::{Instruction, LinkedInstructions};

/// Take 8 bytes from an iterator to make 64 bit values
fn take8(i: &mut Iter<u8>) -> [u8; 8] {
	[
		*i.next().unwrap(),
		*i.next().unwrap(),
		*i.next().unwrap(),
		*i.next().unwrap(),
		*i.next().unwrap(),
		*i.next().unwrap(),
		*i.next().unwrap(),
		*i.next().unwrap(),
	]
}

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
					let amt_parts = take8(&mut byte_iter);
					let amount = i64::from_be_bytes(amt_parts);

					Instruction::IncrDp { amount }
				},
				1 => {
					let amount = *byte_iter.next().unwrap() as i8;
					let ofst_parts = take8(&mut byte_iter);
					let offset = i64::from_be_bytes(ofst_parts);

					Instruction::Incr { amount, offset }
				},
				2 => {
					let parts = take8(&mut byte_iter);
					let destination = u64::from_be_bytes(parts);

					Instruction::BranchIfZero { destination }
				},
				3 => {
					let parts = take8(&mut byte_iter);
					let destination = u64::from_be_bytes(parts);

					Instruction::BranchIfNotZero { destination }
				},
				4 => Instruction::Read,
				5 => Instruction::Write,
				6 => {
					let amount = *byte_iter.next().unwrap() as i8;
					let ofst_parts = take8(&mut byte_iter);
					let offset = i64::from_be_bytes(ofst_parts);

					Instruction::Set { amount, offset }
				},
				7 => {
					let amount = *byte_iter.next().unwrap() as i8;
					let ofst_parts = take8(&mut byte_iter);
					let offset = i64::from_be_bytes(ofst_parts);

					Instruction::Mul { amount, offset }
				},
				_ => unreachable!(),
			};

			instructions.push(inst);
		}

		Self(instructions)
	}
}
