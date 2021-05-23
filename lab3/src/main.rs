extern crate getopts;
extern crate riscv_emu_rust;

extern crate lab1;

use riscv_emu_rust::cpu::Xlen;

use getopts::Options;
use std::env;
use std::fs::File;
use std::io::Read;

use lab1::pkg::*;

fn run_elf(input_path: &str) -> std::io::Result<()> {
	let mut elf_file = File::open(input_path)?;
	let mut elf_contents = vec![];
	elf_file.read_to_end(&mut elf_contents)?;
	unsafe {
		EMULATOR.setup_program(elf_contents, vec![]);
		EMULATOR.update_xlen(Xlen::Bit64);
		EMULATOR.run_program(false, "");
	}
	Ok(())
}

fn main() -> std::io::Result<()> {
	let args: Vec<String> = env::args().collect();
	let mut opts = Options::new();
	opts.optflagopt("i", "input", "Set input ELF file", "ELF_PATH");
	opts.optflag("h", "help", "Show this help menu");
	// run_elf(args[1].clone())?;
	match opts.parse(&args[1..]) {
		Ok(_args) => match _args.opt_str("i") {
			Some(input_path) => run_elf(input_path.as_str())?,
			_ => {
				println!("{}", opts.usage(&format!("{} [options]", args[0])));
				return Ok(());
			}
		},
		Err(f) => {
			println!("{}\n{}", f, opts.usage(&format!("{} [options]", args[0])));
			return Ok(());
		}
	};
	Ok(())
}
