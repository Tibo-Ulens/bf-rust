use std::collections::HashMap;
use std::hash::Hash;

use itertools::Itertools;

use crate::error::Error;
use crate::instruction::{Instruction, LinkedInstructions, UnlinkedInstructions};

bitflags! {
	pub struct Optimisations: u8 {
		const COMBINE_CLEARS     = 0b00000001;
		const GROUP_INSTRUCTIONS = 0b00000010;
		const REORDER_INSTRUCTIONS = 0b00000100;
	}
}

impl Optimisations {
	pub fn from_strings(strs: &[String]) -> Self {
		let mut opts = Self::empty();

		for s in strs {
			match s.as_str() {
				"all" => opts.set(Self::all(), true),
				"combine-clears" => opts.set(Self::COMBINE_CLEARS, true),
				"group-instructions" => opts.set(Self::GROUP_INSTRUCTIONS, true),
				"reorder-instructions" => opts.set(Self::REORDER_INSTRUCTIONS, true),
				_ => (),
			}
		}

		opts
	}
}

impl UnlinkedInstructions {
	/// Apply all requested optimisations
	///
	/// List of optimisations:
	///  - Clear pattern combination: `[-]` patterns get combined into a `clear`
	/// instruction
	///  - Instruction grouping: repeated sequences of add/sub and left/right
	/// instructions get combined into a single instruction
	pub fn optimise(self, opts: Optimisations) -> Result<LinkedInstructions, Error> {
		let mut optimised_insts = self.link()?;
		if opts.contains(Optimisations::COMBINE_CLEARS) {
			optimised_insts = optimised_insts.combine_clears().link()?;
		}
		if opts.contains(Optimisations::GROUP_INSTRUCTIONS) {
			optimised_insts = optimised_insts.group_instructions().link()?;
		}
		if opts.contains(Optimisations::REORDER_INSTRUCTIONS) {
			optimised_insts = optimised_insts.reorder().link()?;
		}

		Ok(optimised_insts)
	}
}

impl LinkedInstructions {
	/// Combine `[-]` and `[+]` into a Set 0 instruction
	fn combine_clears(self) -> UnlinkedInstructions {
		let mut optimised_insts = Vec::with_capacity(self.0.len());

		let mut inst_iter = self.0.iter().peekable();
		while let Some(inst) = inst_iter.next() {
			let optimised_instruction = match inst {
				Instruction::BranchIfZero { .. } => {
					match inst_iter.peek().unwrap() {
						Instruction::Incr { amount: n, .. } if *n == 1 || *n == -1 => {
							inst_iter.next();

							if let Instruction::BranchIfNotZero { .. } = inst_iter.peek().unwrap() {
								inst_iter.next();

								Instruction::Set { amount: 0, offset: 0 }
							} else {
								optimised_insts.push(Instruction::BranchIfZero { destination: 0 });
								Instruction::Incr { amount: *n, offset: 0 }
							}
						},
						_ => Instruction::BranchIfZero { destination: 0 },
					}
				},
				inst => inst.to_owned(),
			};

			optimised_insts.push(optimised_instruction);
		}

		UnlinkedInstructions(optimised_insts)
	}

