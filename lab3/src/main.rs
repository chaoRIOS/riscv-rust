extern crate getopts;
extern crate riscv_emu_rust;

use riscv_emu_rust::cpu::Xlen;
use riscv_emu_rust::Emulator;

use getopts::Options;
use std::env;
use std::fs::File;
use std::io::Read;
use std::thread::Builder;

// @TODO: Optimize Cache memory usage
const STACK_SIZE: usize = 32 * 1024 * 1024;

fn run_elf(input_path: &str) -> std::io::Result<()> {
	let mut elf_file = File::open(input_path)?;
	let mut elf_contents = vec![];
	elf_file.read_to_end(&mut elf_contents)?;

	let mut emulator = Emulator::new();
	emulator.setup_program(elf_contents, vec![]);
	emulator.update_xlen(Xlen::Bit64);
	emulator.run_program();
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
			Some(input_path) => {
				let child = Builder::new()
					.stack_size(STACK_SIZE)
					.spawn(move || run_elf(input_path.as_str()))
					.unwrap();
				return child.join().unwrap();
			}
			_ => {
				println!("{}", opts.usage(&format!("{} [options]", args[0])));
			}
		},
		Err(f) => {
			println!("{}\n{}", f, opts.usage(&format!("{} [options]", args[0])));
		}
	};
	Ok(())
}
