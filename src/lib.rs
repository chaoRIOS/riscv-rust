// @TODO: temporal
const TEST_MEMORY_CAPACITY: u64 = 1024 * 512;
const PROGRAM_MEMORY_CAPACITY: u64 = 1024 * 1024 * 128; // big enough to run Linux and xv6

extern crate fnv;
use self::fnv::FnvHashMap;
use std::collections::HashMap;
use std::str;

pub mod cpu;
pub mod elf_analyzer;
pub mod memory;
pub mod mmu;

use cpu::{Cpu, Xlen};
use elf_analyzer::ElfAnalyzer;

/// RISC-V emulator. It emulates RISC-V CPU and peripheral devices.
///
/// Sample code to run the emulator.
/// ```ignore
/// // Creates an emulator with arbitary terminal
/// let mut emulator = Emulator::new(Box::new(DefaultTerminal::new()));
/// // Set up program content binary
/// emulator.setup_program(program_content);
/// // Set up Filesystem content binary
/// emulator.setup_filesystem(fs_content);
/// // Go!
/// emulator.run();
/// ```
pub struct Emulator {
	pub cpu: Cpu,

	/// Stores mapping from symbol to virtual address
	pub symbol_map: Option<FnvHashMap<String, u64>>,
	pub format_map: Option<HashMap<String, String>>,
	pub fu_map: Option<HashMap<String, u8>>,
	pub op_map: Option<HashMap<String, u8>>,

	/// [`riscv-tests`](https://github.com/riscv/riscv-tests) program specific
	/// properties. Whether the program set by `setup_program()` is
	/// [`riscv-tests`](https://github.com/riscv/riscv-tests) program.
	pub is_test: bool,

	/// [`riscv-tests`](https://github.com/riscv/riscv-tests) specific properties.
	/// The address where data will be sent to terminal
	pub tohost_addr: u64,
}

impl Emulator {
	/// Creates a new `Emulator`. [`Terminal`](terminal/trait.Terminal.html)
	/// is internally used for transferring input/output data to/from `Emulator`.
	pub fn new() -> Self {
		Emulator {
			cpu: Cpu::new(),

			symbol_map: Some(FnvHashMap::default()),
			format_map: Some(HashMap::default()),
			fu_map: Some(HashMap::default()),
			op_map: Some(HashMap::default()),

			// These can be updated in setup_program()
			is_test: false,
			tohost_addr: 0, // assuming tohost_addr is non-zero if exists
		}
	}

	/// Runs program set by `setup_program()`. Calls `run_test()` if the program
	/// is [`riscv-tests`](https://github.com/riscv/riscv-tests).
	/// Otherwise calls `run_program()`.
	pub fn run(&mut self) {
		match self.is_test {
			true => self.run_test(),
			false => self.run_program(),
		};
	}

	/// Runs program set by `setup_program()`. The emulator won't stop forever.
	/// * Added our print function.
	pub fn run_program(&mut self) {
		// let fromhost_addr = 0x80001040;
		let fromhost_addr = 0x80001ea0;
		loop {
			// let disas = self.cpu.disassemble_next_instruction();
			// self.put_bytes_to_terminal(disas.as_bytes());
			// self.put_bytes_to_terminal(&[10]); // new line
			let tohost_data_addr = self.cpu.get_mut_mmu().load_word_raw(self.tohost_addr);
			if tohost_data_addr != 0 {
				match tohost_data_addr {
					0..=0x80000000 => {
						println!("{:x}", tohost_data_addr);
						break;
					}
					_ => {
						// println!("tohost_data_addr:{:X}", tohost_data_addr);
						// let disas = self.cpu.disassemble_next_instruction();
						// self.put_bytes_to_terminal(disas.as_bytes());
						// self.put_bytes_to_terminal(&[10]); // new line
						if self
							.cpu
							.get_mut_mmu()
							.load_word_raw((24 + tohost_data_addr).into())
							!= 0 || self
							.cpu
							.get_mut_mmu()
							.load_word_raw((28 + tohost_data_addr).into())
							!= 0
						{
							let base = self
								.cpu
								.get_mut_mmu()
								.load_word_raw((4 * 4 + tohost_data_addr).into());
							let length = self
								.cpu
								.get_mut_mmu()
								.load_word_raw((6 * 4 + tohost_data_addr).into());
							for i in 0..length {
								let data = self.cpu.get_mut_mmu().load_raw((i + base).into());
								// self.put_chars_to_terminal(format!("{:X} ", data).as_bytes());
								print!("{}", data as char);
							}
						}
						self.cpu.get_mut_mmu().store_word_raw(fromhost_addr, 1);
						self.cpu.get_mut_mmu().store_word_raw(self.tohost_addr, 0);
					}
				};
			}
			self.tick();
		}
	}

