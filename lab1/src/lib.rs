#![allow(unused)]
pub mod pkg;
pub mod unittest;
pub mod utils;

use fnv::FnvHashMap;
use pkg::*;
use riscv_emu_rust::cpu::*;
use riscv_emu_rust::memory::*;
use riscv_emu_rust::mmu::*;
use riscv_emu_rust::Emulator;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::ptr::NonNull;
use utils::*;

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
	test_int: &mut u64,
) {
	*id2is_valid_o = false;
	for i in 0..(ID2IS_LEN / 8) as usize + 1 {
		id2is_entry_o[i] = 0;
	}

	if is2id_ready_i == false {
		return;
	}

	if let None = EMULATOR.symbol_map {
		EMULATOR.symbol_map = Some(FnvHashMap::default());
		// EMULATOR.format_map = Some(HashMap::default());
		// EMULATOR.fu_map = Some(HashMap::default());
		// EMULATOR.op_map = Some(HashMap::default());

		let mut _format_map: HashMap<String, String> = HashMap::default();
		let mut _fu_map: HashMap<String, u8> = HashMap::default();
		let mut _op_map: HashMap<String, u8> = HashMap::default();
		for i in 0..COSIM_INSTRUCTIONS.len() {
			let mut _instr: &str = COSIM_INSTRUCTIONS[i];
			let mut _format: &str = COSIM_INSTRUCTIONS_FORMAT[i];
			let mut _fu: u8 = COSIM_INSTRUCTIONS_FU_T[i].clone();
			let mut _op: u8 = COSIM_INSTRUCTIONS_FU_OP[i].clone();
			_format_map.insert(String::from(_instr), String::from(_format));
			_fu_map.insert(String::from(_instr), _fu);
			_op_map.insert(String::from(_instr), _op);
		}
		EMULATOR.format_map = Some(_format_map);
		EMULATOR.fu_map = Some(_fu_map);
		EMULATOR.op_map = Some(_op_map);

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
			#[cfg(debug_assertions)]
			{
				println!("[RS] PC {:x}", EMULATOR.cpu.pc);
				println!("[RS] Boot PC {:x}", boot_addr_i);
			}
			EMULATOR.cpu.pc = boot_addr_i;
			EMULATOR.cpu.instruction_buffer = Vec::new();
		}
	}

	match flush_i {
		true => {
			#[cfg(debug_assertions)]
			{
				println!("[RS] PC {:x}", EMULATOR.cpu.pc);
				println!("[RS] Flush PC {:x}", flush_pc_i);
				// if flush_pc_i > 0x7fffffff00 {
				// 	*id2is_valid_o = true;
				// 	for i in 0..(ID2IS_LEN / 8) as usize + 1 {
				// 		id2is_entry_o[i] = 0xff;
				// 	}
				// 	return;
				// }
			}
			EMULATOR.cpu.pc = flush_pc_i.into();
			EMULATOR.cpu.instruction_buffer = Vec::new();
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
			// panic!(
			// 	"Unknown instruction PC:{:x} WORD:{:x}",
			// 	instruction_address, original_word
			// );
			&Instruction {
				mask: 0,
				data: 0,
				name: "NOP",
				cycles: 0,
				operation: |cpu, word, _address| Ok(()),
				disassemble: |cpu, word, _address, evaluate| String::from("NOP"),
			}
		}
	};

	// Set returning fields

	// valid
	write_variable(
		1,
		1,
		OFFSET_SCOREBOARD_ENTRY + LEN_SCOREBOARD_ENTRY,
		id2is_entry_o,
	);

	// pc
	write_variable(
		instruction_address as u64,
		VLEN,
		OFFSET_SCOREBOARD_ENTRY + OFFSET_PC,
		id2is_entry_o,
	);

	// trans_id
	write_variable(
		0 as u64,
		TRANS_ID_BITS,
		OFFSET_SCOREBOARD_ENTRY + OFFSET_TRANS_ID,
		id2is_entry_o,
	);

	// fu
	match EMULATOR
		.fu_map
		.clone()
		.unwrap()
		.get(&String::from(instruction.get_name()))
	{
		Some(f) => write_variable(
			*f as u64,
			LEN_FU,
			OFFSET_SCOREBOARD_ENTRY + OFFSET_FU,
			id2is_entry_o,
		),
		None => write_variable(
			FU_NONE as u64,
			LEN_FU,
			OFFSET_SCOREBOARD_ENTRY + OFFSET_FU,
			id2is_entry_o,
		),
	};

	// op
	match EMULATOR
		.op_map
		.clone()
		.unwrap()
		.get(&String::from(instruction.get_name()))
	{
		Some(f) => write_variable(
			*f as u64,
			LEN_OP,
			OFFSET_SCOREBOARD_ENTRY + OFFSET_OP,
			id2is_entry_o,
		),
		None => write_variable(
			OP_ADD as u64,
			LEN_OP,
			OFFSET_SCOREBOARD_ENTRY + OFFSET_OP,
			id2is_entry_o,
		),
	};

	// registers
	// rs1
	// rs2
	// rd
	// imm

	match EMULATOR
		.format_map
		.clone()
		.unwrap()
		.get(&String::from(instruction.get_name()))
	{
		Some(f) => match f.as_str() {
			"B" => {
				#[cfg(debug_assertions)]
				println!("[RS] Get B format");
				let _b_format = parse_format_b(word);

				write_variable(
					_b_format.rs1 as u64,
					REG_ADDR_SIZE,
					OFFSET_SCOREBOARD_ENTRY + OFFSET_RS1,
					id2is_entry_o,
				);

				write_variable(
					_b_format.rs2 as u64,
					REG_ADDR_SIZE,
					OFFSET_SCOREBOARD_ENTRY + OFFSET_RS2,
					id2is_entry_o,
				);

				write_variable(
					_b_format.imm as u64,
					XLEN,
					OFFSET_SCOREBOARD_ENTRY + OFFSET_RESULT,
					id2is_entry_o,
				);
				*test_int = _b_format.imm as u64;
			}
			"I" => {
				#[cfg(debug_assertions)]
				println!("[RS] Get I format");
				match instruction.get_name() {
					"CSRRC" | "CSRRCI" | "CSRRS" | "CSRRW" | "CSRRWI" | "CSRRSI" => {
						#[cfg(debug_assertions)]
						println!("[RS] Get CSR format");
						let _csr_format = parse_format_csr(word);

						write_variable(
							_csr_format.csr as u64,
							REG_ADDR_SIZE,
							OFFSET_SCOREBOARD_ENTRY + OFFSET_RS1,
							id2is_entry_o,
						);
						write_variable(
							(_csr_format.rs & 0x1f) as u64,
							REG_ADDR_SIZE,
							OFFSET_SCOREBOARD_ENTRY + OFFSET_RS2,
							id2is_entry_o,
						);
						write_variable(
							_csr_format.rd as u64,
							REG_ADDR_SIZE,
							OFFSET_SCOREBOARD_ENTRY + OFFSET_RD,
							id2is_entry_o,
						);
						match instruction.get_name() {
							"CSRRCI" | "CSRRWI" | "CSRRSI" => {
								//use_zimm
								write_variable(
									1 as u64,
									1,
									OFFSET_SCOREBOARD_ENTRY + OFFSET_USE_ZIMM,
									id2is_entry_o,
								);
							}
							_ => {}
						}
					}
					_ => {
						#[cfg(debug_assertions)]
						println!("[RS] Get I format");
						let _i_format = parse_format_i(word);

						write_variable(
							_i_format.rs1 as u64,
							REG_ADDR_SIZE,
							OFFSET_SCOREBOARD_ENTRY + OFFSET_RS1,
							id2is_entry_o,
						);

						write_variable(
							_i_format.rd as u64,
							REG_ADDR_SIZE,
							OFFSET_SCOREBOARD_ENTRY + OFFSET_RD,
							id2is_entry_o,
						);

						write_variable(
							_i_format.imm as u64,
							XLEN,
							OFFSET_SCOREBOARD_ENTRY + OFFSET_RESULT,
							id2is_entry_o,
						);
						*test_int = _i_format.imm as u64;

						//use_imm
						write_variable(
							1 as u64,
							1,
							OFFSET_SCOREBOARD_ENTRY + OFFSET_USE_IMM,
							id2is_entry_o,
						);
					}
				}
			}
			"J" => {
				#[cfg(debug_assertions)]
				println!("[RS] Get J format");
				let _j_format = parse_format_j(word);

				write_variable(
					_j_format.rd as u64,
					REG_ADDR_SIZE,
					OFFSET_SCOREBOARD_ENTRY + OFFSET_RD,
					id2is_entry_o,
				);

				write_variable(
					_j_format.imm as u64,
					XLEN,
					OFFSET_SCOREBOARD_ENTRY + OFFSET_RESULT,
					id2is_entry_o,
				);
				*test_int = _j_format.imm as u64;

				//use_imm
				write_variable(
					1 as u64,
					1,
					OFFSET_SCOREBOARD_ENTRY + OFFSET_USE_IMM,
					id2is_entry_o,
				);
			}
			"R" => {
				#[cfg(debug_assertions)]
				println!("[RS] Get R format");
				let _r_format = parse_format_r(word);

				write_variable(
					_r_format.rs1 as u64,
					REG_ADDR_SIZE,
					OFFSET_SCOREBOARD_ENTRY + OFFSET_RS1,
					id2is_entry_o,
				);

				write_variable(
					_r_format.rs2 as u64,
					REG_ADDR_SIZE,
					OFFSET_SCOREBOARD_ENTRY + OFFSET_RS2,
					id2is_entry_o,
				);

				write_variable(
					_r_format.rd as u64,
					REG_ADDR_SIZE,
					OFFSET_SCOREBOARD_ENTRY + OFFSET_RD,
					id2is_entry_o,
				);
			}
			"S" => {
				#[cfg(debug_assertions)]
				println!("[RS] Get S format");
				let _s_format = parse_format_s(word);

				write_variable(
					_s_format.rs1 as u64,
					REG_ADDR_SIZE,
					OFFSET_SCOREBOARD_ENTRY + OFFSET_RS1,
					id2is_entry_o,
				);

				write_variable(
					_s_format.rs2 as u64,
					REG_ADDR_SIZE,
					OFFSET_SCOREBOARD_ENTRY + OFFSET_RS2,
					id2is_entry_o,
				);

				write_variable(
					_s_format.imm as u64,
					XLEN,
					OFFSET_SCOREBOARD_ENTRY + OFFSET_RESULT,
					id2is_entry_o,
				);
				*test_int = _s_format.imm as u64;

				//use_imm
				write_variable(
					1 as u64,
					1,
					OFFSET_SCOREBOARD_ENTRY + OFFSET_USE_IMM,
					id2is_entry_o,
				);
			}
			"U" => {
				#[cfg(debug_assertions)]
				println!("[RS] Get U format");
				let _u_format = parse_format_u(word);

				write_variable(
					_u_format.rd as u64,
					REG_ADDR_SIZE,
					OFFSET_SCOREBOARD_ENTRY + OFFSET_RD,
					id2is_entry_o,
				);

				write_variable(
					_u_format.imm as u64,
					XLEN,
					OFFSET_SCOREBOARD_ENTRY + OFFSET_RESULT,
					id2is_entry_o,
				);
				*test_int = _u_format.imm as u64;

				//use_imm
				write_variable(
					1 as u64,
					1,
					OFFSET_SCOREBOARD_ENTRY + OFFSET_USE_IMM,
					id2is_entry_o,
				);
			}
			_ => {
				#[cfg(debug_assertions)]
				println!("[RS] Get undefined {} format", f);
			}
		},
		None => {}
	}

	// Set tag fields
	// valid
	write_variable(
		1 as u64,
		1,
		OFFSET_SCOREBOARD_ENTRY + OFFSET_VALID,
		id2is_entry_o,
	);
	match instruction.get_name() {
		"JAL" => write_variable(
			0 as u64,
			1,
			OFFSET_SCOREBOARD_ENTRY + OFFSET_VALID,
			id2is_entry_o,
		),
		_ => {}
	}

	// use_pc
	match instruction.get_name() {
		"AUIPC" => {
			write_variable(
				1 as u64,
				1,
				OFFSET_SCOREBOARD_ENTRY + OFFSET_USE_PC,
				id2is_entry_o,
			);
		}
		_ => {}
	}

	// is_compressed
	write_variable(
		0 as u64,
		1,
		OFFSET_SCOREBOARD_ENTRY + OFFSET_IS_COMPRESSED,
		id2is_entry_o,
	);

	// is_ctrl_flow
	match instruction.get_name() {
		"BEQ" | "BGE" | "BGEU" | "BLT" | "BLTU" | "BNE" | "JAL" | "JALR" => {
			write_variable(1 as u64, 1, 0, id2is_entry_o);
		}
		_ => {}
	}

	// id2is_valid_o
	*id2is_valid_o = true;
	// *test_int = 0xaaaa_ffff_ffff_ffff;
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
pub unsafe extern "C" fn dpi_issue_execute_writeback(
	_clk_i: bool,
	rst_ni: bool,
	flush_i: bool,
	id2is_valid_i: bool,
	id2is_entry_i: &[u8; (ID2IS_LEN / 8) as usize + 1],

	ex2io_load_o: &mut bool,
	ex2io_store_o: &mut bool,
	ex2io_data_o: &mut [u8; (XLEN / 8) as usize + 1],
	ex2io_addr_o: &mut [u8; (PLEN / 8) as usize + 1],
) {
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

	match id2is_valid_i {
		true => {
			let instruction_address = 0x80000000 as u64;

			// Fetch and decode
			// Fetching
			EMULATOR.cpu.pc = instruction_address;
			let original_word = match EMULATOR.cpu.fetch() {
				Ok(word) => word,
				Err(e) => panic!("Failed to fetch original_word"),
			};

			// Parsing cache line
			// use input.is_compressed
			let word = match (original_word & 0x3) == 0x3 {
				true => {
					// EMULATOR.cpu.pc = EMULATOR.cpu.pc.wrapping_add(4); // 32-bit length non-compressed instruction
					original_word
				}
				false => {
					// EMULATOR.cpu.pc = EMULATOR.cpu.pc.wrapping_add(2); // 16-bit length compressed instruction
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

			// let cycles = instruction.get_cycles();

			match instruction.get_name() {
				"LB" | "LBU" | "LD" | "LH" | "LHU" | "LUI" | "LW" | "LWU" => {
					*ex2io_load_o = true;
					*ex2io_store_o = false;
					for i in 0..ex2io_addr_o.len() {
						ex2io_addr_o[i] = 0 << i * 8;
					}
					for i in 0..ex2io_data_o.len() {
						ex2io_data_o[i] = 0 << i * 8;
					}
					match instruction.get_name() {
						"LB" => {}
						"LBU" => {}
						"LD" => {}
						"LH" => {}
						"LHU" => {}
						"LUI" => {}
						"LW" => {}
						"LWU" => {}
						_ => {}
					}
				}
				"SB" | "SD" | "SH" => {
					*ex2io_load_o = false;
					*ex2io_store_o = true;
					for i in 0..ex2io_data_o.len() {
						ex2io_data_o[i] = 0 << i * 8;
					}
					for i in 0..ex2io_addr_o.len() {
						ex2io_addr_o[i] = 0 << i * 8;
					}
					match instruction.get_name() {
						"SB" => {}
						"SD" => {}
						"SH" => {}
						_ => {}
					}
				}
				_ => {}
			}

			match (instruction.operation)(EMULATOR.get_mut_cpu(), word, instruction_address) {
				Ok(()) => {}
				Err(e) => EMULATOR
					.get_mut_cpu()
					.handle_exception(e, instruction_address),
			}
		}
		false => {
			*ex2io_load_o = false;
			*ex2io_store_o = false;
			for i in 0..ex2io_data_o.len() {
				ex2io_data_o[i] = 0 << i * 8;
			}
			for i in 0..ex2io_addr_o.len() {
				ex2io_addr_o[i] = 0 << i * 8;
			}
		}
	}
}

/// Dpi MA(load/store) interface.
///
/// @TODO: Determine impl
#[no_mangle]
pub unsafe extern "C" fn dpi_load_store(
	_clk_i: bool,
	rst_ni: bool,
	flush_i: bool,
	id2is_valid_i: bool,
	id2is_entry_i: &[u8; (ID2IS_LEN / 8) as usize + 1],

	ex2io_load_i: bool,
	ex2io_store_i: bool,
	ex2io_data_i: &[u8; (XLEN / 8) as usize + 1],
	ex2io_addr_i: &[u8; (XLEN / 8) as usize + 1],
	io2wb_load_valid_o: &mut bool,
	io2wb_load_data_o: &mut bool,
) {
	// let instr: &Instruction = match EMULATOR.get_cpu().decode_raw(data: u32) {
	// 	Ok(i) => i,
	// 	_ => panic!("decode failed"),
	// };
}
