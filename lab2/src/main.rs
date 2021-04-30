#![allow(unused)]

extern crate getopts;
extern crate riscv_emu_rust;

extern crate lab1;

use riscv_emu_rust::cpu::Xlen;

use getopts::Options;
use std::env;
use std::fs::File;
use std::io::Read;

use lab1::pkg::*;

fn run_elf(file_path: &str, mem_dump: &str) -> std::io::Result<()> {
	let mut memdump_file = File::open(mem_dump)?;
	let mut memdump_contents = vec![];
	memdump_file.read_to_end(&mut memdump_contents)?;
	/*
	for i in 0 .. memdump_contents.len() {
		println!("memdump[{}]={}",i,memdump_contents[i]);
	} */
	
	let mut elf_file = File::open(file_path)?;
	// let mut elf_file = File::open("../resources/lab1/add.out")?;
	let mut elf_contents = vec![];
	elf_file.read_to_end(&mut elf_contents)?;
	unsafe {
		EMULATOR.setup_program(elf_contents,memdump_contents);
		EMULATOR.update_xlen(Xlen::Bit64);

		EMULATOR.run_program();
	}
	Ok(())
}

fn main() -> std::io::Result<()> {
	let args: Vec<String> = env::args().collect();
	let mut opts = Options::new();
	opts.optflagopt("i", "input", "Set input ELF file", "ELF_PATH");
	opts.optflag("t", "trace", "Enable memory access tracing");
	opts.optflag("h", "help", "Show this help menu");
	opts.optflagopt("m", "memory","Set memory dump file","MEMDUMP_PATH");
	// run_elf(args[1].clone())?;
	let mut mem_dump : String = "".to_string();
	match opts.parse(&args[1..]) {
		Ok(_args) => {
			match _args.opt_str("m") {
				Some(path) => {
					mem_dump = path.to_string();
				}
				_ => {
					println!("{}", opts.usage(&format!("{} [options]", args[0])));
					return Ok(());
				}
			}
			match _args.opt_str("i") {
				Some(path) => {
					if _args.opt_present("t") {
						// @TODO: generate trace
					}
					run_elf(path.as_str(),mem_dump.as_str())?
				}
				_ => {
					println!("{}", opts.usage(&format!("{} [options]", args[0])));
					return Ok(());
				}
			}
		}
		Err(f) => {
			println!("{}", opts.usage(&format!("{} [options]", args[0])));
			return Ok(());
		}
	};
	Ok(())
}