	/// Method for running [`riscv-tests`](https://github.com/riscv/riscv-tests) program.
	/// The differences from `run_program()` are
	/// * Disassembles every instruction and dumps to terminal
	/// * The emulator stops when the test finishes
	/// * Displays the result message (pass/fail) to terminal
	pub fn run_test(&mut self) {
		// @TODO: Send this message to terminal?
		println!("This elf file seems riscv-tests elf file. Running in test mode.");
		loop {
			let disas = self.cpu.disassemble_next_instruction();
			self.put_bytes_to_terminal(disas.as_bytes());
			self.put_bytes_to_terminal(&[10]); // new line

			self.tick();

			// It seems in riscv-tests ends with end code
			// written to a certain physical memory address
			// (0x80001000 in mose test cases) so checking
			// the data in the address and terminating the test
			// if non-zero data is written.
			// End code 1 seems to mean pass.
			let endcode = self.cpu.get_mut_mmu().load_word_raw(self.tohost_addr);
			if endcode != 0 {
				match endcode {
					1 => self.put_bytes_to_terminal(
						format!("Test Passed with {:X}\n", endcode).as_bytes(),
					),
					_ => self.put_bytes_to_terminal(
						format!("Test Failed with {:X}\n", endcode).as_bytes(),
					),
				};
				break;
			}
		}
	}

	/// Helper method. Sends ascii code bytes to terminal.
	///
	/// # Arguments
	/// * `bytes`
	fn put_bytes_to_terminal(&mut self, bytes: &[u8]) {
		for i in 0..bytes.len() {
			let str = vec![bytes[i]];
			match str::from_utf8(&str) {
				Ok(s) => {
					print!("{}", s);
				}
				Err(_e) => {}
			};
		}
	}

	/// Runs CPU one cycle
	pub fn tick(&mut self) {
		self.cpu.tick();
	}

	/// Sets up program run by the program. This method analyzes the passed content
	/// and configure CPU properly. If the passed contend doesn't seem ELF file,
	/// it panics. This method is expected to be called only once.
	///
	/// # Arguments
	/// * `data` Program binary
	// @TODO: Make ElfAnalyzer and move the core logic there.
	// @TODO: Returns `Err` if the passed contend doesn't seem ELF file
	pub fn setup_program(&mut self, data: Vec<u8>) {
		let analyzer = ElfAnalyzer::new(data);

		if !analyzer.validate() {
			panic!("This file does not seem ELF file");
		}

		let header = analyzer.read_header();
		//let program_headers = analyzer._read_program_headers(&header);
		let section_headers = analyzer.read_section_headers(&header);

		let mut program_data_section_headers = vec![];
		let mut symbol_table_section_headers = vec![];
		let mut string_table_section_headers = vec![];

		for i in 0..section_headers.len() {
			match section_headers[i].sh_type {
				1 => program_data_section_headers.push(&section_headers[i]),
				2 => symbol_table_section_headers.push(&section_headers[i]),
				3 => string_table_section_headers.push(&section_headers[i]),
				_ => {}
			};
		}

		// Find program data section named .tohost to detect if the elf file is riscv-tests
		self.tohost_addr = match analyzer
			.find_tohost_addr(&program_data_section_headers, &string_table_section_headers)
		{
			Some(address) => address,
			None => 0x80001ea8,
		};

		// Creates symbol - virtual address mapping
		if string_table_section_headers.len() > 0 {
			let entries = analyzer.read_symbol_entries(&header, &symbol_table_section_headers);
			// Assuming symbols are in the first string table section.
			// @TODO: What if symbol can be in the second or later string table sections?
			let map = analyzer.create_symbol_map(&entries, &string_table_section_headers[0]);
			let mut _symbol_map = FnvHashMap::default();
			for key in map.keys() {
				_symbol_map.insert(key.to_string(), *map.get(key).unwrap());
			}
			self.symbol_map = Some(_symbol_map);
		}

		// Detected whether the elf file is riscv-tests.
		// Setting up CPU and Memory depending on it.

		self.cpu.update_xlen(match header.e_width {
			32 => Xlen::Bit32,
			64 => Xlen::Bit64,
			_ => panic!("No happen"),
		});

		if self.tohost_addr != 0 {
			self.is_test = true;
			self.cpu.get_mut_mmu().init_memory(TEST_MEMORY_CAPACITY);
		} else {
			self.is_test = false;
			self.cpu.get_mut_mmu().init_memory(PROGRAM_MEMORY_CAPACITY);
		}

		for i in 0..program_data_section_headers.len() {
			let sh_addr = program_data_section_headers[i].sh_addr;
			let sh_offset = program_data_section_headers[i].sh_offset as usize;
			let sh_size = program_data_section_headers[i].sh_size as usize;
			if sh_addr >= 0x80000000 && sh_offset > 0 && sh_size > 0 {
				for j in 0..sh_size {
					self.cpu
						.get_mut_mmu()
						.store_raw(sh_addr + j as u64, analyzer.read_byte(sh_offset + j));
				}
			}
		}

		self.cpu.update_pc(header.e_entry);
	}

