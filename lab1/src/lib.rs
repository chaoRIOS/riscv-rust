pub mod pkg;
use bitvec::prelude::*;
use fnv::FnvHashMap;
use pkg::*;
use riscv_emu_rust::cpu::*;
use riscv_emu_rust::memory::*;
use riscv_emu_rust::mmu::*;
use riscv_emu_rust::Emulator;
use std::fs::File;
use std::io::Read;
use std::ptr::NonNull;

/// Dpi IF+ID(frontend) interface. Embedded an ELF parser and maintained
/// readonly instruction memory space.
#[no_mangle]
pub unsafe extern "C" fn dpi_fetch_decode(
	_clk_i: bool,
	rst_ni: bool,
	flush_i: bool,
	flush_pc_i: u32,
	_flush_bp_i: bool,
	boot_addr_i: u32,
	instruction_o: &mut [u8; 1],
	is_control_flow_instr_o: &mut u8,
) {
	instruction_o[0] = 0b00011010;
	*is_control_flow_instr_o = 0;

	// if let None = EMULATOR.symbol_map {
	// 	EMULATOR.symbol_map = Some(FnvHashMap::default());
	// 	let mut elf_file =
	// 		match File::open("/opt/orv64-merge/rrv64/tb/test_program/benchmarks/dhrystone.riscv") {
	// 			Ok(f) => f,
	// 			Err(_) => panic!("Failed to load ELF"),
	// 		};
	// 	let mut elf_contents = vec![];
	// 	match elf_file.read_to_end(&mut elf_contents) {
	// 		Ok(_) => {}
	// 		Err(_) => panic!("Failed to read ELF"),
	// 	};
	// 	EMULATOR.setup_program(elf_contents);
	// }

	// Check input signals
	// match rst_ni {
	// 	true => {}
	// 	false => {
	// 		EMULATOR.cpu.pc = boot_addr_i.into();
	// 		EMULATOR.cpu.instruction_buffer = [0u32; 64];
	// 	}
	// }

	// match flush_i {
	// 	true => {
	// 		EMULATOR.cpu.pc = flush_pc_i.into();
	// 		EMULATOR.cpu.instruction_buffer = [0u32; 64];
	// 	}
	// 	false => {}
	// }

	// Fetch and decode
	// Fetching
	// let original_bytes = match EMULATOR.cpu.get_mut_mmu().fetch_bytes(EMULATOR.cpu.pc, 8) {
	// 	Ok(bytes) => bytes,
	// 	Err(e) => panic!("Failed to fetch instructions"),
	// };

	// Parsing cache line

	// Decoding

	// println!("[RS]OK");
	// Returing
	// let bits = BitSlice::<Lsb0, _>::from_element(&EMULATOR.cpu.pc);
	// for i in 0..64 {
	// 	instruction_o.pc[i] = bits[i];
	// }
	// instruction_o.pc = [true; 64];
	// instruction_o.trans_id = [true; 3];
	// instruction_o.fu = fu_t::NONE;
	// instruction_o.op = fu_op::ADD;
	// instruction_o.rs1 = 0xabcd;
	// instruction_o.rs2 = [true; 6];
	// instruction_o.rd = [true; 6];
	// instruction_o.result = 123u64;
	// (*instruction_o).valid = true;
	// (*instruction_o).use_imm = false;
	// (*instruction_o).use_zimm = true;
	// (*instruction_o).use_pc = false;
	// instruction_o.ex = exception_t {
	// 	cause: [true; 64],
	// 	tval: [true; 64],
	// 	valid: true,
	// };
	// instruction_o.bp = branchpredict_sbe_t {
	// 	cf: cf_t::NoCF,
	// 	predict_address: [true; 39],
	// };

	// (*ptr).is_compressed = true;
	// (*ptr).valid = true;
	// (*ptr).use_imm = false;
	// (*ptr).use_zimm = true;
	// (*ptr).use_pc = false;
	// pub extern "C" fn destroy(ptr: NonNull<Object>) {
	//     let obj = unsafe { Box::<Object>::from_raw(ptr.as_ptr()) };
	//     println!("{:?} is destroied", obj);
	// }

	// println!("[RS]instruction_o OK");

	// println!("[RS]is_control_flow_instr_o OK");
}

/// Dpi EX(backend) interface.
///
/// For those load/store instrucions, only
/// calculate corresponding data and addresses here.
///
/// May use
/// 1) return values
/// 2) &mut input parameters
///
/// to carry calculating results
#[no_mangle]
pub extern "C" fn dpi_execute(data: u32) -> u32 {
	// let instr: &Instruction = match EMULATOR.get_cpu().decode_raw(data: u32) {
	// 	Ok(i) => i,
	// 	_ => panic!("decode failed"),
	// };

	10
}

/// Dpi MA(load/store) interface.
///
/// @TODO: Determine impl
#[no_mangle]
pub extern "C" fn dpi_load_store(data: u32) -> u32 {
	// let instr: &Instruction = match EMULATOR.get_cpu().decode_raw(data: u32) {
	// 	Ok(i) => i,
	// 	_ => panic!("decode failed"),
	// };

	10
}
