#![allow(unused)]

extern crate getopts;
extern crate riscv_emu_rust;

pub mod pkg;

use riscv_emu_rust::cpu::Xlen;

use std::env;
use std::fs::File;
use std::io::Read;

use pkg::*;

fn run_elf(file_path: &str) -> std::io::Result<()> {
	let mut elf_file = File::open(file_path)?;
	// let mut elf_file = File::open("../resources/lab1/add.out")?;
	let mut elf_contents = vec![];
	elf_file.read_to_end(&mut elf_contents)?;
	unsafe {
		EMULATOR.setup_program(elf_contents);
		EMULATOR.update_xlen(Xlen::Bit64);

		EMULATOR.run_program();
	}
	Ok(())
}

fn main() -> std::io::Result<()> {
	let args: Vec<String> = env::args().collect();
	// println!("{:?}", &args[1]);
	// run_elf("/opt/orv64-merge/rrv64/tb/test_program/benchmarks/dhrystone.riscv")?;
	println!("running lab1");
	run_elf(&args[1])?;
	// run_elf("/home/cwang/work/riscv-rust/lab1/test64.elf")?;
	// println!("len:{}+({})", ID2IS_LEN, BYTE_ARRAY_OFFSET);
	Ok(())
}
