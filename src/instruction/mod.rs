use std::fmt;

mod linked;
mod unlinked;

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
