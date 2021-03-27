#![allow(unused)]
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
		instruction_buffer: vec![],
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

/// Configurable globals
// ISA globals
const XLEN: usize = 64;
const VLEN: usize = 39;
const PLEN: usize = 56;
const REG_ADDR_SIZE: usize = 6;
// Architectural globals
const NR_SB_ENTRIES: usize = 8;
const TRANS_ID_BITS: usize = 3; // log2(NR_SB_ENTRIES)
const ISSUE_NUM: usize = 2;
// Impl globals
const LEN_FU_T: usize = 4;
const LEN_FU_OP: usize = 7;
const LEN_CF_T: usize = 3;

// FU_T ENUMs
const FU_T_NONE: u8 = 0;
const FU_T_LOAD: u8 = 1;
const FU_T_STORE: u8 = 2;
const FU_T_ALU: u8 = 3;
const FU_T_CTRL_FLOW: u8 = 4;
const FU_T_MULT: u8 = 5;
const FU_T_CSR: u8 = 6;

// FU_OP ENUMs
const FU_OP_ADD: u8 = 0;
const FU_OP_SUB: u8 = 1;
const FU_OP_ADDW: u8 = 2;
const FU_OP_SUBW: u8 = 3;
const FU_OP_XORL: u8 = 4;
const FU_OP_ORL: u8 = 5;
const FU_OP_ANDL: u8 = 6;
const FU_OP_SRA: u8 = 7;
const FU_OP_SRL: u8 = 8;
const FU_OP_SLL: u8 = 9;
const FU_OP_SRLW: u8 = 10;
const FU_OP_SLLW: u8 = 11;
const FU_OP_SRAW: u8 = 12;
const FU_OP_LTS: u8 = 13;
const FU_OP_LTU: u8 = 14;
const FU_OP_GES: u8 = 15;
const FU_OP_GEU: u8 = 16;
const FU_OP_EQ: u8 = 17;
const FU_OP_NE: u8 = 18;
const FU_OP_JALR: u8 = 19;
const FU_OP_BRANCH: u8 = 20;
const FU_OP_SLTS: u8 = 21;
const FU_OP_SLTU: u8 = 22;
const FU_OP_MRET: u8 = 23;
const FU_OP_SRET: u8 = 24;
const FU_OP_DRET: u8 = 25;
const FU_OP_ECALL: u8 = 26;
const FU_OP_WFI: u8 = 27;
const FU_OP_FENCE: u8 = 28;
const FU_OP_FENCE_I: u8 = 29;
const FU_OP_SFENCE_VMA: u8 = 30;
const FU_OP_CSR_WRITE: u8 = 31;
const FU_OP_CSR_READ: u8 = 32;
const FU_OP_CSR_SET: u8 = 33;
const FU_OP_CSR_CLEAR: u8 = 34;
const FU_OP_LD: u8 = 35;
const FU_OP_SD: u8 = 36;
const FU_OP_LW: u8 = 37;
const FU_OP_LWU: u8 = 38;
const FU_OP_SW: u8 = 39;
const FU_OP_LH: u8 = 40;
const FU_OP_LHU: u8 = 41;
const FU_OP_SH: u8 = 42;
const FU_OP_LB: u8 = 43;
const FU_OP_SB: u8 = 44;
const FU_OP_LBU: u8 = 45;
const FU_OP_MUL: u8 = 46;
const FU_OP_MULH: u8 = 47;
const FU_OP_MULHU: u8 = 48;
const FU_OP_MULHSU: u8 = 49;
const FU_OP_MULW: u8 = 50;
const FU_OP_DIV: u8 = 51;
const FU_OP_DIVU: u8 = 52;
const FU_OP_DIVW: u8 = 53;
const FU_OP_DIVUW: u8 = 54;
const FU_OP_REM: u8 = 55;
const FU_OP_REMU: u8 = 56;
const FU_OP_REMW: u8 = 57;
const FU_OP_REMUW: u8 = 58;

const CF_T_NO_CF: u8 = 0;
const CF_T_BRANCH: u8 = 1;
const CF_T_JUMP: u8 = 2;
const CF_T_JUMP_R: u8 = 3;
const CF_T_RETURN: u8 = 4;

#[repr(C, packed)]
// XLEN * 2 + 1
pub struct exception_t {
	pub cause: [bool; XLEN],
	pub tval: [bool; XLEN],
	pub valid: bool,
}

#[repr(C, packed)]
// LEN_CF_T + VLEN
pub struct branchpredict_sbe_t {
	pub cf: [bool; LEN_CF_T],
	pub predict_address: [bool; VLEN],
}

#[repr(C, packed)]
// VLEN + TRANS_ID_BITS + LEN_FU_T + LEN_FU_OP + REG_ADDR_SIZE * 3
// + XLEN + 4 + (XLEN * 2 + 1) + (LEN_CF_T + VLEN) + 1
pub struct scoreboard_entry_t {
	pub pc: [bool; VLEN],
	pub trans_id: [bool; TRANS_ID_BITS],
	pub fu: [bool; LEN_FU_T],
	pub op: [bool; LEN_FU_OP],
	pub rs1: [bool; REG_ADDR_SIZE],
	pub rs2: [bool; REG_ADDR_SIZE],
	pub rd: [bool; REG_ADDR_SIZE],
	pub result: [bool; XLEN],
	pub valid: bool,
	pub use_imm: bool,
	pub use_zimm: bool,
	pub use_pc: bool,
	pub ex: exception_t,
	pub bp: branchpredict_sbe_t,
	pub is_compressed: bool,
}

#[repr(C, packed)]
// 1 + (VLEN + TRANS_ID_BITS + LEN_FU_T + LEN_FU_OP + REG_ADDR_SIZE * 3
// + XLEN + 4 + (XLEN * 2 + 1) + (LEN_CF_T + VLEN) + 1) + 1
pub struct id_per_issue_t {
	pub valid: bool,
	pub sbe: scoreboard_entry_t,
	pub is_ctrl_flow: bool,
}

#[repr(C, packed)]
// (1 + (VLEN + TRANS_ID_BITS + LEN_FU_T + LEN_FU_OP + REG_ADDR_SIZE * 3
// + XLEN + 4 + (XLEN * 2 + 1) + (LEN_CF_T + VLEN) + 1) + 1) * ISSUE_NUM
pub struct inst_id2is_t {
	issue_inst: [id_per_issue_t; ISSUE_NUM],
}

pub const ID2IS_LEN: usize = (1
	+ (VLEN
		+ TRANS_ID_BITS
		+ LEN_FU_T
		+ LEN_FU_OP
		+ REG_ADDR_SIZE * 3
		+ XLEN + 4
		+ (XLEN * 2 + 1)
		+ (LEN_CF_T + VLEN)
		+ 1) + 1)
	* ISSUE_NUM;
