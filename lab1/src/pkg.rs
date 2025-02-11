#![allow(unused)]
// use crate::utils::*;
use fnv::FnvHashMap;
use riscv_emu_rust::cpu::*;
use riscv_emu_rust::l1cache::*;
use riscv_emu_rust::l2cache::*;
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
		privilege_mode: PrivilegeMode::User,
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
			privilege_mode: PrivilegeMode::User,
			memory: MemoryWrapper {
				memory: Memory { data: vec![] },
			},
			l1_cache: L1Cache::static_new(),
			l2_cache: L2Cache::static_new(),

			memory_access_trace: vec![],

			mstatus: 0,
			tlb_tag: [0; TLB_ENTRY_NUM],
			tlb_value: [0; TLB_ENTRY_NUM],
			tlb_bitnum: 0,

			dram_latency: 0,
		},
		reservation: 0,
		is_reservation_set: false,
		_dump_flag: false,
		unsigned_data_mask: 0xffffffffffffffff,
		tohost_addr: 0,

		exit_signal: false,
	},

	symbol_map: None,
	format_map: None,
	fu_map: None,
	op_map: None,

	// These can be updated in setup_program()
	is_test: false,
	tohost_addr: 0,

	run_time: 0.0,
};

pub const COSIM_INSTRUCTIONS: [&'static str; 75] = [
	"ADD", "ADDI", "ADDIW", "ADDW", "AND", "ANDI", "AUIPC", "BEQ", "BGE", "BGEU", "BLT", "BLTU",
	"BNE", "CSRRC", "CSRRCI", "CSRRS", "CSRRW", "CSRRWI", "CSRRSI", "DIV", "DIVU", "DIVUW", "DIVW",
	"EBREAK", "ECALL", "FENCE", "FENCE.I", "JAL", "JALR", "LB", "LBU", "LD", "LH", "LHU", "LUI",
	"LW", "LWU", "MRET", "MUL", "MULH", "MULHSU", "MULHU", "MULW", "OR", "ORI", "REM", "REMU",
	"REMUW", "REMW", "SB", "SD", "SH", "SLL", "SLLI", "SLLIW", "SLLW", "SLT", "SLTI", "SLTIU",
	"SLTU", "SRA", "SRAI", "SRAIW", "SRAW", "SRET", "SRL", "SRLI", "SRLIW", "SRLW", "SUB", "SUBW",
	"SW", "WFI", "XOR", "XORI",
];

pub const COSIM_INSTRUCTIONS_FORMAT: [&'static str; 75] = [
	"R", "I", "I", "R", "R", "I", "U", "B", "B", "B", "B", "B", "B", "I", "I", "I", "I", "I", "I",
	"R", "R", "R", "R", "I", "I", "I", "I", "J", "I", "I", "I", "I", "I", "I", "U", "I", "I", "R",
	"R", "R", "R", "R", "R", "R", "I", "R", "R", "R", "R", "S", "S", "S", "R", "I", "I", "R", "R",
	"I", "I", "R", "R", "I", "I", "R", "R", "R", "I", "I", "R", "R", "R", "S", "R", "R", "I",
];

pub const COSIM_INSTRUCTIONS_FU_T: [u8; 75] = [
	FU_ALU,
	FU_ALU,
	FU_ALU,
	FU_ALU,
	FU_ALU,
	FU_ALU,
	FU_ALU,
	FU_CTRL_FLOW,
	FU_CTRL_FLOW,
	FU_CTRL_FLOW,
	FU_CTRL_FLOW,
	FU_CTRL_FLOW,
	FU_CTRL_FLOW,
	FU_CSR,
	FU_CSR,
	FU_CSR,
	FU_CSR,
	FU_CSR,
	FU_CSR,
	FU_MULT,
	FU_MULT,
	FU_MULT,
	FU_MULT,
	FU_NONE,
	FU_NONE,
	FU_NONE,
	FU_NONE,
	FU_CTRL_FLOW,
	FU_CTRL_FLOW,
	FU_LOAD,
	FU_LOAD,
	FU_LOAD,
	FU_LOAD,
	FU_LOAD,
	FU_ALU,
	FU_LOAD,
	FU_LOAD,
	FU_NONE,
	FU_MULT,
	FU_MULT,
	FU_MULT,
	FU_MULT,
	FU_MULT,
	FU_ALU,
	FU_ALU,
	FU_MULT,
	FU_MULT,
	FU_MULT,
	FU_MULT,
	FU_STORE,
	FU_STORE,
	FU_STORE,
	FU_ALU,
	FU_ALU,
	FU_ALU,
	FU_ALU,
	FU_ALU,
	FU_ALU,
	FU_ALU,
	FU_ALU,
	FU_ALU,
	FU_ALU,
	FU_ALU,
	FU_ALU,
	FU_NONE,
	FU_ALU,
	FU_ALU,
	FU_ALU,
	FU_ALU,
	FU_ALU,
	FU_ALU,
	FU_STORE,
	FU_NONE,
	FU_ALU,
	FU_ALU,
];

