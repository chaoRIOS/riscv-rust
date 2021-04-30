/// DRAM base address. Offset from this base address
/// is the address in main memory.
pub const DRAM_BASE: u64 = 0x80000000;

extern crate fnv;

use cpu::{get_privilege_mode, PrivilegeMode, Trap, TrapType, Xlen};
use l1cache::*;
use l2cache::*;
use memory::Memory;

/// Emulates Memory Management Unit. It holds the Main memory and peripheral
/// devices, maps address to them, and accesses them depending on address.
/// It also manages virtual-physical address translation and memoty protection.
/// It may also be said Bus.
/// @TODO: Memory protection is not implemented yet. We should support.

pub const TLB_ENTRY_NUM: usize = 64;

pub struct Mmu {
	pub clock: u64,
	pub xlen: Xlen,
	pub ppn: u64,
	pub addressing_mode: AddressingMode,
	pub privilege_mode: PrivilegeMode,
	pub memory: MemoryWrapper,
	pub l1_cache: L1Cache,
	pub l2_cache: L2Cache,

	pub memory_access_trace: Vec<MemoryAccessTrace>,

	/// Address translation can be affected `mstatus` (MPRV, MPP in machine mode)
	/// then `Mmu` has copy of it.
	pub mstatus: u64,
	pub tlb_tag: [u64; TLB_ENTRY_NUM],
	pub tlb_value: [u64; TLB_ENTRY_NUM],
	pub tlb_bitnum: usize,
}

pub enum AddressingMode {
	None,
	SV32,
	SV39,
	SV48, // @TODO: Implement
}

pub struct MemoryAccessTrace {
	pub address: u64,
	pub operation: MemoryAccessType,
	pub cycle: u64,
}

pub enum MemoryAccessType {
	Execute,
	Read,
	Write,
	DontCare,
}

fn _get_addressing_mode_name(mode: &AddressingMode) -> &'static str {
	match mode {
		AddressingMode::None => "None",
		AddressingMode::SV32 => "SV32",
		AddressingMode::SV39 => "SV39",
		AddressingMode::SV48 => "SV48",
	}
}

impl Mmu {
	/// Creates a new `Mmu`.
	///
	/// # Arguments
	/// * `xlen`
	pub fn new(xlen: Xlen) -> Self {
		Mmu {
			clock: 0,
			xlen: xlen,
			ppn: 0,
			addressing_mode: AddressingMode::None,
			privilege_mode: PrivilegeMode::Machine,
			memory: MemoryWrapper::new(),
			l1_cache: L1Cache::new(),
			l2_cache: L2Cache::new(),

			memory_access_trace: vec![],

			mstatus: 0,
			tlb_tag: [0; TLB_ENTRY_NUM],
			tlb_value: [0; TLB_ENTRY_NUM],
			tlb_bitnum: 0,
		}
	}

	/// Updates XLEN, 32-bit or 64-bit
	///
	/// # Arguments
	/// * `xlen`
	pub fn update_xlen(&mut self, xlen: Xlen) {
		self.xlen = xlen;
	}

	/// Initializes Main memory. This method is expected to be called only once.
	///
	/// # Arguments
	/// * `capacity`
	pub fn init_memory(&mut self, capacity: u64) {
		self.memory.init(capacity);
	}

	/// Runs one cycle of MMU and peripheral devices.
	pub fn tick(&mut self, _mip: &mut u64) -> u64 {
		let mmu_latency = self.clock;
		self.clock = 0;
		if self.memory_access_trace.len() > 0 {
			self.memory_access_trace = vec![];
		}
		mmu_latency as u64
	}

	/// Updates addressing mode
	///
	/// # Arguments
	/// * `new_addressing_mode`
	pub fn update_addressing_mode(&mut self, new_addressing_mode: AddressingMode) {
		self.addressing_mode = new_addressing_mode;
	}

	/// Updates privilege mode
	///
	/// # Arguments
	/// * `mode`
	pub fn update_privilege_mode(&mut self, mode: PrivilegeMode) {
		self.privilege_mode = mode;
	}

	/// Updates mstatus copy. `CPU` needs to call this method whenever
	/// `mstatus` is updated.
	///
	/// # Arguments
	/// * `mstatus`
	pub fn update_mstatus(&mut self, mstatus: u64) {
		self.mstatus = mstatus;
	}

	/// Updates PPN used for address translation
	///
	/// # Arguments
	/// * `ppn`
	pub fn update_ppn(&mut self, ppn: u64) {
		self.ppn = ppn;
	}

	fn get_effective_address(&self, address: u64) -> u64 {
		match self.xlen {
			Xlen::Bit32 => address & 0xffffffff,
			Xlen::Bit64 => address,
		}
	}

