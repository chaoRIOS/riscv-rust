#![allow(unused)]
pub mod pkg;
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
	flush_pc_i: u64,
	_flush_bp_i: bool,
	boot_addr_i: u64,
	is2id_ready_i: bool,
	id2is_valid_o: &mut bool,
	id2is_entry_o: &mut [u8; (ID2IS_LEN / 8) as usize + 1],
) {
	*id2is_valid_o = false;
	for i in 0..(ID2IS_LEN / 8) as usize + 1 {
		id2is_entry_o[i] = 0xfa;
	}

	if let None = EMULATOR.symbol_map {
		EMULATOR.symbol_map = Some(FnvHashMap::default());
		let mut elf_file =
			match File::open("/opt/orv64-merge/rrv64/tb/test_program/benchmarks/dhrystone.riscv") {
				Ok(f) => f,
				Err(_) => panic!("Failed to load ELF"),
			};
		let mut elf_contents = vec![];
		match elf_file.read_to_end(&mut elf_contents) {
			Ok(_) => {}
			Err(_) => panic!("Failed to read ELF"),
		};
		EMULATOR.setup_program(elf_contents);
	}

	// Check input signals
	match rst_ni {
		true => {}
		false => {
			// EMULATOR.cpu.pc = boot_addr_i;
			// EMULATOR.cpu.instruction_buffer = Vec::new();
		}
	}

	match flush_i {
		true => {
			// EMULATOR.cpu.pc = flush_pc_i.into();
			// EMULATOR.cpu.instruction_buffer = Vec::new();
		}
		false => {}
	}

	// Fetch and decode
	// Fetching
	let original_word = match EMULATOR.cpu.fetch() {
		Ok(word) => word,
		Err(e) => panic!("Failed to fetch original_word"),
	};
	let instruction_address = EMULATOR.cpu.pc;

	// Parsing cache line
	let word = match (original_word & 0x3) == 0x3 {
		true => {
			EMULATOR.cpu.pc = EMULATOR.cpu.pc.wrapping_add(4); // 32-bit length non-compressed instruction
			original_word
		}
		false => {
			EMULATOR.cpu.pc = EMULATOR.cpu.pc.wrapping_add(2); // 16-bit length compressed instruction
			EMULATOR.cpu.uncompress(original_word & 0xffff)
		}
	};

	// Decoding
	let instruction = match EMULATOR.cpu.decode(word) {
		Ok(inst) => inst,
		Err(()) => {
			panic!(
				"Unknown instruction PC:{:x} WORD:{:x}",
				instruction_address, original_word
			);
		}
	};

	// Forming return value
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
pub extern "C" fn dpi_execute(data: u32) {
	// let instr: &Instruction = match EMULATOR.get_cpu().decode_raw(data: u32) {
	// 	Ok(i) => i,
	// 	_ => panic!("decode failed"),
	// };
}

/// Dpi MA(load/store) interface.
///
/// @TODO: Determine impl
#[no_mangle]
pub extern "C" fn dpi_load_store(data: u64, addr: u64) {
	// let instr: &Instruction = match EMULATOR.get_cpu().decode_raw(data: u32) {
	// 	Ok(i) => i,
	// 	_ => panic!("decode failed"),
	// };
}
