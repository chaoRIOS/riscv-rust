#![allow(unused)]

use std::cmp;

use pkg::{BHT_MAX_VALUE, BHT_MIN_VALUE, BHT_TAKEN_VALUE};
use pkg::{BTB_COLUMNS, BTB_INDEX_BITS, BTB_OFFSET_BITS, BTB_ROWS};

#[cfg(feature = "direct-map")]
use pkg::{BHT_COLUMNS, BHT_INDEX_BITS, BHT_OFFSET_BITS, BHT_ROWS};

#[cfg(feature = "PAG")]
use pkg::{
	BHT_NUMBER, PAG_BITS, PAG_COLUMNS, PAG_INDEX_BITS, PAG_MAX_VALUE, PAG_MIN_VALUE,
	PAG_OFFSET_BITS, PAG_ROWS,
};

pub struct BranchPredictor {
	pub branch_target_buffer: [[BTBEntry; BTB_COLUMNS]; BTB_ROWS],
	#[cfg(feature = "direct-map")]
	pub branch_history_table: [[BHTEntry; BHT_COLUMNS]; BHT_ROWS],
	#[cfg(feature = "PAG")]
	pub pag_table: [[PAGEntry; PAG_COLUMNS]; PAG_ROWS],
	#[cfg(feature = "PAG")]
	pub branch_history_table: [BHTEntry; BHT_NUMBER],
}

impl BranchPredictor {
	pub fn new() -> Self {
		BranchPredictor {
			branch_target_buffer: [[BTBEntry::new(); BTB_COLUMNS]; BTB_ROWS],
			#[cfg(feature = "direct-map")]
			branch_history_table: [[BHTEntry::new(); BHT_COLUMNS]; BHT_ROWS],
			#[cfg(feature = "PAG")]
			pag_table: [[PAGEntry::new(); PAG_COLUMNS]; PAG_ROWS],
			#[cfg(feature = "PAG")]
			branch_history_table: [BHTEntry::new(); BHT_NUMBER],
		}
	}

	pub fn predict(&self, instruction_address: u64) -> (bool, u64) {
		let btb_index = (instruction_address >> BTB_OFFSET_BITS) & ((1 << BTB_INDEX_BITS) - 1);
		let btb_offset = instruction_address & ((1 << BTB_OFFSET_BITS) - 1);

		let btb_entry = self.branch_target_buffer[btb_index as usize][btb_offset as usize];
		if btb_entry.is_valid() {
			#[cfg(feature = "direct-map")]
			{
				let bht_index =
					(instruction_address >> BHT_OFFSET_BITS) & ((1 << BHT_INDEX_BITS) - 1);
				let bht_offset = instruction_address & ((1 << BHT_OFFSET_BITS) - 1);
				let bht_entry = self.branch_history_table[bht_index as usize][bht_offset as usize];
				return (bht_entry.is_taken(), btb_entry.get_address());
			}
			#[cfg(feature = "PAG")]
			{
				let pag_index =
					(instruction_address >> PAG_OFFSET_BITS) & ((1 << PAG_INDEX_BITS) - 1);
				let pag_offset = instruction_address & ((1 << PAG_OFFSET_BITS) - 1);
				let pag_entry = self.pag_table[pag_index as usize][pag_offset as usize];
				let bht_entry = self.branch_history_table[pag_entry.predictor_index];
				return (bht_entry.is_taken(), btb_entry.get_address());
			}
		}
		(false, 0)
	}

	pub fn update(
		&mut self,
		instruction_address: u64,
		target_address: u64,
		eventually_taken: bool,
	) {
		let btb_index = (instruction_address >> BTB_OFFSET_BITS) & ((1 << BTB_INDEX_BITS) - 1);
		let btb_offset = instruction_address & ((1 << BTB_OFFSET_BITS) - 1);
		#[cfg(feature = "direct-map")]
		let bht_index = (instruction_address >> BHT_OFFSET_BITS) & ((1 << BHT_INDEX_BITS) - 1);
		#[cfg(feature = "direct-map")]
		let bht_offset = instruction_address & ((1 << BHT_OFFSET_BITS) - 1);
		#[cfg(feature = "PAG")]
		let pag_index = (instruction_address >> PAG_OFFSET_BITS) & ((1 << PAG_INDEX_BITS) - 1);
		#[cfg(feature = "PAG")]
		let pag_offset = instruction_address & ((1 << PAG_OFFSET_BITS) - 1);
		#[cfg(feature = "PAG")]
		let bht_index = self.pag_table[pag_index as usize][pag_offset as usize].predictor_index;
		#[cfg(feature = "debug-bp")]
		{
			#[cfg(feature = "direct-map")]
			println!(
				"Updating btb[{}][{}]:0x{:08x} bht[{}][{}]:{}",
				btb_index,
				btb_offset,
				target_address,
				bht_index,
				bht_offset,
				self.branch_history_table[bht_index as usize][bht_offset as usize].taken
			);
			#[cfg(feature = "PAG")]
			println!(
				"Updating btb[{}][{}]:0x{:08x} pag[{}][{}]:{} bht[{}]:{}",
				btb_index,
				btb_offset,
				target_address,
				pag_index,
				pag_offset,
				self.pag_table[pag_index as usize][pag_offset as usize].predictor_index,
				bht_index,
				self.branch_history_table[bht_index as usize].taken
			);
		}

		self.branch_target_buffer[btb_index as usize][btb_offset as usize].set_valid();
		self.branch_target_buffer[btb_index as usize][btb_offset as usize]
			.set_address(target_address);

		#[cfg(feature = "direct-map")]
		self.branch_history_table[bht_index as usize][bht_offset as usize].update(eventually_taken);
		#[cfg(feature = "PAG")]
		{
			self.pag_table[pag_index as usize][pag_offset as usize].update(eventually_taken);
			self.branch_history_table[bht_index].update(eventually_taken);
		}

		#[cfg(feature = "debug-bp")]
		{
			#[cfg(feature = "direct-map")]
			println!(
				"Updated  btb[{}][{}]:0x{:08x} bht[{}][{}]:{}",
				btb_index,
				btb_offset,
				target_address,
				bht_index,
				bht_offset,
				self.branch_history_table[bht_index as usize][bht_offset as usize].taken
			);
			#[cfg(feature = "PAG")]
			println!(
				"Updated  btb[{}][{}]:0x{:08x} pag[{}][{}]:{} bht[{}]:{}",
				btb_index,
				btb_offset,
				target_address,
				pag_index,
				pag_offset,
				self.pag_table[pag_index as usize][pag_offset as usize].predictor_index,
				bht_index,
				self.branch_history_table[bht_index as usize].taken
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
			taken: BHT_TAKEN_VALUE,
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

#[cfg(feature = "PAG")]
#[derive(Copy, Clone)]
pub struct PAGEntry {
	predictor_index: usize,
}

#[cfg(feature = "PAG")]
impl PAGEntry {
	pub fn new() -> Self {
		PAGEntry { predictor_index: 0 }
	}

	pub fn update(&mut self, eventually_taken: bool) {
		self.predictor_index = cmp::max(
			((self.predictor_index << 1)
				| match eventually_taken {
					true => 1,
					false => 0,
				}) & ((1 << PAG_BITS) - 1),
			PAG_MIN_VALUE,
		);
	}
}
