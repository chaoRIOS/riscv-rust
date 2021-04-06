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
	format_map: None,
	fu_map: None,
	op_map: None,

	// These can be updated in setup_program()
	is_test: false,
	tohost_addr: 0,
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
	FU_T_ALU,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_CTRL_FLOW,
	FU_T_CTRL_FLOW,
	FU_T_CTRL_FLOW,
	FU_T_CTRL_FLOW,
	FU_T_CTRL_FLOW,
	FU_T_CTRL_FLOW,
	FU_T_CSR,
	FU_T_CSR,
	FU_T_CSR,
	FU_T_CSR,
	FU_T_CSR,
	FU_T_CSR,
	FU_T_MULT,
	FU_T_MULT,
	FU_T_MULT,
	FU_T_MULT,
	FU_T_NONE,
	FU_T_NONE,
	FU_T_NONE,
	FU_T_NONE,
	FU_T_CTRL_FLOW,
	FU_T_CTRL_FLOW,
	FU_T_LOAD,
	FU_T_LOAD,
	FU_T_LOAD,
	FU_T_LOAD,
	FU_T_LOAD,
	FU_T_ALU,
	FU_T_LOAD,
	FU_T_LOAD,
	FU_T_NONE,
	FU_T_MULT,
	FU_T_MULT,
	FU_T_MULT,
	FU_T_MULT,
	FU_T_MULT,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_MULT,
	FU_T_MULT,
	FU_T_MULT,
	FU_T_MULT,
	FU_T_STORE,
	FU_T_STORE,
	FU_T_STORE,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_NONE,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_ALU,
	FU_T_STORE,
	FU_T_NONE,
	FU_T_ALU,
	FU_T_ALU,
];

