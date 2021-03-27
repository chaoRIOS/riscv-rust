extern crate getopts;
extern crate riscv_emu_rust;

pub mod pkg;

use riscv_emu_rust::cpu::Xlen;
use riscv_emu_rust::Emulator;

use std::fs::File;
use std::io::Read;

use pkg::*;

fn main() -> std::io::Result<()> {
	let mut elf_file =
		File::open("/opt/orv64-merge/rrv64/tb/test_program/benchmarks/dhrystone.riscv")?;
	// let mut elf_file = File::open("../resources/lab1/add.out")?;
	let mut elf_contents = vec![];
	elf_file.read_to_end(&mut elf_contents)?;

	unsafe {
		// let mut emulator = Emulator::new();
		EMULATOR.setup_program(elf_contents);
		EMULATOR.update_xlen(Xlen::Bit64);

		EMULATOR.run_program();
	}
	// emulator.run_program();
	Ok(())
}
