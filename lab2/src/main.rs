#![allow(unused)]

extern crate getopts;
extern crate riscv_emu_rust;

extern crate lab1;

use riscv_emu_rust::cpu::Xlen;
#[cfg(feature = "dramsim")]
use riscv_emu_rust::dram::{setup_pipe, terminate_pipe};

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
	let mut memdump_contents = vec![];
	#[cfg(feature = "memdump")]
	{
		let mut memdump_file = File::open(mem_dump)?;
		memdump_file.read_to_end(&mut memdump_contents)?;
	}
	let mut elf_file = File::open(input_path)?;
	let mut elf_contents = vec![];
	elf_file.read_to_end(&mut elf_contents)?;
	unsafe {
		EMULATOR.setup_program(elf_contents, memdump_contents);
		EMULATOR.update_xlen(Xlen::Bit64);
		EMULATOR.run_program(trace_memory_access, trace_path);
	}
	#[cfg(feature = "dramsim")]
	{
		terminate_pipe();
	}
	Ok(())
}

fn main() -> std::io::Result<()> {
	let args: Vec<String> = env::args().collect();
	let mut opts = Options::new();
	opts.optflagopt("i", "input", "Set input ELF file", "ELF_PATH");
	opts.optflagopt("t", "req-pipe", "DRAMSim requesting pipe", "TRACE_PATH");
	opts.optflagopt("T", "resp-pipe", "DRAMSim responsing pipe", "TRACE_PATH");
	opts.optflag("h", "help", "Show this help menu");
	opts.optflagopt("m", "memory", "Set memory dump file", "MEMDUMP_PATH");
	// run_elf(args[1].clone())?;
	let mut mem_dump: String = "".to_string();
	match opts.parse(&args[1..]) {
		Ok(_args) => {
			#[cfg(feature = "memdump")]
			{
				match _args.opt_str("m") {
					Some(path) => {
						mem_dump = path.to_string();
					}
					_ => {
						println!("{}", opts.usage(&format!("{} [options]", args[0])));
						return Ok(());
					}
				}
			}

			#[cfg(feature = "dramsim")]
			{
				let request_pipe = match _args.opt_str("t") {
					Some(trace_path) => trace_path,
					_ => {
						println!("{}", opts.usage(&format!("{} [options]", args[0])));
						return Ok(());
					}
				};

				let response_pipe = match _args.opt_str("T") {
					Some(trace_path) => trace_path,
					_ => {
						println!("{}", opts.usage(&format!("{} [options]", args[0])));
						return Ok(());
					}
				};
				match setup_pipe(request_pipe.as_str(), response_pipe.as_str()) {
					0 => {}
					_ => {
						println!("{}", opts.usage(&format!("{} [options]", args[0])));
						return Ok(());
					}
				};
			}

			match _args.opt_str("i") {
				Some(input_path) => run_elf(input_path.as_str(), "", false, mem_dump.as_str())?,
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
