#[allow(unused)]
use rand::random;

use l1cache::{PlacementPolicy, L1_CACHE_SIZE};

/// 64B cache block size
pub const L2_CACHE_BLOCK_SIZE: i32 = 64;

/// 8*L1 L2 cache size
pub const L2_CACHE_SIZE: i32 = 4 * L1_CACHE_SIZE;

/// 8-way set-associative
pub const L2_SET_ASSOCIATIVE_WAY: i32 = 8;

/// number of sets
/// 1024 sets
pub const L2_CACHE_SET_NUMBER: i32 = L2_CACHE_SIZE / (L2_CACHE_BLOCK_SIZE * L2_SET_ASSOCIATIVE_WAY);

/// L2 cache line format
pub const L2_CACHE_OFFSET_BITS: i32 = 6;
pub const L2_CACHE_INDEX_BITS: i32 = 6 + 3;
pub const L2_CACHE_TAG_BITS: i32 = 32 - L2_CACHE_OFFSET_BITS - L2_CACHE_INDEX_BITS;

/// 64B cache block size
pub const L2_CACHE_HIT_LATENCY: i32 = 1;
pub const L2_CACHE_MISS_LATENCY: i32 = 2;

#[derive(Copy, Clone)]
pub struct L2CacheLine {
	pub l1_inclusive: bool,
	pub valid: bool,
	pub tag: u64,
	pub data_blocks: [u8; L2_CACHE_BLOCK_SIZE as usize],
}
impl L2CacheLine {
	pub fn new() -> Self {
		L2CacheLine {
			l1_inclusive: false,
			valid: false,
			tag: 0 as u64,
			data_blocks: [0 as u8; L2_CACHE_BLOCK_SIZE as usize],
		}
	}

	pub fn get(&self, offset: u64, width: u64) -> u64 {
		let mut value: u64 = 0;
		assert_eq!((width > 0) && (width <= 64), true);
		assert_eq!(offset + width <= 64, true);
		for _width in 0..width {
			value = value | ((self.data_blocks[(offset + _width) as usize] as u64) << (_width * 8))
		}
		value & ((1u128 << (width * 8)) - 1) as u64
	}

	pub fn set(&mut self, offset: u64, width: u64, value: u64) {
		assert_eq!((width > 0) && (width <= 64), true);
		assert_eq!(offset + width <= 64, true);
		for _width in 0..width {
			self.data_blocks[(offset + _width) as usize] =
				((value >> (_width * 8)) & ((1 << 8) - 1)) as u8;
		}
	}
}

#[derive(Copy, Clone)]
pub struct L2CacheSet {
	pub data: [L2CacheLine; L2_SET_ASSOCIATIVE_WAY as usize],
}
impl L2CacheSet {
	pub fn new() -> Self {
		L2CacheSet {
			data: [L2CacheLine::new(); L2_SET_ASSOCIATIVE_WAY as usize],
		}
	}
}

#[derive(Clone)]
pub struct L2Cache {
	pub data:
		[L2CacheSet; (L2_CACHE_SIZE / (L2_CACHE_BLOCK_SIZE * L2_SET_ASSOCIATIVE_WAY)) as usize],
	pub hit_num: u64,
	pub miss_num: u64,
}

impl L2Cache {
	pub fn new() -> Self {
		L2Cache {
			data: [L2CacheSet::new(); L2_CACHE_SET_NUMBER as usize],
			hit_num: 0,
			miss_num: 0,
		}
	}

	/// Read 1 line from cache
	///
	/// # Arguments
	/// * `tag`: tag for line matching
	/// * `index`: index for set selecting
	fn read_line_raw(&self, tag: u64, index: u64) -> Result<L2CacheLine, ()> {
		// index the set
		let mut _l2_cache_set = self.data[index as usize];
		// traverse the set
		for _way in 0..L2_SET_ASSOCIATIVE_WAY {
			let _line = _l2_cache_set.data[_way as usize];
			if (_line.tag == tag) && (_line.valid == true) {
				// hit
				#[cfg(feature = "debug-cache")]
				println!("Hit [{}][{}]", index, _way);
				return Ok(_line);
			}
		}
		// @TODO: miss
		Err(())
	}

