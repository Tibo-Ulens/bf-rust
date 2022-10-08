use std::fmt;

mod linked;
mod unlinked;

/// Instructions which have not had their jump targets linked yet
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct UnlinkedInstructions(pub Vec<Instruction>);

/// Instructions which have had their jump targets linked
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct LinkedInstructions(pub Vec<Instruction>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
	start: usize,
	end:   usize,
}

impl fmt::Display for Position {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.start == self.end {
			write!(f, "{}", self.start)
		} else {
			write!(f, "{}-{}", self.start, self.end)
		}
	}
}

pub trait Combine<T> {
	fn combine(&self, other: T) -> T;
}

impl Combine<Option<Position>> for Option<Position> {
	fn combine(&self, other: Self) -> Self {
		match (*self, other) {
			(Some(pos1), Some(pos2)) => {
				let (first_pos, second_pos) =
					if pos1.start <= pos2.start { (pos1, pos2) } else { (pos2, pos1) };

				// If they're adjacent positions, we can merge them.
				if first_pos.end + 1 >= second_pos.start {
					Some(Position { start: first_pos.start, end: second_pos.end })
				} else {
					// Otherwise, just use the second position.
					Some(pos2)
				}
			},
			_ => None,
		}
	}
}

pub type Cell = i8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Instruction {
	IncrDp { amount: i64 },
	Incr { amount: Cell, offset: i64 },
	BranchIfZero { destination: u64 },
	BranchIfNotZero { destination: u64 },
	Read,
	Write,

	// The following instructions are IR-only, the have no direct BF equivalent
	Set { amount: Cell, offset: i64 },
	Mul { amount: Cell, offset: i64 },
}

impl fmt::Display for Instruction {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::IncrDp { amount } => write!(f, "DP += {}", amount),
			Self::Incr { amount, offset } => write!(f, "MEM[DP + {}] += {}", offset, amount),
			Self::BranchIfZero { destination } => write!(f, "BRANCH FWD {}", destination),
			Self::BranchIfNotZero { destination } => write!(f, "BRANCH BCK {}", destination),
			Self::Read => write!(f, "READ -> MEM[DP]"),
			Self::Write => write!(f, "WRITE <- MEM[DP]"),
			Self::Set { amount, offset } => write!(f, "MEM[DP + {}] = {}", offset, amount),
			Self::Mul { amount, offset } => {
				write!(f, "MEM[DP + {}] += MEM[DP] * {}", offset, amount)
			},
		}
	}
}

impl Instruction {
	/// Encode the instruction as bytecode
	pub fn to_bytecode(&self) -> Vec<u8> {
		match self {
			Self::IncrDp { amount } => {
				let mut inst_bytes = vec![0];
				let amt_parts: [u8; 8] = amount.to_be_bytes();
				inst_bytes.extend_from_slice(&amt_parts);

				inst_bytes
			},
			Self::Incr { amount, offset } => {
				let mut inst_bytes = vec![1, *amount as u8];
				let ofst_parts: [u8; 8] = offset.to_be_bytes();
				inst_bytes.extend_from_slice(&ofst_parts);

				inst_bytes
			},
			Self::BranchIfZero { destination } => {
				let mut inst_bytes = vec![2];
				let dest_parts: [u8; 8] = destination.to_be_bytes();
				inst_bytes.extend_from_slice(&dest_parts);

				inst_bytes
			},
			Self::BranchIfNotZero { destination } => {
				let mut inst_bytes = vec![3];
				let dest_parts: [u8; 8] = destination.to_be_bytes();
				inst_bytes.extend_from_slice(&dest_parts);

				inst_bytes
			},
			Self::Read => {
				vec![4]
			},
			Self::Write => {
				vec![5]
			},
			Self::Set { amount, offset } => {
				let mut inst_bytes = vec![6, *amount as u8];
				let ofst_parts: [u8; 8] = offset.to_be_bytes();
				inst_bytes.extend_from_slice(&ofst_parts);

				inst_bytes
			},
			Self::Mul { amount, offset } => {
				let mut inst_bytes = vec![7, *amount as u8];
				let ofst_parts: [u8; 8] = offset.to_be_bytes();
				inst_bytes.extend_from_slice(&ofst_parts);

				inst_bytes
			},
		}
	}
}
