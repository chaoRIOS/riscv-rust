use cpu::Instruction;

pub const ROB_SIZE: usize = 32;

/// Slot for RS in an ROB Entry
#[derive(Copy, Clone)]
pub struct RsSlot {
	// Is data present in slot
	pub present: bool,
	// Index of a ROBEntry
	pub tag: usize,
	// Data in ROB
	pub data: u64,
}

/// Slot for RD in an ROB Entry
#[derive(Copy, Clone)]
pub struct RdSlot {
	// Is data present in slot
	pub present: bool,
	// Number of architectural registers
	pub register_number: usize,
	// Data in ROB
	pub data: u64,
	// Exception
	pub exception: bool,
}

#[derive(Copy, Clone)]
pub struct ROBEntry {
	// Valid bit
	pub valid: bool,
	// Is this instruction in-flight
	pub issued: bool,
	// Index of instruction
	pub instruction: Instruction,
	// Rs1
	pub rs1: RsSlot,
	// Rs2
	pub rs2: RsSlot,
	// Rd
	pub rd: RdSlot,
}

pub struct ReOrderBuffer {
	// Oldest pointer
	pub oldest: usize,
	// Free pointer
	pub free: usize,
	// Entries
	pub entries: [ROBEntry; ROB_SIZE],
}

impl RsSlot {
	pub fn new() -> RsSlot {
		RsSlot {
			present: false,
			tag: 0,
			data: 0,
		}
	}

	pub const fn static_new() -> RsSlot {
		RsSlot {
			present: false,
			tag: 0,
			data: 0,
		}
	}
}

impl RdSlot {
	pub fn new() -> RdSlot {
		RdSlot {
			present: false,
			register_number: 0,
			data: 0,
			exception: false,
		}
	}

	pub const fn static_new() -> RdSlot {
		RdSlot {
			present: false,
			register_number: 0,
			data: 0,
			exception: false,
		}
	}
}

impl ROBEntry {
	pub fn new() -> ROBEntry {
		ROBEntry {
			valid: false,
			issued: false,
			instruction: Instruction::new(),
			rs1: RsSlot::new(),
			rs2: RsSlot::new(),
			rd: RdSlot::new(),
		}
	}
}

impl ReOrderBuffer {
	pub fn new() -> ReOrderBuffer {
		ReOrderBuffer {
			oldest: 0,
			free: 0,
			entries: [ROBEntry::new(); ROB_SIZE],
		}
	}
}
