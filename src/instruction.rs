use std::fmt;

/// Instructions which have not had their jump targets linked yet
#[repr(transparent)]
pub struct UnlinkedInstructions(pub Vec<Instruction>);

/// Instructions which have had their jump targets linked
#[repr(transparent)]
pub struct LinkedInstructions(pub Vec<Instruction>);

#[derive(Clone, Copy, Debug)]
pub enum Instruction {
	/// `>`
	Right(u64),
	/// `<`
	Left(u64),
	/// `+`
	Add(u8),
	/// `-`
	Sub(u8),
	/// `[`
	BranchIfZero(u64),
	/// `]`
	BranchIfNotZero(u64),
	/// `,`
	Read,
	/// `.`
	Write,
	/// `[-]`
	Clear,
}

impl fmt::Display for Instruction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Right(n) => ">".repeat(*n as usize),
				Self::Left(n) => "<".repeat(*n as usize),
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

impl Instruction {
	/// Encode the instruction as bytecode
	pub fn to_bytecode(&self) -> Vec<u8> {
		match self {
			Self::Right(n) => {
				let mut inst_bytes = vec![0];
				let parts: [u8; 8] = n.to_be_bytes();
				inst_bytes.extend_from_slice(&parts);

				inst_bytes
			},
			Self::Left(n) => {
				let mut inst_bytes = vec![1];
				let parts: [u8; 8] = n.to_be_bytes();
				inst_bytes.extend_from_slice(&parts);

				inst_bytes
			},
			Self::Add(n) => {
				vec![2, *n]
			},
			Self::Sub(n) => {
				vec![3, *n]
			},
			Self::BranchIfZero(n) => {
				let mut inst_bytes = vec![4];
				let parts: [u8; 8] = n.to_be_bytes();
				inst_bytes.extend_from_slice(&parts);

				inst_bytes
			},
			Self::BranchIfNotZero(n) => {
				let mut inst_bytes = vec![5];
				let parts: [u8; 8] = n.to_be_bytes();
				inst_bytes.extend_from_slice(&parts);

				inst_bytes
			},
			Self::Read => {
				vec![6]
			},
			Self::Write => {
				vec![7]
			},
			Self::Clear => {
				vec![8]
			},
		}
	}
}

impl UnlinkedInstructions {
	/// Transpile a slice of bytes into abstract, unlinked instructions
	pub fn transpile(bytes: &[u8]) -> UnlinkedInstructions {
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
