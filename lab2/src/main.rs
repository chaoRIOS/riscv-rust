#![allow(unused)]

extern crate getopts;
extern crate riscv_emu_rust;

extern crate lab1;

use riscv_emu_rust::{
	cpu::Xlen,
	dram::{setup_pipe, terminate_pipe},
};

use getopts::Options;
use std::env;
use std::fs::File;
use std::io::Read;

use lab1::pkg::*;

fn run_elf(
	input_path: &str,
	trace_path: &str,
	trace_memory_access: bool,
	mem_dump: &str,
) -> std::io::Result<()> {
	let mut memdump_file = File::open(mem_dump)?;
	let mut memdump_contents = vec![];
	memdump_file.read_to_end(&mut memdump_contents)?;
	let mut elf_file = File::open(input_path)?;
	let mut elf_contents = vec![];
	elf_file.read_to_end(&mut elf_contents)?;
	setup_pipe(
		"/home/cwang/work/riscv-rust/lab2/rqst_to_memory",
		"/home/cwang/work/riscv-rust/lab2/resp_to_cpu ",
	);
	unsafe {
		EMULATOR.setup_program(elf_contents, memdump_contents);
		EMULATOR.update_xlen(Xlen::Bit64);
		EMULATOR.run_program(trace_memory_access, trace_path);
	}
	terminate_pipe();
	Ok(())
}

fn main() -> std::io::Result<()> {
	let args: Vec<String> = env::args().collect();
	let mut opts = Options::new();
	opts.optflagopt("i", "input", "Set input ELF file", "ELF_PATH");
	opts.optflagopt("t", "trace", "Enable memory access tracing", "TRACE_PATH");
	opts.optflag("h", "help", "Show this help menu");
	opts.optflagopt("m", "memory", "Set memory dump file", "MEMDUMP_PATH");
	// run_elf(args[1].clone())?;
	let mut mem_dump: String = "".to_string();
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
				Some(input_path) => {
					match _args.opt_str("t") {
						// @TODO: generate trace
						Some(trace_path) => {
							println!("{}", trace_path);
							let mut file = File::create(&trace_path).unwrap();
							run_elf(
								input_path.as_str(),
								trace_path.as_str(),
								true,
								mem_dump.as_str(),
							)?
						}
						_ => run_elf(input_path.as_str(), "", false, mem_dump.as_str())?,
					}
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