	/// Fetches an instruction byte. This method takes virtual address
	/// and translates into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	fn fetch(&mut self, v_address: u64) -> Result<u8, Trap> {
		match self.translate_address(v_address, &MemoryAccessType::Execute) {
			Ok(p_address) => Ok(self.load_raw(p_address)),
			Err(()) => {
				return Err(Trap {
					trap_type: TrapType::InstructionPageFault,
					value: v_address,
				})
			}
		}
	}

	/// Fetches instruction 8 bytes(64bits). This method takes virtual address
	/// and translates into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	pub fn fetch_bytes(&mut self, v_address: u64, width: u64) -> Result<Vec<u8>, Trap> {
		match (v_address & 0xfff) <= (0x1000 - width) {
			true => {
				// Fast path. All bytes fetched are in the same page so
				// translating an address only once.
				let effective_address = self.get_effective_address(v_address);
				match self.translate_address(effective_address, &MemoryAccessType::Execute) {
					Ok(p_address) => {
						let mut data = vec![];

						for i in 0..width {
							data.push(self.load_raw(p_address.wrapping_add(i)));
						}

						Ok(data)
					}
					Err(()) => Err(Trap {
						trap_type: TrapType::InstructionPageFault,
						value: effective_address,
					}),
				}
			}
			false => {
				let mut data = vec![];

				for i in 0..width {
					match self
						.translate_address(v_address.wrapping_add(i), &MemoryAccessType::Execute)
					{
						Ok(p_address) => data.push(self.load_raw(p_address)),
						Err(()) => {
							return Err(Trap {
								trap_type: TrapType::InstructionPageFault,
								value: v_address,
							})
						}
					}
				}

				Ok(data)
			}
		}
	}

	/// Fetches instruction four bytes. This method takes virtual address
	/// and translates into physical address inside.
	///
	/// @TODO: Seperate I$
	///
	/// # Arguments
	/// * `v_address` Virtual address
	pub fn fetch_word(&mut self, v_address: u64) -> Result<u32, Trap> {
		let width = 4;
		match (v_address & 0xfff) <= (0x1000 - width) {
			true => {
				// Fast path. All bytes fetched are in the same page so
				// translating an address only once.
				let effective_address = self.get_effective_address(v_address);
				match self.translate_address(effective_address, &MemoryAccessType::Execute) {
					Ok(p_address) => Ok(self.load_word_raw(p_address)),
					Err(()) => Err(Trap {
						trap_type: TrapType::InstructionPageFault,
						value: effective_address,
					}),
				}
			}
			false => {
				let mut data = 0 as u32;
				for i in 0..width {
					match self.fetch(v_address.wrapping_add(i)) {
						Ok(byte) => data |= (byte as u32) << (i * 8),
						Err(e) => return Err(e),
					};
				}
				Ok(data)
			}
		}
	}

	/// Write back method for L1 cache
	///
	/// # Arguments
	/// * `victim_cache_line`: the line needs to be written back
	/// * `index`: index of the line
	fn l1_write_back(&mut self, victim_cache_line: L1CacheLine, l1_index: u64) {
		let mut write_back_address = 0 as u64;
		// [ tag | index | 000000 ]
		write_back_address = write_back_address
			| (victim_cache_line.tag << (L1_CACHE_INDEX_BITS + L1_CACHE_OFFSET_BITS))
			| (l1_index << L1_CACHE_OFFSET_BITS);

		// Write back to L2
		// Latency for checking
		self.clock = self.clock.wrapping_add(L1_CACHE_HIT_LATENCY as u64);
		// Latency for checking
		self.clock = self.clock.wrapping_add(L2_CACHE_HIT_LATENCY as u64);
		match self.l2_cache.read_line_info(write_back_address) {
			Ok(l2_way) => {
				let l2_index: u64 =
					(write_back_address >> L2_CACHE_OFFSET_BITS) & ((1 << L2_CACHE_INDEX_BITS) - 1);
				self.l2_cache.data[l2_index as usize].data[l2_way as usize].data_blocks =
					victim_cache_line.data_blocks;
				self.l2_cache.data[l2_index as usize].data[l2_way as usize].l1_inclusive = false;
			}
			_ => {}
		}
	}

	/// Allocate a new L1 entry
	/// with returning the allocated way index
	///
	/// # Arguments
	/// * `index`: physical address
	fn l1_miss_handler(&mut self, index: u64) -> Result<u64, ()> {
		// let index: u64 = (index >> L1_CACHE_OFFSET_BITS) & ((1 << L1_CACHE_INDEX_BITS) - 1);
		let way = self
			.l1_cache
			.allocate_new_line(index, PlacementPolicy::Random);
		let victim = self.l1_cache.data[index as usize].data[way as usize];

		// Detect victim cache line
		// write back the victim
		match victim.valid {
			true => {
				self.l1_write_back(victim, index);
				// Latency for checking
				self.clock = self.clock.wrapping_add(L1_CACHE_HIT_LATENCY as u64);
			}
			false => {
				// Latency for checking
				self.clock = self.clock.wrapping_add(L1_CACHE_HIT_LATENCY as u64);
			}
		}

		Ok(way as u64)
	}

	/// Refill a line to L1 cache after allocation and writing back
	/// Modified downward interface of L1
	///
	/// # Arguments
	/// * `index`: pre-parsed index for the p_address
	/// * `way`: allocated way in L1 cache
	/// * `cache_line` : target cache line
	pub fn l1_refill(&mut self, index: u64, way: u64, cache_line: L1CacheLine) -> Result<(), ()> {
		// Place the line
		#[cfg(debug_assertions)]
		println!("refill[{}][{}]: {:x?}", index, way, cache_line.data_blocks);
		self.l1_cache.data[index as usize].data[way as usize] = cache_line;
		self.l1_cache.data[index as usize].data[way as usize].valid = true;
		Ok(())
	}

	/// Flush L1 cache
	/// (e.g. FENCE)
	pub fn l1_flush(&mut self) {
		for index in 0..L1_CACHE_SET_NUMBER {
			for way in 0..L1_SET_ASSOCIATIVE_WAY {
				if self.l1_cache.data[index as usize].data[way as usize].valid == true {
					// invalid
					self.l1_cache.data[index as usize].data[way as usize].valid = false;
					// write back
					self.l1_write_back(
						self.l1_cache.data[index as usize].data[way as usize],
						index as u64,
					);
				}
			}
		}
	}

	/// Write back method for L2 cache
	///
	/// # Arguments
	/// * `victim_cache_line`: the line needs to be written back
	/// * `index`: index of the line
	fn l2_write_back(&mut self, victim_cache_line: L2CacheLine, index: u64) {
		let mut write_back_address = 0 as u64;
		// [ tag | index | 000000 ]
		write_back_address = write_back_address
			| (victim_cache_line.tag << (L2_CACHE_INDEX_BITS + L2_CACHE_OFFSET_BITS))
			| (index << L2_CACHE_OFFSET_BITS);

		// Reuse current memory interface
		// @TODO: optimize
		for i in 0..(L2_CACHE_BLOCK_SIZE / 8) as u64 {
			self.memory
				.write_doubleword(write_back_address + i * 8, victim_cache_line.get(i * 8, 8));
		}

		// Latency for checking
		self.clock = self.clock.wrapping_add(L2_CACHE_HIT_LATENCY as u64);

		// Trace memory access
		self.memory_access_trace.push(MemoryAccessTrace {
			address: write_back_address,
			operation: MemoryAccessType::Write,
			cycle: self.clock,
		});

		// Latency for accessing memory
		self.clock = self.clock.wrapping_add(L2_CACHE_MISS_LATENCY as u64);
	}

	/// Allocate a new L2 entry
	/// with returning the allocated way index
	///
	/// # Arguments
	/// * `index`: physical address
	fn l2_miss_handler(&mut self, index: u64) -> Result<u64, ()> {
		// let index: u64 = (index >> L2_CACHE_OFFSET_BITS) & ((1 << L2_CACHE_INDEX_BITS) - 1);
		let way = self
			.l2_cache
			.allocate_new_line(index, PlacementPolicy::Random);
		let victim = self.l2_cache.data[index as usize].data[way as usize];

		// Detect victim cache line
		// write back the victim
		match victim.valid {
			true => {
				self.l2_write_back(victim, index);
				// Latency for checking
				self.clock = self.clock.wrapping_add(L2_CACHE_HIT_LATENCY as u64);
			}
			false => {
				// Latency for checking
				self.clock = self.clock.wrapping_add(L2_CACHE_HIT_LATENCY as u64);
			}
		}

		Ok(way as u64)
	}

	/// Refill a line to L2 cache after allocation and writing back
	/// Modified downward interface of L2
	///
	/// # Arguments
	/// * `index`: pre-parsed index for the p_address
	/// * `way`: allocated way in L2 cache
	/// * `cache_line` : target cache line
	pub fn l2_refill(&mut self, index: u64, way: u64, cache_line: L2CacheLine) -> Result<(), ()> {
		// Place the line
		#[cfg(debug_assertions)]
		println!("refill[{}][{}]: {:x?}", index, way, cache_line.data_blocks);
		self.l2_cache.data[index as usize].data[way as usize] = cache_line;
		self.l2_cache.data[index as usize].data[way as usize].valid = true;
		Ok(())
	}

	/// Flush L2 cache
	/// (e.g. FENCE)
	pub fn l2_flush(&mut self) {
		for index in 0..L2_CACHE_SET_NUMBER {
			for way in 0..L2_SET_ASSOCIATIVE_WAY {
				if self.l2_cache.data[index as usize].data[way as usize].valid == true {
					// invalid
					self.l2_cache.data[index as usize].data[way as usize].valid = false;
					// write back
					self.l2_write_back(
						self.l2_cache.data[index as usize].data[way as usize],
						index as u64,
					);
				}
			}
		}
	}

	/// General memory subsystem interface
	///
	/// # Arguments
	/// * `p_address` : p_address
	fn aquire_cache_line(&mut self, p_address: u64) -> Result<u64, Trap> {
		// pre-parse index
		let l1_index: u64 = (p_address >> L1_CACHE_OFFSET_BITS) & ((1 << L1_CACHE_INDEX_BITS) - 1);
		let l1_tag: u64 = p_address >> (L1_CACHE_INDEX_BITS + L1_CACHE_OFFSET_BITS);

		match self.l1_cache.read_line_info(p_address) {
			Ok(l1_way) => {
				// L1 hit
				self.clock = self.clock.wrapping_add(L1_CACHE_HIT_LATENCY as u64);
				self.l1_cache.hit_num += 1;
				Ok(l1_way)
			}
			Err(()) => {
				// L1 miss

				// Performance counter
				self.l1_cache.miss_num += 1;

				// pre-parse index
				let l2_index: u64 =
					(p_address >> L2_CACHE_OFFSET_BITS) & ((1 << L2_CACHE_INDEX_BITS) - 1);
				let l2_tag: u64 = p_address >> (L2_CACHE_INDEX_BITS + L2_CACHE_OFFSET_BITS);

				// Allocate L1 entry
				// Latency of L1 miss handler:
				// Check: if no write back
				// Check + Check_L2(must hit) + Check: if write back
				let l1_way_allocated = self.l1_miss_handler(l1_index).unwrap();

				// Aquire a new line
				let mut new_line = L1CacheLine::new();

				// Access L2 cache
				match self.l2_cache.read_line_info(p_address) {
					Ok(l2_way) => {
						// L1 miss but L2 hit

						// Performance counter
						self.l2_cache.hit_num += 1;

						// Latency for checking
						self.clock = self.clock.wrapping_add(L2_CACHE_HIT_LATENCY as u64);

						// Refill with hitted L2 line
						new_line.data_blocks =
							self.l2_cache.data[l2_index as usize].data[l2_way as usize].data_blocks;
						new_line.valid = true;
						new_line.tag = l1_tag;

						match self.l1_refill(l1_index, l1_way_allocated, new_line) {
							// Eventual returning value
							// Only data_blocks interested
							Ok(()) => {
								self.l2_cache.data[l2_index as usize].data[l2_way as usize]
									.l1_inclusive = true;
								Ok(l1_way_allocated)
							}
							Err(()) => {
								// @TODO: determine TrapType
								Err(Trap {
									trap_type: TrapType::LoadAccessFault,
									value: p_address,
								})
							}
						}
					}
					Err(()) => {
						// L1 miss and L2 miss

						// Performance counter
						self.l2_cache.miss_num += 1;

						// Access memory
						// @TODO : optimize
						// Align cache line
						let p_address_aligned =
							(p_address >> L1_CACHE_OFFSET_BITS) << L1_CACHE_OFFSET_BITS;

						// Latency for misshandler:
						// Check: if no write back
						// Check + memory access + Check : if write back
						let l2_way_allocated = self.l2_miss_handler(l2_index).unwrap();

						// Access memory for new line
						//
						#[cfg(debug_assertions)]
						println!("refill from {:x}", p_address_aligned);
						for i in 0..L1_CACHE_BLOCK_SIZE {
							new_line.data_blocks[i as usize] =
								self.load_raw(p_address_aligned + i as u64);
						}

						// Trace memory access
						self.memory_access_trace.push(MemoryAccessTrace {
							address: p_address_aligned,
							operation: MemoryAccessType::Read,
							cycle: self.clock,
						});

						// Latency for accessing memory
						self.clock = self.clock.wrapping_add(L2_CACHE_MISS_LATENCY as u64);

						// Refill L2 with new line
						match self.l2_refill(
							l2_index,
							l2_way_allocated,
							L2CacheLine {
								l1_inclusive: true,
								valid: true,
								tag: l2_tag,
								data_blocks: new_line.data_blocks,
							},
						) {
							Ok(()) => {
								// Refill L1 with new line
								match self.l1_refill(
									l1_index,
									l1_way_allocated,
									L1CacheLine {
										valid: true,
										tag: l1_tag,
										data_blocks: new_line.data_blocks,
									},
								) {
									// Eventual returning value
									// Only data_blocks interested
									Ok(()) => Ok(l1_way_allocated),
									Err(()) => {
										// @TODO: determine TrapType
										Err(Trap {
											trap_type: TrapType::LoadAccessFault,
											value: p_address,
										})
									}
								}
							}
							Err(()) => {
								// @TODO: determine TrapType
								Err(Trap {
									trap_type: TrapType::LoadAccessFault,
									value: p_address,
								})
							}
						}
					}
				}
			}
		}
	}

	/// Loads multiple bytes. This method takes virtual address and translates
	/// into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	/// * `width` Must be 1, 2, 4, or 8
	fn load_bytes(&mut self, v_address: u64, width: u64) -> Result<u64, Trap> {
		debug_assert!(
			width == 1 || width == 2 || width == 4 || width == 8,
			"Width must be 1, 2, 4, or 8. {:X}",
			width
		);
		match (v_address & 0xfff) <= (0x1000 - width) {
			true => match self.translate_address(v_address, &MemoryAccessType::Read) {
				Ok(p_address) => {
					#[cfg(debug_assertions)]
					println!("\nload {} @ 0x{:x}", width, p_address);

					// pre-parse index
					let l1_index: u64 =
						(p_address >> L1_CACHE_OFFSET_BITS) & ((1 << L1_CACHE_INDEX_BITS) - 1);
					let _l1_tag: u64 = p_address >> (L1_CACHE_INDEX_BITS + L1_CACHE_OFFSET_BITS);
					let l1_offset = p_address & ((1 << L1_CACHE_OFFSET_BITS) - 1);

					let l1_way = match self.aquire_cache_line(p_address) {
						Ok(l1_way) => l1_way,
						Err(trap) => return Err(trap),
					};

					Ok(self.l1_cache.data[l1_index as usize].data[l1_way as usize]
						.get(l1_offset, width))
				}
				Err(()) => Err(Trap {
					trap_type: TrapType::LoadPageFault,
					value: v_address,
				}),
			},
			false => {
				let mut data = 0 as u64;
				for i in 0..width {
					match self.load(v_address.wrapping_add(i)) {
						Ok(byte) => data |= (byte as u64) << (i * 8),
						Err(e) => return Err(e),
					};
				}
				Ok(data)
			}
		}
	}

	/// Loads an byte. This method takes virtual address and translates
	/// into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	pub fn load(&mut self, v_address: u64) -> Result<u8, Trap> {
		match self.load_bytes(v_address, 1) {
			Ok(data) => Ok(data as u8),
			Err(e) => Err(e),
		}
	}

	/// Loads two bytes. This method takes virtual address and translates
	/// into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	pub fn load_halfword(&mut self, v_address: u64) -> Result<u16, Trap> {
		match self.load_bytes(v_address, 2) {
			Ok(data) => Ok(data as u16),
			Err(e) => Err(e),
		}
	}

	/// Loads four bytes. This method takes virtual address and translates
	/// into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	pub fn load_word(&mut self, v_address: u64) -> Result<u32, Trap> {
		match self.load_bytes(v_address, 4) {
			Ok(data) => Ok(data as u32),
			Err(e) => Err(e),
		}
	}

	/// Loads eight bytes. This method takes virtual address and translates
	/// into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	pub fn load_doubleword(&mut self, v_address: u64) -> Result<u64, Trap> {
		match self.load_bytes(v_address, 8) {
			Ok(data) => Ok(data as u64),
			Err(e) => Err(e),
		}
	}

	/// Stores multiple bytes. This method takes virtual address and translates
	/// into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	/// * `value` data written
	/// * `width` Must be 1, 2, 4, or 8
	fn store_bytes(&mut self, v_address: u64, value: u64, width: u64) -> Result<(), Trap> {
		debug_assert!(
			width == 1 || width == 2 || width == 4 || width == 8,
			"Width must be 1, 2, 4, or 8. {:X}",
			width
		);
		match (v_address & 0xfff) <= (0x1000 - width) {
			true => match self.translate_address(v_address, &MemoryAccessType::Write) {
				Ok(p_address) => {
					// Store to cache
					// @TODO: store buffer
					// Get allocated cache line's way index
					#[cfg(debug_assertions)]
					println!("\nstore {} @ 0x{:x}", width, p_address);

					// pre-parse index
					let l1_index: u64 =
						(p_address >> L1_CACHE_OFFSET_BITS) & ((1 << L1_CACHE_INDEX_BITS) - 1);
					let _l1_tag: u64 = p_address >> (L1_CACHE_INDEX_BITS + L1_CACHE_OFFSET_BITS);
					let l1_offset = p_address & ((1 << L1_CACHE_OFFSET_BITS) - 1);

					let l1_way = match self.aquire_cache_line(p_address) {
						Ok(l1_way) => l1_way,
						Err(trap) => return Err(trap),
					};

					// Update cache line
					self.l1_cache.data[l1_index as usize].data[l1_way as usize]
						.set(l1_offset, width, value);

					Ok(())
				}
				Err(()) => Err(Trap {
					trap_type: TrapType::StorePageFault,
					value: v_address,
				}),
			},
			false => {
				for i in 0..width {
					match self.store(v_address.wrapping_add(i), ((value >> (i * 8)) & 0xff) as u8) {
						Ok(()) => {}
						Err(e) => return Err(e),
					}
				}
				Ok(())
			}
		}
	}

	/// Store an byte. This method takes virtual address and translates
	/// into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	/// * `value`
	pub fn store(&mut self, v_address: u64, value: u8) -> Result<(), Trap> {
		self.store_bytes(v_address, value as u64, 1)
	}

	/// Stores two bytes. This method takes virtual address and translates
	/// into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	/// * `value` data written
	pub fn store_halfword(&mut self, v_address: u64, value: u16) -> Result<(), Trap> {
		self.store_bytes(v_address, value as u64, 2)
	}

	/// Stores four bytes. This method takes virtual address and translates
	/// into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	/// * `value` data written
	pub fn store_word(&mut self, v_address: u64, value: u32) -> Result<(), Trap> {
		self.store_bytes(v_address, value as u64, 4)
	}

	/// Stores eight bytes. This method takes virtual address and translates
	/// into physical address inside.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	/// * `value` data written
	pub fn store_doubleword(&mut self, v_address: u64, value: u64) -> Result<(), Trap> {
		self.store_bytes(v_address, value as u64, 8)
	}

	/// Loads a byte from main memory or peripheral devices depending on
	/// physical address.
	///
	/// # Arguments
	/// * `p_address` Physical address
	pub fn load_raw(&mut self, p_address: u64) -> u8 {
		let effective_address = self.get_effective_address(p_address);
		self.memory.read_byte(effective_address)
	}

	/// Loads two bytes from main memory or peripheral devices depending on
	/// physical address.
	///
	/// # Arguments
	/// * `p_address` Physical address
	pub fn load_halfword_raw(&mut self, p_address: u64) -> u16 {
		let effective_address = self.get_effective_address(p_address);
		match effective_address >= DRAM_BASE
			&& effective_address.wrapping_add(1) > effective_address
		{
			// Fast path. Directly load main memory at a time.
			true => self.memory.read_halfword(effective_address),
			false => {
				let mut data = 0 as u16;
				for i in 0..2 {
					data |= (self.load_raw(effective_address.wrapping_add(i)) as u16) << (i * 8)
				}
				data
			}
		}
	}

	/// Loads four bytes from main memory or peripheral devices depending on
	/// physical address.
	///
	/// # Arguments
	/// * `p_address` Physical address
	pub fn load_word_raw(&mut self, p_address: u64) -> u32 {
		let effective_address = self.get_effective_address(p_address);
		match effective_address >= DRAM_BASE
			&& effective_address.wrapping_add(3) > effective_address
		{
			// Fast path. Directly load main memory at a time.
			true => self.memory.read_word(effective_address),
			false => {
				let mut data = 0 as u32;
				for i in 0..4 {
					data |= (self.load_raw(effective_address.wrapping_add(i)) as u32) << (i * 8)
				}
				data
			}
		}
	}

	/// Loads eight bytes from main memory or peripheral devices depending on
	/// physical address.
	///
	/// # Arguments
	/// * `p_address` Physical address
	fn load_doubleword_raw(&mut self, p_address: u64) -> u64 {
		let effective_address = self.get_effective_address(p_address);
		match effective_address >= DRAM_BASE
			&& effective_address.wrapping_add(7) > effective_address
		{
			// Fast path. Directly load main memory at a time.
			true => self.memory.read_doubleword(effective_address),
			false => {
				let mut data = 0 as u64;
				for i in 0..8 {
					data |= (self.load_raw(effective_address.wrapping_add(i)) as u64) << (i * 8)
				}
				data
			}
		}
	}

	/// Stores a byte to main memory or peripheral devices depending on
	/// physical address.
	///
	/// # Arguments
	/// * `p_address` Physical address
	/// * `value` data written
	pub fn store_raw(&mut self, p_address: u64, value: u8) {
		let effective_address = self.get_effective_address(p_address);
		self.memory.write_byte(effective_address, value);
	}

	/// Stores two bytes to main memory or peripheral devices depending on
	/// physical address.
	///
	/// # Arguments
	/// * `p_address` Physical address
	/// * `value` data written
	fn _store_halfword_raw(&mut self, p_address: u64, value: u16) {
		let effective_address = self.get_effective_address(p_address);
		match effective_address >= DRAM_BASE
			&& effective_address.wrapping_add(1) > effective_address
		{
			// Fast path. Directly store to main memory at a time.
			true => self.memory.write_halfword(effective_address, value),
			false => {
				for i in 0..2 {
					self.store_raw(
						effective_address.wrapping_add(i),
						((value >> (i * 8)) & 0xff) as u8,
					);
				}
			}
		}
	}

	/// Stores four bytes to main memory or peripheral devices depending on
	/// physical address.
	///
	/// # Arguments
	/// * `p_address` Physical address
	/// * `value` data written
	pub fn store_word_raw(&mut self, p_address: u64, value: u32) {
		let effective_address = self.get_effective_address(p_address);
		match effective_address >= DRAM_BASE
			&& effective_address.wrapping_add(3) > effective_address
		{
			// Fast path. Directly store to main memory at a time.
			true => self.memory.write_word(effective_address, value),
			false => {
				for i in 0..4 {
					self.store_raw(
						effective_address.wrapping_add(i),
						((value >> (i * 8)) & 0xff) as u8,
					);
				}
			}
		}
	}

	/// Stores eight bytes to main memory or peripheral devices depending on
	/// physical address.
	///
	/// # Arguments
	/// * `p_address` Physical address
	/// * `value` data written
	pub fn store_doubleword_raw(&mut self, p_address: u64, value: u64) {
		let effective_address = self.get_effective_address(p_address);
		match effective_address >= DRAM_BASE
			&& effective_address.wrapping_add(7) > effective_address
		{
			// Fast path. Directly store to main memory at a time.
			true => self.memory.write_doubleword(effective_address, value),
			false => {
				for i in 0..8 {
					self.store_raw(
						effective_address.wrapping_add(i),
						((value >> (i * 8)) & 0xff) as u8,
					);
				}
			}
		}
	}

	/// Checks if passed virtual address is valid (pointing a certain device) or not.
	/// This method can return page fault trap.
	///
	/// # Arguments
	/// * `v_address` Virtual address
	pub fn validate_address(&mut self, v_address: u64) -> Result<bool, ()> {
		// @TODO: Support other access types?
		let p_address = match self.translate_address(v_address, &MemoryAccessType::DontCare) {
			Ok(address) => address,
			Err(()) => return Err(()),
		};
		let effective_address = self.get_effective_address(p_address);
		let valid = match effective_address >= DRAM_BASE {
			true => self.memory.validate_address(effective_address),
			false => match effective_address {
				0x00001020..=0x00001fff => true,
				0x02000000..=0x0200ffff => true,
				0x0C000000..=0x0fffffff => true,
				0x10000000..=0x100000ff => true,
				0x10001000..=0x10001FFF => true,
				_ => false,
			},
		};
		Ok(valid)
	}

	fn translate_address(
		&mut self,
		v_address: u64,
		access_type: &MemoryAccessType,
	) -> Result<u64, ()> {
		let address = self.get_effective_address(v_address);
		let p_address = match self.addressing_mode {
			AddressingMode::None => Ok(address),
			AddressingMode::SV32 => match self.privilege_mode {
				// @TODO: Optimize
				PrivilegeMode::Machine => match access_type {
					MemoryAccessType::Execute => Ok(address),
					// @TODO: Remove magic number
					_ => match (self.mstatus >> 17) & 1 {
						0 => Ok(address),
						_ => {
							let privilege_mode = get_privilege_mode((self.mstatus >> 9) & 3);
							match privilege_mode {
								PrivilegeMode::Machine => Ok(address),
								_ => {
									let current_privilege_mode = self.privilege_mode.clone();
									self.update_privilege_mode(privilege_mode);
									let result = self.translate_address(v_address, access_type);
									self.update_privilege_mode(current_privilege_mode);
									result
								}
							}
						}
					},
				},
				PrivilegeMode::User | PrivilegeMode::Supervisor => {
					let vpns = [(address >> 12) & 0x3ff, (address >> 22) & 0x3ff];
					self.tlb_or_pagewalk(address, 2 - 1, self.ppn, &vpns, &access_type)
				}
				_ => Ok(address),
			},
			AddressingMode::SV39 => match self.privilege_mode {
				// @TODO: Optimize
				// @TODO: Remove duplicated code with SV32
				PrivilegeMode::Machine => match access_type {
					MemoryAccessType::Execute => Ok(address),
					// @TODO: Remove magic number
					_ => match (self.mstatus >> 17) & 1 {
						0 => Ok(address),
						_ => {
							let privilege_mode = get_privilege_mode((self.mstatus >> 9) & 3);
							match privilege_mode {
								PrivilegeMode::Machine => Ok(address),
								_ => {
									let current_privilege_mode = self.privilege_mode.clone();
									self.update_privilege_mode(privilege_mode);
									let result = self.translate_address(v_address, access_type);
									self.update_privilege_mode(current_privilege_mode);
									result
								}
							}
						}
					},
				},
				PrivilegeMode::User | PrivilegeMode::Supervisor => {
					let vpns = [
						(address >> 12) & 0x1ff,
						(address >> 21) & 0x1ff,
						(address >> 30) & 0x1ff,
					];
					self.tlb_or_pagewalk(address, 3 - 1, self.ppn, &vpns, &access_type)
				}
				_ => Ok(address),
			},
			AddressingMode::SV48 => {
				panic!("AddressingMode SV48 is not supported yet.");
			}
		};
		p_address
	}

	fn tlb_entry_search(&mut self, vpns: u64) -> usize {
		let mask = match self.addressing_mode {
			AddressingMode::SV32 => ((1 << 12) - 1) ^ ((1 << 32) - 1),
			_ => ((1 << 12) - 1) ^ ((1 << 39) - 1),
		};

		for i in 0..TLB_ENTRY_NUM {
			/*
			if self.tlb_tag[i] & 1 == 1 {
				return i;
			}
			*/
			if (vpns & mask) == (self.tlb_tag[i] & mask) {
				// hit !
				return i;
			}
		}
		return TLB_ENTRY_NUM + 1;
	}

	fn tlb_entry_avaliable(&mut self, vpns: u64) -> bool {
		let i: usize = self.tlb_entry_search(vpns);
		return (self.tlb_tag[i] & 1) == 1 && (i != TLB_ENTRY_NUM + 1);
	}

	fn tlb_bitnum_increased(&mut self, i: usize) {
		if self.tlb_tag[i] & 2 != 2 {
			self.tlb_tag[i] = self.tlb_tag[i] | 2; // bit PLRU algorithm
			self.tlb_bitnum = self.tlb_bitnum + 1;
			if self.tlb_bitnum == TLB_ENTRY_NUM {
				for j in 0..TLB_ENTRY_NUM {
					self.tlb_tag[j] = self.tlb_tag[j] & (!2);
				}
				self.tlb_bitnum = 0;
			}
		}
	}

	fn tlb_get_entry(&mut self, vpns: u64) -> u64 {
		let i: usize = self.tlb_entry_search(vpns);
		self.tlb_bitnum_increased(i);
		self.tlb_value[i]
	}

	fn tlb_update_entry(&mut self, vpns: u64, new_pte: u64) {
		let i: usize = self.tlb_entry_search(vpns);
		if i == TLB_ENTRY_NUM + 1 {
			// find if exist free entry
			for j in 0..TLB_ENTRY_NUM {
				if self.tlb_tag[j] & 1 == 0 {
					// found free entry
					self.tlb_tag[j] = vpns | 1;
					self.tlb_value[j] = new_pte;
					self.tlb_bitnum_increased(j);
					return;
				}
			}
			// no free entry, select a victim
			for j in 0..TLB_ENTRY_NUM {
				if self.tlb_tag[j] & 2 == 0 {
					// found victim, swap it
					self.tlb_tag[j] = vpns | 1;
					self.tlb_value[j] = new_pte;
					self.tlb_bitnum_increased(j);
				}
			}
			// no victim
			println!(
				"tlb_update_entry error: no free slot or victim, this situation shouldnt occur!"
			);
			panic!();
		}
	}

	fn tlb_or_pagewalk(
		&mut self,
		v_address: u64,
		level: u8,
		parent_ppn: u64,
		vpns: &[u64],
		access_type: &MemoryAccessType,
	) -> Result<u64, ()> {
		let pagesize = 4096;
		let ptesize = match self.addressing_mode {
			AddressingMode::SV32 => 4,
			_ => 8,
		};
		let pte_address = parent_ppn * pagesize + vpns[level as usize] * ptesize;
		let vpn = match self.addressing_mode {
			AddressingMode::SV32 => (vpns[0] << 12) | (vpns[1] << 22),
			AddressingMode::SV39 => (vpns[0] << 12) | (vpns[1] << 21) | (vpns[2] << 30),
			_ => (vpns[0] << 12) | (vpns[1] << 21) | (vpns[2] << 30),
		} as u64;
		let pte = match self.addressing_mode {
			AddressingMode::SV32 => match self.tlb_entry_avaliable(vpn) {
				true => self.tlb_get_entry(vpn),
				_ => {
					let tmp = self.load_word_raw(pte_address) as u64;
					let tmp_x = (tmp >> 3) & 1;
					let tmp_w = (tmp >> 2) & 1;
					let tmp_r = (tmp >> 1) & 1;
					if tmp_x != 0 || tmp_r != 0 || tmp_w != 0 {
						// a leaf PTE
						self.tlb_update_entry(vpn, tmp);
					}
					tmp
				},

			},  
			_ => {
				self.load_doubleword_raw(pte_address) 
				/*match self.tlb_entry_avaliable(vpn) {
				true=>self.tlb_get_entry(vpn),
				_=>{
					let tmp = self.load_doubleword_raw(pte_address);
					let tmp_x = (tmp >> 3) & 1;
					let tmp_w = (tmp >> 2) & 1;
					let tmp_r = (tmp >> 1) & 1;
					if tmp_x != 0 || tmp_r != 0 || tmp_w != 0 {
						self.tlb_update_entry(vpn, tmp);
					}
					tmp
				},*/
			} 
		};
		let ppn = match self.addressing_mode {
			AddressingMode::SV32 => (pte >> 10) & 0x3fffff,
			_ => (pte >> 10) & 0xfffffffffff,
		};
		let ppns = match self.addressing_mode {
			AddressingMode::SV32 => [(pte >> 10) & 0x3ff, (pte >> 20) & 0xfff, 0 /*dummy*/],
			AddressingMode::SV39 => [
				(pte >> 10) & 0x1ff,
				(pte >> 19) & 0x1ff,
				(pte >> 28) & 0x3ffffff,
			],
			_ => panic!(), // Shouldn't happen
		};
		let _rsw = (pte >> 8) & 0x3;
		let d = (pte >> 7) & 1;
		let a = (pte >> 6) & 1;
		let _g = (pte >> 5) & 1;
		let _u = (pte >> 4) & 1;
		let x = (pte >> 3) & 1;
		let w = (pte >> 2) & 1;
		let r = (pte >> 1) & 1;
		let v = pte & 1;

		// println!("VA:{:X} Level:{:X} PTE_AD:{:X} PTE:{:X} PPPN:{:X} PPN:{:X} PPN1:{:X} PPN0:{:X}", v_address, level, pte_address, pte, parent_ppn, ppn, ppns[1], ppns[0]);

		if v == 0 || (r == 0 && w == 1) {
			return Err(());
		}

		if r == 0 && x == 0 {
			return match level {
				0 => Err(()),
				_ => self.tlb_or_pagewalk(v_address, level - 1, ppn, vpns, access_type),
			};
		}

		// Leaf page found

		if a == 0
			|| (match access_type {
				MemoryAccessType::Write => d == 0,
				_ => false,
			}) {
			let new_pte = pte
				| (1 << 6) | (match access_type {
				MemoryAccessType::Write => 1 << 7,
				_ => 0,
			});
			match self.addressing_mode {
				AddressingMode::SV32 => self.store_word_raw(pte_address, new_pte as u32),
				_ => self.store_doubleword_raw(pte_address, new_pte),
			};
		}

		match access_type {
			MemoryAccessType::Execute => {
				if x == 0 {
					return Err(());
				}
			}
			MemoryAccessType::Read => {
				if r == 0 {
					return Err(());
				}
			}
			MemoryAccessType::Write => {
				if w == 0 {
					return Err(());
				}
			}
			_ => {}
		};

		let offset = v_address & 0xfff; // [11:0]
								// @TODO: Optimize
		let p_address = match self.addressing_mode {
			AddressingMode::SV32 => match level {
				1 => {
					if ppns[0] != 0 {
						return Err(());
					}
					(ppns[1] << 22) | (vpns[0] << 12) | offset
				}
				0 => (ppn << 12) | offset,
				_ => panic!(), // Shouldn't happen
			},
			_ => match level {
				2 => {
					if ppns[1] != 0 || ppns[0] != 0 {
						return Err(());
					}
					(ppns[2] << 30) | (vpns[1] << 21) | (vpns[0] << 12) | offset
				}
				1 => {
					if ppns[0] != 0 {
						return Err(());
					}
					(ppns[2] << 30) | (ppns[1] << 21) | (vpns[0] << 12) | offset
				}
				0 => (ppn << 12) | offset,
				_ => panic!(), // Shouldn't happen
			},
		};

		// println!("PA:{:X}", p_address);
		Ok(p_address)
	}
}

