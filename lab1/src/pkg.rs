use bitvec::prelude::*;
use fnv::FnvHashMap;
use riscv_emu_rust::cpu::*;
use riscv_emu_rust::memory::*;
use riscv_emu_rust::mmu::*;
use riscv_emu_rust::Emulator;
use std::fs::File;
use std::io::Read;
use std::{mem, slice};

/// GLOBAL EMULATOR.
pub static mut EMULATOR: Emulator = Emulator {
	cpu: Cpu {
		clock: 0,
		xlen: Xlen::Bit64,
		privilege_mode: PrivilegeMode::Machine,
		wfi: false,
		x: [0; 32],
		f: [0.0; 32],
		pc: 0,
		instruction_buffer: [0u32; 64],
		csr: [0; CSR_CAPACITY],
		mmu: Mmu {
			clock: 0,
			xlen: Xlen::Bit64,
			ppn: 0,
			addressing_mode: AddressingMode::None,
			privilege_mode: PrivilegeMode::Machine,
			memory: MemoryWrapper {
				memory: Memory { data: vec![] },
			},
			mstatus: 0,
		},
		reservation: 0,
		is_reservation_set: false,
		_dump_flag: false,
		unsigned_data_mask: 0xffffffffffffffff,
	},

	symbol_map: None,

	// These can be updated in setup_program()
	is_test: false,
	tohost_addr: 0,
};

pub enum fu_t {
	NONE,      // 0
	LOAD,      // 1
	STORE,     // 2
	ALU,       // 3
	CTRL_FLOW, // 4
	MULT,      // 5
	CSR,       // 6
}
pub enum fu_op {
	// basic ALU op
	ADD,
	SUB,
	ADDW,
	SUBW,
	// logic operations
	XORL,
	ORL,
	ANDL,
	// shifts
	SRA,
	SRL,
	SLL,
	SRLW,
	SLLW,
	SRAW,
	// comparisons
	LTS,
	LTU,
	GES,
	GEU,
	EQ,
	NE,
	// jumps
	JALR,
	BRANCH,
	// set lower than operations
	SLTS,
	SLTU,
	// CSR functions
	MRET,
	SRET,
	DRET,
	ECALL,
	WFI,
	FENCE,
	FENCE_I,
	SFENCE_VMA,
	CSR_WRITE,
	CSR_READ,
	CSR_SET,
	CSR_CLEAR,
	// LSU functions
	LD,
	SD,
	LW,
	LWU,
	SW,
	LH,
	LHU,
	SH,
	LB,
	SB,
	LBU,
	// Multiplications
	MUL,
	MULH,
	MULHU,
	MULHSU,
	MULW,
	// Divisions
	DIV,
	DIVU,
	DIVW,
	DIVUW,
	REM,
	REMU,
	REMW,
	REMUW,
}
pub enum cf_t {
	NoCF,   // No control flow prediction
	Branch, // Branch
	Jump,   // Jump to address from immediate
	JumpR,  // Jump to address from registers
	Return, // Return Address Prediction
}
#[repr(C, packed)]
pub struct exception_t {
	pub cause: [bool; 64],
	pub tval: [bool; 64],
	pub valid: bool,
}
#[repr(C, packed)]
pub struct branchpredict_sbe_t {
	pub cf: cf_t,
	pub predict_address: [bool; 39],
}
#[repr(C, packed)]
pub struct scoreboard_entry_t {
	// pub pc: [bool; 64],
	// pub trans_id: [bool; 3],
	// pub fu: fu_t,
	// pub op: fu_op,
	// pub rs1: u32,
	// pub rs2: [bool; 6],
	// pub rd: [bool; 6],
	// pub result: u64,
	pub valid: bool,
	pub use_imm: bool,
	pub use_zimm: bool,
	pub use_pc: bool,
	// pub ex: exception_t,
	// pub bp: branchpredict_sbe_t,
	pub is_compressed: bool,
}