pub const COSIM_INSTRUCTIONS_FU_OP: [u8; 75] = [
	FU_OP_ADD,
	FU_OP_ADD,
	FU_OP_ADDW,
	FU_OP_ADDW,
	FU_OP_ANDL,
	FU_OP_ANDL,
	FU_OP_ADD,
	FU_OP_EQ,
	FU_OP_GES,
	FU_OP_GEU,
	FU_OP_LTS,
	FU_OP_LTU,
	FU_OP_NE,
	FU_OP_CSR_READ,
	FU_OP_CSR_READ,
	FU_OP_CSR_READ,
	FU_OP_CSR_WRITE,
	FU_OP_CSR_WRITE,
	FU_OP_CSR_READ,
	FU_OP_DIV,
	FU_OP_DIVU,
	FU_OP_DIVUW,
	FU_OP_DIVW,
	FU_OP_ADD, //EBREAK
	FU_OP_ECALL,
	FU_OP_FENCE,
	FU_OP_FENCE_I,
	FU_OP_JALR,
	FU_OP_JALR,
	FU_OP_LB,
	FU_OP_LBU,
	FU_OP_LD,
	FU_OP_LH,
	FU_OP_LHU,
	FU_OP_ADD,
	FU_OP_LW,
	FU_OP_LWU,
	FU_OP_MRET,
	FU_OP_MUL,
	FU_OP_MULH,
	FU_OP_MULHSU,
	FU_OP_MULHU,
	FU_OP_MULW,
	FU_OP_ORL,
	FU_OP_ORL,
	FU_OP_REM,
	FU_OP_REMU,
	FU_OP_REMUW,
	FU_OP_REMW,
	FU_OP_SB,
	FU_OP_SD,
	FU_OP_SH,
	FU_OP_SLL,
	FU_OP_SLL,
	FU_OP_SLLW,
	FU_OP_SLLW,
	FU_OP_SLTS,
	FU_OP_SLTS,
	FU_OP_SLTU,
	FU_OP_SLTU,
	FU_OP_SRA,
	FU_OP_SRA,
	FU_OP_SRAW,
	FU_OP_SRAW,
	FU_OP_SRET,
	FU_OP_SRL,
	FU_OP_SRL,
	FU_OP_SRLW,
	FU_OP_SRLW,
	FU_OP_SUB,
	FU_OP_SUBW,
	FU_OP_SW,
	FU_OP_WFI,
	FU_OP_XORL,
	FU_OP_XORL,
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
pub const LEN_FU_T: usize = 4;
pub const LEN_FU_OP: usize = 7;
pub const LEN_CF_T: usize = 3;

// FU_T ENUMs
pub const FU_T_NONE: u8 = 0;
pub const FU_T_LOAD: u8 = 1;
pub const FU_T_STORE: u8 = 2;
pub const FU_T_ALU: u8 = 3;
pub const FU_T_CTRL_FLOW: u8 = 4;
pub const FU_T_MULT: u8 = 5;
pub const FU_T_CSR: u8 = 6;

// FU_OP ENUMs
pub const FU_OP_ADD: u8 = 0;
pub const FU_OP_SUB: u8 = 1;
pub const FU_OP_ADDW: u8 = 2;
pub const FU_OP_SUBW: u8 = 3;
pub const FU_OP_XORL: u8 = 4;
pub const FU_OP_ORL: u8 = 5;
pub const FU_OP_ANDL: u8 = 6;
pub const FU_OP_SRA: u8 = 7;
pub const FU_OP_SRL: u8 = 8;
pub const FU_OP_SLL: u8 = 9;
pub const FU_OP_SRLW: u8 = 10;
pub const FU_OP_SLLW: u8 = 11;
pub const FU_OP_SRAW: u8 = 12;
pub const FU_OP_LTS: u8 = 13;
pub const FU_OP_LTU: u8 = 14;
pub const FU_OP_GES: u8 = 15;
pub const FU_OP_GEU: u8 = 16;
pub const FU_OP_EQ: u8 = 17;
pub const FU_OP_NE: u8 = 18;
pub const FU_OP_JALR: u8 = 19;
pub const FU_OP_BRANCH: u8 = 20;
pub const FU_OP_SLTS: u8 = 21;
pub const FU_OP_SLTU: u8 = 22;
pub const FU_OP_MRET: u8 = 23;
pub const FU_OP_SRET: u8 = 24;
pub const FU_OP_DRET: u8 = 25;
pub const FU_OP_ECALL: u8 = 26;
pub const FU_OP_WFI: u8 = 27;
pub const FU_OP_FENCE: u8 = 28;
pub const FU_OP_FENCE_I: u8 = 29;
pub const FU_OP_SFENCE_VMA: u8 = 30;
pub const FU_OP_CSR_WRITE: u8 = 31;
pub const FU_OP_CSR_READ: u8 = 32;
pub const FU_OP_CSR_SET: u8 = 33;
pub const FU_OP_CSR_CLEAR: u8 = 34;
pub const FU_OP_LD: u8 = 35;
pub const FU_OP_SD: u8 = 36;
pub const FU_OP_LW: u8 = 37;
pub const FU_OP_LWU: u8 = 38;
pub const FU_OP_SW: u8 = 39;
pub const FU_OP_LH: u8 = 40;
pub const FU_OP_LHU: u8 = 41;
pub const FU_OP_SH: u8 = 42;
pub const FU_OP_LB: u8 = 43;
pub const FU_OP_SB: u8 = 44;
pub const FU_OP_LBU: u8 = 45;
pub const FU_OP_MUL: u8 = 46;
pub const FU_OP_MULH: u8 = 47;
pub const FU_OP_MULHU: u8 = 48;
pub const FU_OP_MULHSU: u8 = 49;
pub const FU_OP_MULW: u8 = 50;
pub const FU_OP_DIV: u8 = 51;
pub const FU_OP_DIVU: u8 = 52;
pub const FU_OP_DIVW: u8 = 53;
pub const FU_OP_DIVUW: u8 = 54;
pub const FU_OP_REM: u8 = 55;
pub const FU_OP_REMU: u8 = 56;
pub const FU_OP_REMW: u8 = 57;
pub const FU_OP_REMUW: u8 = 58;

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
	pub cf: [bool; LEN_CF_T],
	pub predict_address: [bool; VLEN],
}

pub const OFFSET_BP_PREDICT_ADDRESS: usize = 0;
pub const OFFSET_BP_CF: usize = OFFSET_BP_PREDICT_ADDRESS + VLEN;
pub const LEN_BP: usize = OFFSET_BP_CF + LEN_CF_T;

// VLEN + TRANS_ID_BITS + LEN_FU_T + LEN_FU_OP + REG_ADDR_SIZE * 3
// + XLEN + 4 + (XLEN * 2 + 1) + (LEN_CF_T + VLEN) + 1
pub struct ScoreboardEntryT {
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
	pub ex: ExceptionT,
	pub bp: BranchpredictSbeT,
	pub is_compressed: bool,
}