	/// Loads symbols of program and adds them to `symbol_map`.
	///
	/// # Arguments
	/// * `content` Program binary
	// pub fn load_program_for_symbols(&mut self, content: Vec<u8>) {
	// 	let analyzer = ElfAnalyzer::new(content);

	// 	if !analyzer.validate() {
	// 		panic!("This file does not seem ELF file");
	// 	}

	// 	let header = analyzer.read_header();
	// 	let section_headers = analyzer.read_section_headers(&header);

	// 	let mut program_data_section_headers = vec![];
	// 	let mut symbol_table_section_headers = vec![];
	// 	let mut string_table_section_headers = vec![];

	// 	for i in 0..section_headers.len() {
	// 		match section_headers[i].sh_type {
	// 			1 => program_data_section_headers.push(&section_headers[i]),
	// 			2 => symbol_table_section_headers.push(&section_headers[i]),
	// 			3 => string_table_section_headers.push(&section_headers[i]),
	// 			_ => {}
	// 		};
	// 	}

	// 	// Creates symbol - virtual address mapping
	// 	if string_table_section_headers.len() > 0 {
	// 		let entries = analyzer.read_symbol_entries(&header, &symbol_table_section_headers);
	// 		// Assuming symbols are in the first string table section.
	// 		// @TODO: What if symbol can be in the second or later string table sections?
	// 		let map = analyzer.create_symbol_map(&entries, &string_table_section_headers[0]);
	// 		for key in map.keys() {
	// 			self.symbol_map
	// 				.unwrap()
	// 				.insert(key.to_string(), *map.get(key).unwrap());
	// 		}
	// 	}
	// }

	/// Updates XLEN (the width of an integer register in bits) in CPU.
	///
	/// # Arguments
	/// * `xlen`
	pub fn update_xlen(&mut self, xlen: Xlen) {
		self.cpu.update_xlen(xlen);
	}

	/// Returns immutable reference to `Cpu`.
	pub fn get_cpu(&self) -> &Cpu {
		&self.cpu
	}

	/// Returns mutable reference to `Cpu`.
	pub fn get_mut_cpu(&mut self) -> &mut Cpu {
		&mut self.cpu
	}

	// /// Returns a virtual address corresponding to symbol strings
	// ///
	// /// # Arguments
	// /// * `s` Symbol strings
	// pub fn get_addredd_of_symbol(&self, s: &String) -> Option<u64> {
	// 	match self.symbol_map.unwrap().get(s) {
	// 		Some(address) => Some(*address),
	// 		None => None,
	// 	}
	// }
}
