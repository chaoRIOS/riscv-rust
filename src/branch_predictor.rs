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

trait Predictor {
	fn new() -> Self;

	fn predict(&self, instruction_address: u64) -> bool;

	fn update(&mut self, instruction_address: u64, eventually_taken: bool);
}

pub struct BranchPredictor {
	pub branch_target_buffer: [[BTBEntry; BTB_COLUMNS]; BTB_ROWS],
	#[cfg(feature = "direct-map")]
	pub direct_mapping_predictor: DirectMappingPredictor,
	#[cfg(feature = "PAG")]
	pub per_address_global_predictor: PerAddressGlobalPredictor,
}

impl BranchPredictor {
	pub fn new() -> Self {
		BranchPredictor {
			branch_target_buffer: [[BTBEntry::new(); BTB_COLUMNS]; BTB_ROWS],
			#[cfg(feature = "direct-map")]
			direct_mapping_predictor: DirectMappingPredictor::new(),
			#[cfg(feature = "PAG")]
			per_address_global_predictor: PerAddressGlobalPredictor::new(),
		}
	}

	pub fn predict(&self, instruction_address: u64) -> (Vec<bool>, u64) {
		let btb_index = (instruction_address >> BTB_OFFSET_BITS) & ((1 << BTB_INDEX_BITS) - 1);
		let btb_offset = instruction_address & ((1 << BTB_OFFSET_BITS) - 1);

		let btb_entry = self.branch_target_buffer[btb_index as usize][btb_offset as usize];
		let mut predictions = vec![];
		if btb_entry.is_valid() {
			#[cfg(feature = "PAG")]
			{
				predictions.push(
					self.per_address_global_predictor
						.predict(instruction_address),
				);
			}
			#[cfg(feature = "direct-map")]
			{
				predictions.push(self.direct_mapping_predictor.predict(instruction_address));
			}
			return (predictions, btb_entry.get_address());
		}
		#[cfg(feature = "PAG")]
		predictions.push(false);
		#[cfg(feature = "direct-map")]
		predictions.push(false);

		(predictions, 0)
	}

	pub fn update(
		&mut self,
		instruction_address: u64,
		target_address: u64,
		eventually_taken: bool,
	) {
		let btb_index = (instruction_address >> BTB_OFFSET_BITS) & ((1 << BTB_INDEX_BITS) - 1);
		let btb_offset = instruction_address & ((1 << BTB_OFFSET_BITS) - 1);

		self.branch_target_buffer[btb_index as usize][btb_offset as usize].set_valid();
		self.branch_target_buffer[btb_index as usize][btb_offset as usize]
			.set_address(target_address);

		#[cfg(feature = "direct-map")]
		{
			self.direct_mapping_predictor
				.update(instruction_address, eventually_taken);
		}
		#[cfg(feature = "PAG")]
		{
			self.per_address_global_predictor
				.update(instruction_address, eventually_taken);
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

#[cfg(feature = "direct-map")]
pub struct DirectMappingPredictor {
	pub branch_history_table: [[BHTEntry; BHT_COLUMNS]; BHT_ROWS],
}

#[cfg(feature = "direct-map")]
impl Predictor for DirectMappingPredictor {
	fn new() -> Self {
		DirectMappingPredictor {
			branch_history_table: [[BHTEntry::new(); BHT_COLUMNS]; BHT_ROWS],
		}
	}

	fn predict(&self, instruction_address: u64) -> bool {
		let bht_index = (instruction_address >> BHT_OFFSET_BITS) & ((1 << BHT_INDEX_BITS) - 1);
		let bht_offset = instruction_address & ((1 << BHT_OFFSET_BITS) - 1);
		let bht_entry = self.branch_history_table[bht_index as usize][bht_offset as usize];
		return bht_entry.is_taken();
	}

	fn update(&mut self, instruction_address: u64, eventually_taken: bool) {
		let bht_index = (instruction_address >> BHT_OFFSET_BITS) & ((1 << BHT_INDEX_BITS) - 1);
		let bht_offset = instruction_address & ((1 << BHT_OFFSET_BITS) - 1);
		self.branch_history_table[bht_index as usize][bht_offset as usize].update(eventually_taken);
	}
}

#[cfg(feature = "PAG")]
pub struct PerAddressGlobalPredictor {
	pub pag_table: [[PAGEntry; PAG_COLUMNS]; PAG_ROWS],
	pub branch_history_table: [BHTEntry; BHT_NUMBER],
}

#[cfg(feature = "PAG")]
impl Predictor for PerAddressGlobalPredictor {
	fn new() -> Self {
		PerAddressGlobalPredictor {
			pag_table: [[PAGEntry::new(); PAG_COLUMNS]; PAG_ROWS],
			branch_history_table: [BHTEntry::new(); BHT_NUMBER],
		}
	}

	fn predict(&self, instruction_address: u64) -> bool {
		let pag_index = (instruction_address >> PAG_OFFSET_BITS) & ((1 << PAG_INDEX_BITS) - 1);
		let pag_offset = instruction_address & ((1 << PAG_OFFSET_BITS) - 1);
		let bht_index = self.pag_table[pag_index as usize][pag_offset as usize].predictor_index;
		let bht_entry = self.branch_history_table[bht_index as usize];
		return bht_entry.is_taken();
	}

	fn update(&mut self, instruction_address: u64, eventually_taken: bool) {
		let pag_index = (instruction_address >> PAG_OFFSET_BITS) & ((1 << PAG_INDEX_BITS) - 1);
		let pag_offset = instruction_address & ((1 << PAG_OFFSET_BITS) - 1);
		let bht_index = self.pag_table[pag_index as usize][pag_offset as usize].predictor_index;
		self.pag_table[pag_index as usize][pag_offset as usize].update(eventually_taken);
		self.branch_history_table[bht_index].update(eventually_taken);
	}
}