// pub const OFFSET_PC: usize = ID2IS_LEN - 1;
// pub const OFFSET_TRANS_ID: usize = OFFSET_PC - VLEN;
// pub const OFFSET_FU: usize = OFFSET_TRANS_ID + TRANS_ID_BITS;
// pub const OFFSET_OP: usize = OFFSET_FU + LEN_FU_T;
// pub const OFFSET_RS1: usize = OFFSET_OP + LEN_FU_OP;
// pub const OFFSET_RS2: usize = OFFSET_RS1 + REG_ADDR_SIZE;
// pub const OFFSET_RD: usize = OFFSET_RS2 + REG_ADDR_SIZE;
// pub const OFFSET_RESULT: usize = OFFSET_RD + REG_ADDR_SIZE;
// pub const OFFSET_VALID: usize = OFFSET_RESULT + XLEN;
// pub const OFFSET_USE_IMM: usize = OFFSET_VALID + 1;
// pub const OFFSET_USE_ZIMM: usize = OFFSET_USE_IMM + 1;
// pub const OFFSET_USE_PC: usize = OFFSET_USE_ZIMM + 1;
// pub const OFFSET_EX: usize = OFFSET_USE_PC + 1;
// pub const OFFSET_BP: usize = OFFSET_EX + LEN_EXCEPTION;
// pub const OFFSET_IS_COMPRESSED: usize = OFFSET_BP + LEN_BP;

pub const OFFSET_PC: usize = OFFSET_TRANS_ID + TRANS_ID_BITS;
pub const OFFSET_TRANS_ID: usize = OFFSET_FU + LEN_FU_T;
pub const OFFSET_FU: usize = OFFSET_OP + LEN_FU_OP;
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

#[repr(C, packed)]
// 1 + (VLEN + TRANS_ID_BITS + LEN_FU_T + LEN_FU_OP + REG_ADDR_SIZE * 3
// + XLEN + 4 + (XLEN * 2 + 1) + (LEN_CF_T + VLEN) + 1) + 1
pub struct IdPerIssueT {
	pub valid: bool,
	pub sbe: ScoreboardEntryT,
	pub is_ctrl_flow: bool,
}

// 0              1       ...               ...   312   LEN(313)
// [is_ctrl_flow, sbe.is_compressed, ... , sbe.pc, valid]
pub const OFFSET_SCOREBOARD_ENTRY: usize = 1;