/// [`Memory`](../memory/struct.Memory.html) wrapper. Converts physical address to the one in memory
/// using [`DRAM_BASE`](constant.DRAM_BASE.html) and accesses [`Memory`](../memory/struct.Memory.html).
pub struct MemoryWrapper {
	pub memory: Memory,
}

impl MemoryWrapper {
	fn new() -> Self {
		MemoryWrapper {
			memory: Memory::new(),
		}
	}

	fn init(&mut self, capacity: u64) {
		self.memory.init(capacity);
	}

	pub fn read_byte(&mut self, p_address: u64) -> u8 {
		debug_assert!(
			p_address >= DRAM_BASE,
			"Memory address must equals to or bigger than DRAM_BASE. {:X}",
			p_address
		);
		self.memory.read_byte(p_address - DRAM_BASE)
	}

	pub fn read_halfword(&mut self, p_address: u64) -> u16 {
		debug_assert!(
			p_address >= DRAM_BASE && p_address.wrapping_add(1) >= DRAM_BASE,
			"Memory address must equals to or bigger than DRAM_BASE. {:X}",
			p_address
		);
		self.memory.read_halfword(p_address - DRAM_BASE)
	}

	pub fn read_word(&mut self, p_address: u64) -> u32 {
		debug_assert!(
			p_address >= DRAM_BASE && p_address.wrapping_add(3) >= DRAM_BASE,
			"Memory address must equals to or bigger than DRAM_BASE. {:X}",
			p_address
		);
		self.memory.read_word(p_address - DRAM_BASE)
	}

