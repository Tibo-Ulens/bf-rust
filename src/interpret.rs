use std::io::{BufReader, BufWriter, Read, Write};

use crate::transformers::{Instruction, LinkedInstructions};
use crate::Error;

const MEM_SIZE: usize = 65536;

pub struct Interpreter<'i> {
	ip:     usize,
	dp:     usize,
	memory: [u8; MEM_SIZE],
	insts:  &'i [Instruction],
}

impl<'i> Interpreter<'i> {
	pub fn new(insts: &'i LinkedInstructions) -> Self {
		Self { ip: 0, dp: 0, memory: [0; MEM_SIZE], insts: &insts.0 }
	}

	pub fn interpret(&mut self) -> Result<(), Error> {
		let mut writer = BufWriter::new(std::io::stdout());
		let mut reader = BufReader::new(std::io::stdin());

		while self.ip < self.insts.len() {
			match self.insts[self.ip] {
				Instruction::Right(n) => {
					self.dp = (self.dp + n) % MEM_SIZE;
				},
				Instruction::Left(n) => {
					self.dp = (self.dp - n) % MEM_SIZE;
				},
				Instruction::Add(n) => {
					self.memory[self.dp] = self.memory[self.dp].wrapping_add(n);
				},
				Instruction::Sub(n) => {
					self.memory[self.dp] = self.memory[self.dp].wrapping_sub(n);
				},
				Instruction::Write => {
					writer.write_all(&[self.memory[self.dp]])?;
				},
				Instruction::Read => {
					writer.flush()?;
					let mut buffer = [0; 1];
					let bytes = reader.read(&mut buffer)?;

					if bytes == 1 {
						self.memory[self.dp] = buffer[0];
					} else {
						return Err(Error::CouldNotReadInput);
					}
				},
				Instruction::BranchIfZero(dest) => {
					if self.memory[self.dp] == 0 {
						self.ip = dest;
						continue;
					}
				},
				Instruction::BranchIfNotZero(dest) => {
					if self.memory[self.dp] != 0 {
						self.ip = dest;
						continue;
					}
				},
				Instruction::Clear => {
					self.memory[self.dp] = 0;
				},
			}

			self.ip += 1;
		}

		writer.flush()?;

		Ok(())
	}
}