#[repr(C, packed)]
// (1 + (VLEN + TRANS_ID_BITS + LEN_FU_T + LEN_FU_OP + REG_ADDR_SIZE * 3
// + XLEN + 4 + (XLEN * 2 + 1) + (LEN_CF_T + VLEN) + 1) + 1) * ISSUE_NUM
pub struct inst_id2is_t {
	issue_inst: [IdPerIssueT; ISSUE_NUM],
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

// this is a const, e.g. 320 array write 313 flush to right, make it be 7
pub const BYTE_ARRAY_OFFSET: usize = ((ID2IS_LEN / 8) as usize + 1) * 8 - ID2IS_LEN;

/*
input:
@value : the value to be written to byte_array
@width : [0...width-1] bit of value is to be written to byte_array
@offset: index of (offset + BYTE_ARRAY_OFFSET)-th bit will be the first bit of byte_array to be written
@byte_array : the array to be written

See lab1/unit-test/write_variable.rs example to use it
*/
pub fn write_variable(
	value: u64,
	width: usize,
	offset: usize, // consider this is 0--0ffset-1 used, offset+1 unused
	byte_array: &mut [u8; (ID2IS_LEN / 8) as usize + 1],
) {
	println!(
		"[RS] Writing {} with {} bits to {} offset",
		value, width, offset
	);
	let target_l = (BYTE_ARRAY_OFFSET + offset) / 8 as usize;
	let target_l_width = 8 - (BYTE_ARRAY_OFFSET + offset) % 8;
	let target_r = (BYTE_ARRAY_OFFSET + offset + width) / 8 as usize;
	let target_r_width = (BYTE_ARRAY_OFFSET + offset + width) % 8;
	let mut mask: u8;
	let mut bit_processed = target_l_width;
	let mut value_byte: u8;
	let flitered_value = value & (make_all_ones(width) as u64);
	//println!(
	//	"debuglog:: at write_variable, flitered_value={0}",
	//	flitered_value
	//);
	if target_l == target_r {
		let rmask = make_all_ones(target_r_width) as u8;
		let lmask = (make_all_ones(target_l_width) << (8 - target_l_width)) as u8;
		value_byte = ((value << (8 - target_l_width)) & make_all_ones(8)) as u8;
	//	println!("debuglig:: target-l-width={0},valuebyte={1}",target_l_width,value);
		write_byte(value_byte, rmask & lmask, &mut byte_array[target_l]);
		return;
	}
	{
		mask = (make_all_ones(target_l_width) << (8 - target_l_width)) as u8;
		value_byte =
			((flitered_value & make_all_ones(target_l_width)) << (8 - target_l_width)) as u8;
	//	println!(
	//		"debuglog:: Processing leftwidth, tar_l_wid={0},tar_l={1},mask={2},valuebyte={3}",
	//		target_l_width, target_l, mask, value_byte
	//	);
		write_byte(value_byte, mask, &mut byte_array[target_l]);
	}
	for i in target_l + 1..target_r {
		mask = make_all_ones(8) as u8;
		value_byte = get_range_bits(flitered_value, bit_processed, bit_processed + 8) as u8;
	//	println!(
	//		"debuglog:: Processing midwidth, i={0},mask={1},valuebyte={2}",
	//		i, mask, value_byte
	//	);
		write_byte(value_byte, mask, &mut byte_array[i]);
		bit_processed += 8;
	}
	if target_l < target_r {
		mask = make_all_ones(target_r_width) as u8;
		value_byte = (get_range_bits(flitered_value, bit_processed, width)
			& make_all_ones(target_r_width)) as u8;
	/*	println!(
			"debuglog:: Processing rightwidth, tar_r_wid={0},tar_r={1},mask={2},valuebyte={3},bitprocessed={4},getrangebit={5}",
			target_r_width,
			target_r,
			mask,
			value_byte,
			bit_processed,
			get_range_bits(flitered_value, bit_processed, width));*/
		write_byte(value_byte, mask, &mut byte_array[target_r]);
	}
}

fn write_byte(value: u8, mask: u8, tar: &mut u8) {
	*tar = (value & mask) | ((*tar) & (!mask));
}

// note : return [left_side, right_side) bits of value, index for bit start from 0
fn get_range_bits(value: u64, left_side: usize, right_side: usize) -> u64 {
	// println!(
	// 	"debuglog:: get_range_bits: value={0},left={1},rig={2}",
	// 	value, left_side, right_side
	// );
	(value >> left_side) & make_all_ones(right_side - left_side)
}

fn make_all_ones(width: usize) -> u64 {
	match width <= 63 {
		true => (1 << width) - 1,
		false => ((1 << 63) - 1) << 1 + 1,
	}
}


// Slice byte_array [left_index,width+left_index), return as a u64 
// An inversion function of write_variable

pub fn read_variable(
	left_index: usize,
	width : usize,
	byte_array: &mut [u8; (ID2IS_LEN / 8) as usize + 1],
) -> u64 {
	if width > 64 {
		println!("[Err]: Width out of range, maximium 64");
	}
	let target_l = (BYTE_ARRAY_OFFSET + left_index) / 8 as usize;
	let target_l_width = 8 - (BYTE_ARRAY_OFFSET + left_index) % 8;
	let target_r = (BYTE_ARRAY_OFFSET + left_index + width) / 8 as usize;
	let target_r_width = (BYTE_ARRAY_OFFSET + left_index + width) % 8;
	let mut mask : u8;
	let mut value_byte : u8;
    //let flitered_value = value & make_all_ones(width);
    let mut value : u64 = 0;
	if target_l == target_r {
		let rmask = make_all_ones(target_r_width) as u8;
		let lmask = (make_all_ones(target_l_width) << (8 - target_l_width)) as u8;
		return read_byte(rmask & lmask, byte_array[target_l]) as u64 >> (8 - target_l_width);
	}

	{
		mask = make_all_ones(target_r_width) as u8;
		value_byte = read_byte(mask,byte_array[target_r]);
		value = value_byte as u64;
		//println!("debuglog:: processing rightwidth, valuebyte={0},value={1},bytearray[]={2},mask={3}",value_byte,value,byte_array[target_r],mask);
	}
	
	let mut i : usize = target_r - 1;
	while i>target_l {
		mask = make_all_ones(8) as u8;
		value_byte = read_byte(mask,byte_array[i]); 
		value = (value << 8) | value_byte as u64;
        //println!("debuglog:: Processing midwidth, i={0},mask={1},valuebyte={2},value={3}",i,mask,value_byte,value);
		i = i - 1;
	}

    if target_l<target_r
	{
		mask = (make_all_ones(target_l_width) << (8 - target_l_width)) as u8;
		value_byte = read_byte(mask,byte_array[target_l]) >> (8 - target_l_width) as u8;
		value = (value << target_l_width) | value_byte as u64;
        //println!("debuglog:: Processing leftwidth, valuebyte={0},value={1}",value_byte,value);
	}
	value
}


fn read_byte(
	mask: u8, 
	tar: u8
) -> u8 {
	tar & mask
}
