use std::io::{BufReader, BufWriter, Read, Write};

use crate::error::Error;
use crate::instruction::{Instruction, LinkedInstructions};

const MEM_SIZE: usize = 65536;

pub struct Interpreter<'i> {
	ip:     usize,
	dp:     u16,
	memory: [u8; MEM_SIZE],
	insts:  &'i [Instruction],
}

impl<'i> Interpreter<'i> {
	pub fn new(insts: &'i LinkedInstructions) -> Self {
		Self { ip: 0, dp: 0, memory: [0; MEM_SIZE], insts: &insts.0 }
	}

	/// Run the provided bytecode
	pub fn run(&mut self) -> Result<(), Error> {
		let mut writer = BufWriter::new(std::io::stdout());
		let mut reader = BufReader::new(std::io::stdin());

		while self.ip < self.insts.len() {
			match self.insts[self.ip] {
				Instruction::IncrDp { amount } => {
					self.dp += amount as u16;
				},
				Instruction::Incr { amount, offset } => {
					self.memory[(self.dp + offset as u16) as usize] += amount as u8;
				},
				Instruction::Write => {
					writer.write_all(&[self.memory[self.dp as usize]])?;
				},
				Instruction::Read => {
					writer.flush()?;
					let mut buffer = [0; 1];
					let bytes = reader.read(&mut buffer)?;

					if bytes == 1 {
						self.memory[self.dp as usize] = buffer[0];
					} else {
						return Err(Error::CouldNotReadInput);
					}
				},
				Instruction::BranchIfZero { destination } => {
					if self.memory[self.dp as usize] == 0 {
						self.ip = destination as usize;
						continue;
					}
				},
				Instruction::BranchIfNotZero { destination } => {
					if self.memory[self.dp as usize] != 0 {
						self.ip = destination as usize;
						continue;
					}
				},
				Instruction::Set { amount, offset } => {
					self.memory[(self.dp + offset as u16) as usize] = amount as u8;
				},
				Instruction::Mul { amount, offset } => {
					self.memory[(self.dp + offset as u16) as usize] +=
						self.memory[self.dp as usize] * amount as u8
				},
			}

			self.ip += 1;
		}

		writer.flush()?;

		Ok(())
	}
}
