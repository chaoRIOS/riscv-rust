mod dummy_terminal;

use dummy_terminal::DummyTerminal;
use riscv_emu_rust::cpu::Xlen;
use riscv_emu_rust::cpu::Instruction;
use riscv_emu_rust::Emulator;

static emulator: Emulator = init_emu();


#[no_mangle]
pub extern "C" fn decode(data: u32) -> u32 {

    let instr: &Instruction = match emulator.get_cpu().decode_raw(data: u32){
        Ok(i)=>i,
        _=>panic!("decode failed"),
    };

    10:u32
}

fn init_emu() -> Emulator {
	let mut _emulator = Emulator::new(Box::new(DummyTerminal::new()))
    _emulator.update_xlen(Xlen::Bit64);
    _emulator
}