pub const COSIM_INSTRUCTIONS_FU_OP: [u8; 75] = [
	OP_ADD,
	OP_ADD,
	OP_ADDW,
	OP_ADDW,
	OP_ANDL,
	OP_ANDL,
	OP_ADD,
	OP_EQ,
	OP_GES,
	OP_GEU,
	OP_LTS,
	OP_LTU,
	OP_NE,
	OP_CSR_READ,
	OP_CSR_READ,
	OP_CSR_READ,
	OP_CSR_WRITE,
	OP_CSR_WRITE,
	OP_CSR_READ,
	OP_DIV,
	OP_DIVU,
	OP_DIVUW,
	OP_DIVW,
	OP_ADD, //EBREAK
	OP_ECALL,
	OP_FENCE,
	OP_FENCE_I,
	OP_ADD, //JAL
	OP_JALR,
	OP_LB,
	OP_LBU,
	OP_LD,
	OP_LH,
	OP_LHU,
	OP_ADD,
	OP_LW,
	OP_LWU,
	OP_MRET,
	OP_MUL,
	OP_MULH,
	OP_MULHSU,
	OP_MULHU,
	OP_MULW,
	OP_ORL,
	OP_ORL,
	OP_REM,
	OP_REMU,
	OP_REMUW,
	OP_REMW,
	OP_SB,
	OP_SD,
	OP_SH,
	OP_SLL,
	OP_SLL,
	OP_SLLW,
	OP_SLLW,
	OP_SLTS,
	OP_SLTS,
	OP_SLTU,
	OP_SLTU,
	OP_SRA,
	OP_SRA,
	OP_SRAW,
	OP_SRAW,
	OP_SRET,
	OP_SRL,
	OP_SRL,
	OP_SRLW,
	OP_SRLW,
	OP_SUB,
	OP_SUBW,
	OP_SW,
	OP_WFI,
	OP_XORL,
	OP_XORL,
];

/// Configurable globals
// ISA globals
pub const XLEN: usize = 64;
pub const VLEN: usize = 39;
pub const PLEN: usize = 56;
pub const REG_ADDR_SIZE: usize = 6;
// Architectural globals
pub const NR_SB_ENTRIES: usize = 8;
pub const TRANS_ID_BITS: usize = 3; // log2(NR_SB_ENTRIES)
pub const ISSUE_NUM: usize = 1;
// Impl globals
pub const LEN_FU: usize = 4;
pub const LEN_OP: usize = 7;
pub const LEN_CF: usize = 3;

// FU_T ENUMs
pub const FU_NONE: u8 = 0;
pub const FU_LOAD: u8 = 1;
pub const FU_STORE: u8 = 2;
pub const FU_ALU: u8 = 3;
pub const FU_CTRL_FLOW: u8 = 4;
pub const FU_MULT: u8 = 5;
pub const FU_CSR: u8 = 6;

// FU_OP ENUMs
pub const OP_ADD: u8 = 0;
pub const OP_SUB: u8 = 1;
pub const OP_ADDW: u8 = 2;
pub const OP_SUBW: u8 = 3;
pub const OP_XORL: u8 = 4;
pub const OP_ORL: u8 = 5;
pub const OP_ANDL: u8 = 6;
pub const OP_SRA: u8 = 7;
pub const OP_SRL: u8 = 8;
pub const OP_SLL: u8 = 9;
pub const OP_SRLW: u8 = 10;
pub const OP_SLLW: u8 = 11;
pub const OP_SRAW: u8 = 12;
pub const OP_LTS: u8 = 13;
pub const OP_LTU: u8 = 14;
pub const OP_GES: u8 = 15;
pub const OP_GEU: u8 = 16;
pub const OP_EQ: u8 = 17;
pub const OP_NE: u8 = 18;
pub const OP_JALR: u8 = 19;
pub const OP_BRANCH: u8 = 20;
pub const OP_SLTS: u8 = 21;
pub const OP_SLTU: u8 = 22;
pub const OP_MRET: u8 = 23;
pub const OP_SRET: u8 = 24;
pub const OP_DRET: u8 = 25;
pub const OP_ECALL: u8 = 26;
pub const OP_WFI: u8 = 27;
pub const OP_FENCE: u8 = 28;
pub const OP_FENCE_I: u8 = 29;
pub const OP_SFENCE_VMA: u8 = 30;
pub const OP_CSR_WRITE: u8 = 31;
pub const OP_CSR_READ: u8 = 32;
pub const OP_CSR_SET: u8 = 33;
pub const OP_CSR_CLEAR: u8 = 34;
pub const OP_LD: u8 = 35;
pub const OP_SD: u8 = 36;
pub const OP_LW: u8 = 37;
pub const OP_LWU: u8 = 38;
pub const OP_SW: u8 = 39;
pub const OP_LH: u8 = 40;
pub const OP_LHU: u8 = 41;
pub const OP_SH: u8 = 42;
pub const OP_LB: u8 = 43;
pub const OP_SB: u8 = 44;
pub const OP_LBU: u8 = 45;
pub const OP_MUL: u8 = 46;
pub const OP_MULH: u8 = 47;
pub const OP_MULHU: u8 = 48;
pub const OP_MULHSU: u8 = 49;
pub const OP_MULW: u8 = 50;
pub const OP_DIV: u8 = 51;
pub const OP_DIVU: u8 = 52;
pub const OP_DIVW: u8 = 53;
pub const OP_DIVUW: u8 = 54;
pub const OP_REM: u8 = 55;
pub const OP_REMU: u8 = 56;
pub const OP_REMW: u8 = 57;
pub const OP_REMUW: u8 = 58;

