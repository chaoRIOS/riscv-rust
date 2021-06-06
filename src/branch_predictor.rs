use std::cmp;

use pkg::{
	BHT_COLUMNS, BHT_INDEX_BITS, BHT_MAX_VALUE, BHT_MIN_VALUE, BHT_OFFSET_BITS, BHT_ROWS,
	BHT_TAKEN_VALUE, BTB_COLUMNS, BTB_INDEX_BITS, BTB_OFFSET_BITS, BTB_ROWS,
};

pub struct BranchPredictor {
	pub branch_target_buffer: [[BTBEntry; BTB_COLUMNS]; BTB_ROWS],
	pub branch_history_table: [[BHTEntry; BHT_COLUMNS]; BHT_ROWS],
}

impl BranchPredictor {
	pub fn new() -> Self {
		BranchPredictor {
			branch_target_buffer: [[BTBEntry::new(); BTB_COLUMNS]; BTB_ROWS],
			branch_history_table: [[BHTEntry::new(); BHT_COLUMNS]; BHT_ROWS],
		}
	}

	pub fn predict(&self, instruction_address: u64) -> (bool, u64) {
		let btb_index = (instruction_address >> BTB_OFFSET_BITS) & ((1 << BTB_INDEX_BITS) - 1);
		let btb_offset = instruction_address & ((1 << BTB_OFFSET_BITS) - 1);

		let btb_entry = self.branch_target_buffer[btb_index as usize][btb_offset as usize];
		if btb_entry.is_valid() {
			let bht_index = (instruction_address >> BHT_OFFSET_BITS) & ((1 << BHT_INDEX_BITS) - 1);
			let bht_offset = instruction_address & ((1 << BHT_OFFSET_BITS) - 1);
			let bht_entry = self.branch_history_table[bht_index as usize][bht_offset as usize];
			(bht_entry.is_taken(), btb_entry.get_address())
		} else {
			(false, 0)
		}
	}

	pub fn update(
		&mut self,
		instruction_address: u64,
		target_address: u64,
		eventually_taken: bool,
	) {
		let btb_index = (instruction_address >> BTB_OFFSET_BITS) & ((1 << BTB_INDEX_BITS) - 1);
		let btb_offset = instruction_address & ((1 << BTB_OFFSET_BITS) - 1);
		let bht_index = (instruction_address >> BHT_OFFSET_BITS) & ((1 << BHT_INDEX_BITS) - 1);
		let bht_offset = instruction_address & ((1 << BHT_OFFSET_BITS) - 1);
		#[cfg(feature = "debug-bp")]
		{
			println!(
				"Updating btb[{}][{}]:0x{:08x} bht[{}][{}]:{}",
				btb_index,
				btb_offset,
				target_address,
				bht_index,
				bht_offset,
				self.branch_history_table[bht_index as usize][bht_offset as usize].taken
			);
		}
		self.branch_target_buffer[btb_index as usize][btb_offset as usize].set_valid();
		self.branch_target_buffer[btb_index as usize][btb_offset as usize]
			.set_address(target_address);
		self.branch_history_table[bht_index as usize][bht_offset as usize].update(eventually_taken);
		#[cfg(feature = "debug-bp")]
		{
			println!(
				"Updated  btb[{}][{}]:0x{:08x} bht[{}][{}]:{}",
				btb_index,
				btb_offset,
				target_address,
				bht_index,
				bht_offset,
				self.branch_history_table[bht_index as usize][bht_offset as usize].taken
			);
		}
	}
}

#[derive(Copy, Clone)]
pub struct BTBEntry {
	valid: bool,
	target_address: u64,
}

impl BTBEntry {
	pub fn new() -> Self {
		BTBEntry {
			valid: false,
			target_address: 0,
		}
	}

	pub fn is_valid(&self) -> bool {
		self.valid
	}

	pub fn set_valid(&mut self) {
		self.valid = true;
	}

	pub fn get_address(&self) -> u64 {
		self.target_address
	}

	pub fn set_address(&mut self, address: u64) {
		self.target_address = address;
	}
}

#[derive(Copy, Clone)]
pub struct BHTEntry {
	taken: i32,
}

impl BHTEntry {
	pub fn new() -> Self {
		BHTEntry {
			taken: BHT_TAKEN_VALUE - 1,
		}
	}

	pub fn is_taken(&self) -> bool {
		self.taken >= BHT_TAKEN_VALUE
	}

	pub fn update(&mut self, eventually_taken: bool) {
		match eventually_taken {
			true => self.taken = cmp::min(self.taken + 1, BHT_MAX_VALUE),
			false => self.taken = cmp::max(self.taken - 1, BHT_MIN_VALUE),
		}
	}
}
