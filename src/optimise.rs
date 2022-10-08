use std::collections::HashMap;
use std::hash::Hash;

use itertools::Itertools;

use crate::error::Error;
use crate::instruction::{Cell, Instruction, LinkedInstructions, UnlinkedInstructions};

const MAX_OPT_ITER: u8 = 20;

bitflags! {
	pub struct Optimisations: u8 {
		const COMBINE_CLEARS     = 0b00000001;
		const GROUP_INSTRUCTIONS = 0b00000010;
		const REORDER_INSTRUCTIONS = 0b00000100;
		const COMBINE_MULTIPLY_LOOPS = 0b00001000;
		const COMBINE_SCAN_LOOPS = 0b00010000;
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
				"combine-multiply-loops" => opts.set(Self::COMBINE_MULTIPLY_LOOPS, true),
				"combine-scan-loops" => opts.set(Self::COMBINE_SCAN_LOOPS, true),
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
	pub fn optimise(self, opts: &Optimisations) -> Result<LinkedInstructions, Error> {
		let mut prev = self.clone();
		let mut result = self.optimise_single_pass(opts)?;

		let mut counter = 0;
		while prev.link()? != result && counter <= MAX_OPT_ITER {
			prev = UnlinkedInstructions(result.0);
			result = prev.clone().optimise_single_pass(opts)?;

			counter += 1;
		}

		Ok(result)
	}

	fn optimise_single_pass(self, opts: &Optimisations) -> Result<LinkedInstructions, Error> {
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
		if opts.contains(Optimisations::COMBINE_MULTIPLY_LOOPS) {
			optimised_insts = optimised_insts.combine_multiply_loops().link()?;
		}
		// if opts.contains(Optimisations::COMBINE_SCAN_LOOPS) {
		// 	optimised_insts = optimised_insts.combine_scan_loops().link()?;
		// }

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
						Instruction::Incr { amount: n, offset: 0 } if *n == 1 || *n == -1 => {
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
							Instruction::IncrDp { amount: prev_amt },
							Instruction::IncrDp { amount },
						) => Ok(Instruction::IncrDp { amount: prev_amt + amount }),
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
						Instruction::Incr { amount: 0, .. } | Instruction::IncrDp { amount: 0 }
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
				Instruction::Incr { .. } | Instruction::Set { .. } | Instruction::IncrDp { .. } => {
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

	/// Recognize multiply loop patterns and combine them into a set of Mul
	/// instructions and a Set(0)
	///
	/// eg. [->++>+++<<] -> Mul(2, 1), Mul(3, 2), Set(0)
	fn combine_multiply_loops(self) -> UnlinkedInstructions {
		let mut result = vec![];

		let mut iter = self.0.iter().enumerate();
		while let Some((idx, inst)) = iter.next() {
			match inst {
				Instruction::BranchIfZero { destination } => {
					let loop_body = &self.0[(idx + 1)..(*destination as usize)];
					if let Some(mut changes) = is_multiply_loop(loop_body) {
						// The first entry will be -1 as multiply loops clear
						// their current cell, this will be replaced with a
						// Set(0)
						changes.remove(&0);

						for (ofst, amt) in changes.iter() {
							result.push(Instruction::Mul { amount: *amt, offset: *ofst });
						}
						result.push(Instruction::Set { amount: 0, offset: 0 });

						// Remove the loop body from the iterator
						iter.advance_by(*destination as usize - idx).unwrap();
					} else {
						result.push(inst.to_owned());
					}
				},
				_ => result.push(inst.to_owned()),
			}
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
			Instruction::IncrDp { amount } => {
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
		result.push(Instruction::IncrDp { amount: current_offset })
	}

	result
}

/// Check if a series of instructions matches the multiply loop pattern
///
/// If it is, return the cells that are affected
fn is_multiply_loop(insts: &[Instruction]) -> Option<HashMap<i64, Cell>> {
	let mut net_movement = 0;

	// Multiply loops can only contain Incr and IncrIp instructions
	for inst in insts {
		match inst {
			Instruction::Incr { .. } => (),
			Instruction::IncrDp { amount } => net_movement += amount,
			_ => return None,
		}
	}

	// Multiply loops should have no net movement
	if net_movement != 0 {
		return None;
	}

	// Multiply loops must decrement their first cell to 0
	let changes = cell_changes(insts);
	match changes.get(&0) {
		Some(-1) => (),
		_ => return None,
	}

	// If only a single cell is changed, we are not multiplying anything
	if changes.len() == 1 {
		return None;
	}

	Some(changes)
}

/// Return a hashmap of all the cells that are affected by this
/// sequence of instructions, and how much they change.
/// E.g. "->>+++>+" -> {0: -1, 2: 3, 3: 1}
fn cell_changes(insts: &[Instruction]) -> HashMap<i64, Cell> {
	let mut changes = HashMap::new();
	let mut cell_index = 0;

	for inst in insts {
		match *inst {
			Instruction::Incr { amount, offset } => {
				let current_amount = *changes.get(&(cell_index + offset)).unwrap_or(&0);
				changes.insert(cell_index + offset, current_amount + amount);
			},
			Instruction::IncrDp { amount } => {
				cell_index += amount;
			},
			// We assume this is only called from is_multiply_loop.
			_ => unreachable!(),
		}
	}

	changes
}