	/// Read 1 line from cache
	///
	/// # Arguments
	/// * `tag`: tag for line matching
	/// * `index`: index for set selecting
	fn read_line_info_raw(&self, tag: u64, index: u64) -> Result<u64, ()> {
		// index the set
		let mut _l2_cache_set = self.data[index as usize];
		// traverse the set
		for _way in 0..L2_SET_ASSOCIATIVE_WAY {
			let _line = _l2_cache_set.data[_way as usize];
			if (_line.tag == tag) && (_line.valid == true) {
				// hit
				#[cfg(feature = "debug-cache")]
				println!("Hit [{}][{}]", index, _way);
				return Ok(_way as u64);
			}
		}
		// @TODO: miss
		Err(())
	}

	/// Public interface to read 1 line from cache
	///
	/// # Arguments
	/// * `p_address`: physical address
	pub fn read_line(&mut self, p_address: u64) -> Result<L2CacheLine, ()> {
		let tag: u64 = (p_address >> (L2_CACHE_OFFSET_BITS + L2_CACHE_INDEX_BITS))
			& ((1 << L2_CACHE_TAG_BITS) - 1);
		let index: u64 = (p_address >> L2_CACHE_OFFSET_BITS) & ((1 << L2_CACHE_INDEX_BITS) - 1);
		// println!(
		// 	"{:x}({:b}): [{:b}|{:b}|{:b}]",
		// 	p_address,
		// 	p_address,
		// 	tag,
		// 	index,
		// 	p_address & ((1 << L2_CACHE_OFFSET_BITS) - 1)
		// );

		match self.read_line_raw(tag, index) {
			// hit
			Ok(cache_line) => {
				#[cfg(feature = "debug-cache")]
				println!("Hit {:x}", p_address);
				Ok(cache_line)
			}
			// miss
			_ => {
				#[cfg(feature = "debug-cache")]
				println!("Miss {:x}", p_address);
				Err(())
			}
		}
	}

	/// Public interface to read way index of 1 line from cache
	///
	/// # Arguments
	/// * `p_address`: physical address
	pub fn read_line_info(&mut self, p_address: u64) -> Result<u64, ()> {
		let tag: u64 = (p_address >> (L2_CACHE_OFFSET_BITS + L2_CACHE_INDEX_BITS))
			& ((1 << L2_CACHE_TAG_BITS) - 1);
		let index: u64 = (p_address >> L2_CACHE_OFFSET_BITS) & ((1 << L2_CACHE_INDEX_BITS) - 1);

		match self.read_line_info_raw(tag, index) {
			// hit
			Ok(_way) => {
				#[cfg(feature = "debug-cache")]
				println!("Hit {:x}", p_address);
				Ok(_way)
			}
			// miss
			_ => {
				#[cfg(feature = "debug-cache")]
				println!("Miss {:x}", p_address);
				Err(())
			}
		}
	}

	/// Placement policy
	/// supporting Random, LRU and FIFO
	///
	/// # Arguments
	/// * `index`: index of cache set
	pub fn allocate_new_line(&self, index: u64, policy: PlacementPolicy) -> u8 {
		match policy {
			PlacementPolicy::Random => {
				let mut non_inclusive_ways = vec![];
				for way in 0..L2_SET_ASSOCIATIVE_WAY {
					if self.data[index as usize].data[way as usize].l1_inclusive == false {
						non_inclusive_ways.push(way);
					}
				}
				non_inclusive_ways[(random::<u8>() % (non_inclusive_ways.len() as u8)) as usize]
					as u8
			}
			PlacementPolicy::LRU => {
				// @TODO: LRU
				0
			}
			PlacementPolicy::FIFO => {
				// @TODO: FIFO
				0
			}
		}
	}
}
