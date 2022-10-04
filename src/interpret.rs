use std::fs::File;
use std::io::{Read, BufWriter, Write};
use std::path::Path;

use crate::Error;

const MEM_SIZE: usize = 65536;

pub struct Interpreter {
	ip:              usize,
	dp:              usize,
	memory:          [u8; MEM_SIZE],
	bracket_matches: Vec<usize>,
	bytes:           Vec<u8>,
}

impl Interpreter {
	pub fn new(file_path: &Path) -> Result<Self, Error> {
		let file = File::open(file_path)?;
		let mut bytes = file.bytes().try_collect::<Vec<u8>>()?;
		bytes.shrink_to_fit();

		let bracket_matches = vec![0; bytes.len()];

		Ok(Self { ip: 0, dp: 0, memory: [0; MEM_SIZE], bracket_matches, bytes })
	}

	/// Check that all brackets are balanced and store the indices of
	/// corresponding brackets in a lookup array
	fn precompute_bracket_matches(&mut self) -> Result<(), Error> {
		let mut bracket_stack: Vec<usize> = Vec::new();

		for (idx, byte) in self.bytes.iter().enumerate() {
			if *byte == b'[' {
				bracket_stack.push(idx);
			} else if *byte == b']' {
				let opening_idx = match bracket_stack.pop() {
					Some(i) => i,
					None => {
						return Err(Error::MissingOpeningBracket(idx));
					},
				};

				self.bracket_matches[opening_idx] = idx;
				self.bracket_matches[idx] = opening_idx;
			}
		}

		if bracket_stack.len() != 0 {
			return Err(Error::MissingClosingBracket(bracket_stack.pop().unwrap()));
		}

		Ok(())
	}

	/// Remove all bytes that aren't valid brainfuck instructions
	fn remove_unused_bytes(&mut self) {
		self.bytes.retain(|&byte| {
			byte == b'>'
				|| byte == b'<' || byte == b'+'
				|| byte == b'-' || byte == b'.'
				|| byte == b',' || byte == b'['
				|| byte == b']'
		});
	}

	pub fn interpret(&mut self) -> Result<(), Error> {
		self.remove_unused_bytes();
		self.bytes.shrink_to_fit();
		self.precompute_bracket_matches()?;
		self.bracket_matches.shrink_to_fit();

		let mut writer = BufWriter::new(std::io::stdout());

		while self.ip < self.bytes.len() {
			match self.bytes[self.ip] {
				b'>' => {
					self.dp = (self.dp + 1) % MEM_SIZE;
				},
				b'<' => {
					self.dp = (self.dp - 1) % MEM_SIZE;
				},
				b'+' => {
					self.memory[self.dp] = self.memory[self.dp].wrapping_add(1);
				},
				b'-' => {
					self.memory[self.dp] = self.memory[self.dp].wrapping_sub(1);
				},
				b'.' => {
					writer.write(&[self.memory[self.dp]])?;
				},
				b',' => {
					writer.flush()?;
					let input = std::io::stdin().bytes().next().and_then(|res| res.ok());

					if let Some(i) = input {
						self.memory[self.dp] = i;
					} else {
						return Err(Error::CouldNotReadInput);
					}
				},
				b'[' => {
					if self.memory[self.dp] == 0 {
						self.ip = self.bracket_matches[self.ip];
						continue;
					}
				},
				b']' => {
					if self.memory[self.dp] != 0 {
						self.ip = self.bracket_matches[self.ip];
						continue;
					}
				},
				_ => (),
			}

			self.ip += 1;
		}

		writer.flush()?;

		Ok(())
	}
}