pub const CF_T_NO_CF: u8 = 0;
pub const CF_T_BRANCH: u8 = 1;
pub const CF_T_JUMP: u8 = 2;
pub const CF_T_JUMP_R: u8 = 3;
pub const CF_T_RETURN: u8 = 4;

// XLEN * 2 + 1
pub struct ExceptionT {
	pub cause: [bool; XLEN],
	pub tval: [bool; XLEN],
	pub valid: bool,
}

pub const OFFSET_EXCEPTION_VALID: usize = 0;
pub const OFFSET_EXCEPTION_TVAL: usize = OFFSET_EXCEPTION_VALID + 1;
pub const OFFSET_EXCEPTION_CAUSE: usize = OFFSET_EXCEPTION_TVAL + XLEN;
pub const LEN_EXCEPTION: usize = OFFSET_EXCEPTION_CAUSE + XLEN;

// LEN_CF_T + VLEN
pub struct BranchpredictSbeT {
	pub cf: [bool; LEN_CF],
	pub predict_address: [bool; VLEN],
}

pub const OFFSET_BP_PREDICT_ADDRESS: usize = 0;
pub const OFFSET_BP_CF: usize = OFFSET_BP_PREDICT_ADDRESS + VLEN;
pub const LEN_BP: usize = OFFSET_BP_CF + LEN_CF;

// VLEN + TRANS_ID_BITS + LEN_FU_T + LEN_FU_OP + REG_ADDR_SIZE * 3
// + XLEN + 4 + (XLEN * 2 + 1) + (LEN_CF_T + VLEN) + 1
pub struct ScoreboardEntryT {
	pub pc: [bool; VLEN],
	pub trans_id: [bool; TRANS_ID_BITS],
	pub fu: [bool; LEN_FU],
	pub op: [bool; LEN_OP],
	pub rs1: [bool; REG_ADDR_SIZE],
	pub rs2: [bool; REG_ADDR_SIZE],
	pub rd: [bool; REG_ADDR_SIZE],
	pub result: [bool; XLEN],
	pub valid: bool,
	pub use_imm: bool,
	pub use_zimm: bool,
	pub use_pc: bool,
	pub ex: ExceptionT,
	pub bp: BranchpredictSbeT,
	pub is_compressed: bool,
}

pub const OFFSET_PC: usize = OFFSET_TRANS_ID + TRANS_ID_BITS;
pub const OFFSET_TRANS_ID: usize = OFFSET_FU + LEN_FU;
pub const OFFSET_FU: usize = OFFSET_OP + LEN_OP;
pub const OFFSET_OP: usize = OFFSET_RS1 + REG_ADDR_SIZE;
pub const OFFSET_RS1: usize = OFFSET_RS2 + REG_ADDR_SIZE;
pub const OFFSET_RS2: usize = OFFSET_RD + REG_ADDR_SIZE;
pub const OFFSET_RD: usize = OFFSET_RESULT + XLEN;
pub const OFFSET_RESULT: usize = OFFSET_VALID + 1;
pub const OFFSET_VALID: usize = OFFSET_USE_IMM + 1;
pub const OFFSET_USE_IMM: usize = OFFSET_USE_ZIMM + 1;
pub const OFFSET_USE_ZIMM: usize = OFFSET_USE_PC + 1;
pub const OFFSET_USE_PC: usize = OFFSET_EX + LEN_EXCEPTION;
pub const OFFSET_EX: usize = OFFSET_BP + LEN_BP;
pub const OFFSET_BP: usize = OFFSET_IS_COMPRESSED + 1;
pub const OFFSET_IS_COMPRESSED: usize = 0;

pub const LEN_SCOREBOARD_ENTRY: usize = OFFSET_PC + VLEN;

/// 0              1       ...               ...   312   LEN(313)
/// [is_ctrl_flow, sbe.is_compressed, ... , sbe.pc, valid]
pub const OFFSET_SCOREBOARD_ENTRY: usize = 1;

pub struct IdPerIssueT {
	pub valid: bool,
	pub sbe: ScoreboardEntryT,
	pub is_ctrl_flow: bool,
}

pub struct InstId2IsT {
	issue_inst: [IdPerIssueT; ISSUE_NUM],
}

pub const ID2IS_LEN: usize = (1
	+ (VLEN
		+ TRANS_ID_BITS
		+ LEN_FU + LEN_OP
		+ REG_ADDR_SIZE * 3
		+ XLEN + 4
		+ (XLEN * 2 + 1)
		+ (LEN_CF + VLEN)
		+ 1) + 1)
	* ISSUE_NUM;

// this is a const, e.g. 320 array write 313 flush to right, make it be 7