	/// Group repeated sequences of Incr, Set, and IncrIp instructions into one
	///
	/// Also merges consecutive Incr and Set instructions into a single Set
	fn group_instructions(self) -> UnlinkedInstructions {
		UnlinkedInstructions(
			self.0
				.into_iter()
				.coalesce(|prev, curr| {
					match (prev, curr) {
						// Incr(x), Incr(y) -> Incr(x + y)
						(
							Instruction::Incr { amount: prev_amt, offset: prev_ofst },
							Instruction::Incr { amount, offset },
						) if prev_ofst == offset => {
							let new_amt = prev_amt + amount;
							Ok(Instruction::Incr { amount: new_amt, offset })
						},
						// IncrIp(x), IncrIp(y) -> IncrIp(x + y)
						(
							Instruction::IncrIp { amount: prev_amt },
							Instruction::IncrIp { amount },
						) => Ok(Instruction::IncrIp { amount: prev_amt + amount }),
						// Incr(x), Set(y) -> Set(y)
						(
							Instruction::Incr { offset: prev_ofst, .. },
							Instruction::Set { amount, offset },
						) if prev_ofst == offset => Ok(Instruction::Set { amount, offset }),
						// Set(x), Incr(y) -> Set(x + y)
						(
							Instruction::Set { amount: prev_amt, offset: prev_ofst },
							Instruction::Incr { amount, offset },
						) if prev_ofst == offset => {
							let new_amt = prev_amt + amount;
							Ok(Instruction::Set { amount: new_amt, offset })
						},
						// Set(x), Set(y) -> Set(y)
						(
							Instruction::Set { offset: prev_ofst, .. },
							Instruction::Set { amount, offset },
						) if prev_ofst == offset => Ok(Instruction::Set { amount, offset }),
						_ => Err((prev, curr)),
					}
				})
				.filter(|i| {
					!(matches!(
						i,
						Instruction::Incr { amount: 0, .. } | Instruction::IncrIp { amount: 0 }
					))
				})
				.collect(),
		)
	}

	/// Reorder Incr and IncrIp instructions so the Incr instructions use
	/// offsets and there's only one IncrIp instruction at the end
	///
	/// eg. `++>+++>+` would become
	/// Incr { amount: 2, offset: 0}
	/// Incr { amount: 3, offset: 1}
	/// Incr { amount: 1, offset: 2}
	/// IncrIp { amount: 2 }
	fn reorder(self) -> UnlinkedInstructions {
		let mut sequence = vec![];
		let mut result = vec![];

		for inst in self.0 {
			match inst {
				Instruction::Incr { .. } | Instruction::Set { .. } | Instruction::IncrIp { .. } => {
					sequence.push(inst);
				},
				_ => {
					if !(sequence.is_empty()) {
						result.extend(reorder_sequence(&sequence));
						sequence = vec![];
					}

					result.push(inst);
				},
			}
		}

		if !(sequence.is_empty()) {
			result.extend(reorder_sequence(&sequence));
		}

		UnlinkedInstructions(result)
	}
}

/// Given a hashmap with sortable keys, return a vec of the values sorted by
/// their keys
fn order_hmap_values<K: Ord + Hash + Eq, V>(map: HashMap<K, V>) -> Vec<V> {
	let mut items: Vec<(K, V)> = map.into_iter().collect();
	items.sort_by(|a, b| a.0.cmp(&b.0));
	items.into_iter().map(|(_, v)| v).collect()
}

/// Given a set of Incr, IncrIp, and Set instructions, reorder them by offset
/// so there's only a single IncrIp
fn reorder_sequence(insts: &[Instruction]) -> Vec<Instruction> {
	// Keeps track of instructions with the same offset
	let mut insts_by_offset: HashMap<i64, Vec<Instruction>> = HashMap::new();
	// Keeps track of the current offset as set by IncrIp instructions
	let mut current_offset = 0;

	for inst in insts {
		match inst {
			Instruction::Incr { amount, offset } => {
				let new_offset = current_offset + offset;
				let offset_vec = insts_by_offset.entry(new_offset).or_insert_with(Vec::new);
				offset_vec.push(Instruction::Incr { amount: *amount, offset: new_offset });
			},
			Instruction::Set { amount, offset } => {
				let new_offset = current_offset + offset;
				let offset_vec = insts_by_offset.entry(new_offset).or_insert_with(Vec::new);
				offset_vec.push(Instruction::Set { amount: *amount, offset: new_offset });
			},
			Instruction::IncrIp { amount } => {
				current_offset += amount;
			},
			// Any other instructions are caller-ensured not to be present
			_ => unreachable!(),
		}
	}

	// Add all the reordered Incr/Set instructions in order of increasing
	// offset (for aestheticc)
	let mut result = vec![];
	for insts in order_hmap_values(insts_by_offset) {
		result.extend(insts);
	}

	// If there was net movement, add an IncrIp instruction to reflect it
	if current_offset != 0 {
		result.push(Instruction::IncrIp { amount: current_offset })
	}

	result
}