	pub fn read_doubleword(&mut self, p_address: u64) -> u64 {
		debug_assert!(
			p_address >= DRAM_BASE && p_address.wrapping_add(7) >= DRAM_BASE,
			"Memory address must equals to or bigger than DRAM_BASE. {:X}",
			p_address
		);
		self.memory.read_doubleword(p_address - DRAM_BASE)
	}

	pub fn write_byte(&mut self, p_address: u64, value: u8) {
		debug_assert!(
			p_address >= DRAM_BASE,
			"Memory address must equals to or bigger than DRAM_BASE. {:X}",
			p_address
		);
		self.memory.write_byte(p_address - DRAM_BASE, value)
	}

	pub fn write_halfword(&mut self, p_address: u64, value: u16) {
		debug_assert!(
			p_address >= DRAM_BASE && p_address.wrapping_add(1) >= DRAM_BASE,
			"Memory address must equals to or bigger than DRAM_BASE. {:X}",
			p_address
		);
		self.memory.write_halfword(p_address - DRAM_BASE, value)
	}

	pub fn write_word(&mut self, p_address: u64, value: u32) {
		debug_assert!(
			p_address >= DRAM_BASE && p_address.wrapping_add(3) >= DRAM_BASE,
			"Memory address must equals to or bigger than DRAM_BASE. {:X}",
			p_address
		);
		self.memory.write_word(p_address - DRAM_BASE, value)
	}

	pub fn write_doubleword(&mut self, p_address: u64, value: u64) {
		debug_assert!(
			p_address >= DRAM_BASE && p_address.wrapping_add(7) >= DRAM_BASE,
			"Memory address must equals to or bigger than DRAM_BASE. {:X}",
			p_address
		);
		self.memory.write_doubleword(p_address - DRAM_BASE, value)
	}

	pub fn validate_address(&self, address: u64) -> bool {
		self.memory.validate_address(address - DRAM_BASE)
	}
}
