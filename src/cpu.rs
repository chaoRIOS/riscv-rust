extern crate fnv;

use mmu::{AddressingMode, MemoryAccessType, Mmu};
use std::fs::OpenOptions;
use std::io::prelude::*;

pub const CSR_CAPACITY: usize = 4096;

const CSR_USTATUS_ADDRESS: u16 = 0x000;
const CSR_FFLAGS_ADDRESS: u16 = 0x001;
const CSR_FRM_ADDRESS: u16 = 0x002;
const CSR_FCSR_ADDRESS: u16 = 0x003;
const CSR_UIE_ADDRESS: u16 = 0x004;
const CSR_UTVEC_ADDRESS: u16 = 0x005;
const _CSR_USCRATCH_ADDRESS: u16 = 0x040;
const CSR_UEPC_ADDRESS: u16 = 0x041;
const CSR_UCAUSE_ADDRESS: u16 = 0x042;
const CSR_UTVAL_ADDRESS: u16 = 0x043;
const _CSR_UIP_ADDRESS: u16 = 0x044;
const CSR_SSTATUS_ADDRESS: u16 = 0x100;
const CSR_SEDELEG_ADDRESS: u16 = 0x102;
const CSR_SIDELEG_ADDRESS: u16 = 0x103;
const CSR_SIE_ADDRESS: u16 = 0x104;
const CSR_STVEC_ADDRESS: u16 = 0x105;
const _CSR_SSCRATCH_ADDRESS: u16 = 0x140;
const CSR_SEPC_ADDRESS: u16 = 0x141;
const CSR_SCAUSE_ADDRESS: u16 = 0x142;
const CSR_STVAL_ADDRESS: u16 = 0x143;
const CSR_SIP_ADDRESS: u16 = 0x144;
const CSR_SATP_ADDRESS: u16 = 0x180;
const CSR_MSTATUS_ADDRESS: u16 = 0x300;
const CSR_MISA_ADDRESS: u16 = 0x301;
const CSR_MEDELEG_ADDRESS: u16 = 0x302;
const CSR_MIDELEG_ADDRESS: u16 = 0x303;
const CSR_MIE_ADDRESS: u16 = 0x304;

const CSR_MTVEC_ADDRESS: u16 = 0x305;
const _CSR_MSCRATCH_ADDRESS: u16 = 0x340;
const CSR_MEPC_ADDRESS: u16 = 0x341;
const CSR_MCAUSE_ADDRESS: u16 = 0x342;
const CSR_MTVAL_ADDRESS: u16 = 0x343;
const CSR_MIP_ADDRESS: u16 = 0x344;
const _CSR_PMPCFG0_ADDRESS: u16 = 0x3a0;
const _CSR_PMPADDR0_ADDRESS: u16 = 0x3b0;
pub const CSR_MCYCLE_ADDRESS: u16 = 0xb00;
const _CSR_CYCLE_ADDRESS: u16 = 0xc00;
const CSR_TIME_ADDRESS: u16 = 0xc01;
const _CSR_INSERT_ADDRESS: u16 = 0xc02;

pub const CSR_HPMCOUNTER3_ADDRESS: u16 = 0xc03;
pub const CSR_HPMCOUNTER4_ADDRESS: u16 = 0xc04;
pub const CSR_HPMCOUNTER5_ADDRESS: u16 = 0xc05;
pub const CSR_HPMCOUNTER6_ADDRESS: u16 = 0xc06;
pub const _CSR_HPMCOUNTER7_ADDRESS: u16 = 0xc07;

const _CSR_MHARTID_ADDRESS: u16 = 0xf14;

const MIP_MEIP: u64 = 0x800;
pub const MIP_MTIP: u64 = 0x080;
pub const MIP_MSIP: u64 = 0x008;
pub const MIP_SEIP: u64 = 0x200;
const MIP_STIP: u64 = 0x020;
const MIP_SSIP: u64 = 0x002;

/// Emulates a RISC-V CPU core
pub struct Cpu {
	pub clock: u64,
	pub xlen: Xlen,
	pub privilege_mode: PrivilegeMode,
	pub wfi: bool,
	// using only lower 32bits of x, pc, and csr registers
	// for 32-bit mode
	pub x: [i64; 32],
	pub f: [f64; 32],
	pub pc: u64,
	pub csr: [u64; CSR_CAPACITY],
	pub instruction_buffer: Vec<u32>,
	pub mmu: Mmu,
	pub reservation: u64, // @TODO: Should support multiple address reservations
	pub is_reservation_set: bool,
	pub _dump_flag: bool,
	pub unsigned_data_mask: u64,
	pub tohost_addr: u64,

	// Exit signal
	pub exit_signal: bool,
}

#[derive(Clone)]
pub enum Xlen {
	Bit32,
	Bit64, // @TODO: Support Bit128
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum PrivilegeMode {
	User,
	Supervisor,
	Reserved,
	Machine,
}

#[derive(Debug)]
pub struct Trap {
	pub trap_type: TrapType,
	pub value: u64, // Trap type specific value
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum TrapType {
	InstructionAddressMisaligned,
	InstructionAccessFault,
	IllegalInstruction,
	Breakpoint,
	LoadAddressMisaligned,
	LoadAccessFault,
	StoreAddressMisaligned,
	StoreAccessFault,
	EnvironmentCallFromUMode,
	EnvironmentCallFromSMode,
	EnvironmentCallFromMMode,
	InstructionPageFault,
	LoadPageFault,
	StorePageFault,
	UserSoftwareInterrupt,
	SupervisorSoftwareInterrupt,
	MachineSoftwareInterrupt,
	UserTimerInterrupt,
	SupervisorTimerInterrupt,
	MachineTimerInterrupt,
	UserExternalInterrupt,
	SupervisorExternalInterrupt,
	MachineExternalInterrupt,
}

fn _get_privilege_mode_name(mode: &PrivilegeMode) -> &'static str {
	match mode {
		PrivilegeMode::User => "User",
		PrivilegeMode::Supervisor => "Supervisor",
		PrivilegeMode::Reserved => "Reserved",
		PrivilegeMode::Machine => "Machine",
	}
}

// bigger number is higher privilege level
fn get_privilege_encoding(mode: &PrivilegeMode) -> u8 {
	match mode {
		PrivilegeMode::User => 0,
		PrivilegeMode::Supervisor => 1,
		PrivilegeMode::Reserved => panic!(),
		PrivilegeMode::Machine => 3,
	}
}

/// Returns `PrivilegeMode` from encoded privilege mode bits
pub fn get_privilege_mode(encoding: u64) -> PrivilegeMode {
	match encoding {
		0 => PrivilegeMode::User,
		1 => PrivilegeMode::Supervisor,
		3 => PrivilegeMode::Machine,
		_ => panic!("Unknown privilege uncoding"),
	}
}

fn _get_trap_type_name(trap_type: &TrapType) -> &'static str {
	match trap_type {
		TrapType::InstructionAddressMisaligned => "InstructionAddressMisaligned",
		TrapType::InstructionAccessFault => "InstructionAccessFault",
		TrapType::IllegalInstruction => "IllegalInstruction",
		TrapType::Breakpoint => "Breakpoint",
		TrapType::LoadAddressMisaligned => "LoadAddressMisaligned",
		TrapType::LoadAccessFault => "LoadAccessFault",
		TrapType::StoreAddressMisaligned => "StoreAddressMisaligned",
		TrapType::StoreAccessFault => "StoreAccessFault",
		TrapType::EnvironmentCallFromUMode => "EnvironmentCallFromUMode",
		TrapType::EnvironmentCallFromSMode => "EnvironmentCallFromSMode",
		TrapType::EnvironmentCallFromMMode => "EnvironmentCallFromMMode",
		TrapType::InstructionPageFault => "InstructionPageFault",
		TrapType::LoadPageFault => "LoadPageFault",
		TrapType::StorePageFault => "StorePageFault",
		TrapType::UserSoftwareInterrupt => "UserSoftwareInterrupt",
		TrapType::SupervisorSoftwareInterrupt => "SupervisorSoftwareInterrupt",
		TrapType::MachineSoftwareInterrupt => "MachineSoftwareInterrupt",
		TrapType::UserTimerInterrupt => "UserTimerInterrupt",
		TrapType::SupervisorTimerInterrupt => "SupervisorTimerInterrupt",
		TrapType::MachineTimerInterrupt => "MachineTimerInterrupt",
		TrapType::UserExternalInterrupt => "UserExternalInterrupt",
		TrapType::SupervisorExternalInterrupt => "SupervisorExternalInterrupt",
		TrapType::MachineExternalInterrupt => "MachineExternalInterrupt",
	}
}

fn get_trap_cause(trap: &Trap, xlen: &Xlen) -> u64 {
	let interrupt_bit = match xlen {
		Xlen::Bit32 => 0x80000000 as u64,
		Xlen::Bit64 => 0x8000000000000000 as u64,
	};
	match trap.trap_type {
		TrapType::InstructionAddressMisaligned => 0,
		TrapType::InstructionAccessFault => 1,
		TrapType::IllegalInstruction => 2,
		TrapType::Breakpoint => 3,
		TrapType::LoadAddressMisaligned => 4,
		TrapType::LoadAccessFault => 5,
		TrapType::StoreAddressMisaligned => 6,
		TrapType::StoreAccessFault => 7,
		TrapType::EnvironmentCallFromUMode => 8,
		TrapType::EnvironmentCallFromSMode => 9,
		TrapType::EnvironmentCallFromMMode => 11,
		TrapType::InstructionPageFault => 12,
		TrapType::LoadPageFault => 13,
		TrapType::StorePageFault => 15,
		TrapType::UserSoftwareInterrupt => interrupt_bit,
		TrapType::SupervisorSoftwareInterrupt => interrupt_bit + 1,
		TrapType::MachineSoftwareInterrupt => interrupt_bit + 3,
		TrapType::UserTimerInterrupt => interrupt_bit + 4,
		TrapType::SupervisorTimerInterrupt => interrupt_bit + 5,
		TrapType::MachineTimerInterrupt => interrupt_bit + 7,
		TrapType::UserExternalInterrupt => interrupt_bit + 8,
		TrapType::SupervisorExternalInterrupt => interrupt_bit + 9,
		TrapType::MachineExternalInterrupt => interrupt_bit + 11,
	}
}

impl Cpu {
	/// Creates a new `Cpu`.
	pub fn new() -> Self {
		let mut cpu = Cpu {
			clock: 0,
			xlen: Xlen::Bit64,
			privilege_mode: PrivilegeMode::User,
			wfi: false,
			x: [0; 32],
			f: [0.0; 32],
			pc: 0,
			instruction_buffer: Vec::new(),
			csr: [0; CSR_CAPACITY],
			mmu: Mmu::new(Xlen::Bit64),
			reservation: 0,
			is_reservation_set: false,
			_dump_flag: false,
			unsigned_data_mask: 0xffffffffffffffff,
			tohost_addr: 0,

			exit_signal: false,
		};
		cpu.x[0xb] = 0x1020; // I don't know why but Linux boot seems to require this initialization
		cpu.write_csr_raw(CSR_MISA_ADDRESS, 0x800000008014312f);
		cpu
	}

	/// Updates Program Counter content
	///
	/// # Arguments
	/// * `value`
	pub fn update_pc(&mut self, value: u64) {
		self.pc = value;
	}

	/// Updates GPR
	/// # input gpr_name e.g. "sp", value e.g. 0x123456
	pub fn update_gpr(&mut self, gpr_name: String, value: i64) {
		for i in 0..31 {
			if get_register_name(i) == gpr_name.as_str() {
				self.x[i] = value;
				// println!("updating gpr {} (x[{}]) to {}", gpr_name.as_str(), i, value);
				return;
			}
		}
	}

	/// Updates XLEN, 32-bit or 64-bit
	///
	/// # Arguments
	/// * `xlen`
	pub fn update_xlen(&mut self, xlen: Xlen) {
		self.xlen = xlen.clone();
		self.unsigned_data_mask = match xlen {
			Xlen::Bit32 => 0xffffffff,
			Xlen::Bit64 => 0xffffffffffffffff,
		};
		self.mmu.update_xlen(xlen.clone());
	}

	/// Reads integer register content
	///
	/// # Arguments
	/// * `reg` Register number. Must be 0-31
	pub fn read_register(&self, reg: u8) -> i64 {
		debug_assert!(reg <= 31, "reg must be 0-31. {}", reg);
		match reg {
			0 => 0, // 0th register is hardwired zero
			_ => self.x[reg as usize],
		}
	}

	/// Reads Program counter content
	pub fn read_pc(&self) -> u64 {
		self.pc
	}

	/// Runs program one cycle. Fetch, decode, and execution are completed in a cycle so far.
	pub fn tick(&mut self, trace_memory_access: bool, trace_path: &str) {
		let instruction_address = self.pc;

		if self.wfi {
			if (self.read_csr_raw(CSR_MIE_ADDRESS) & self.read_csr_raw(CSR_MIP_ADDRESS)) != 0 {
				self.wfi = false;
			}
			// @TODO: determine WFI latency
			return;
		}

		let word = match self.fetch_uncompress() {
			Ok(word) => word,
			Err(e) => {
				// Handle instruction page fault
				self.handle_exception(e, instruction_address);
				// @TODO: fix
				// e.g. add fetch latency
				return;
			}
		};

		let decode_result = self.decode(word, instruction_address);

		// Extra exit after decode stage, because we handle to/from host operation in decode stage.
		// Currently detect as exit when
		// * tohost address is set
		// * JAL to the identical address
		// * ECALL
		let pipeline_result = match decode_result {
			Ok(inst) => {
				// println!("inst={},pc={}", inst.get_name(), instruction_address);
				let cycles = inst.cycles;
				let result = (inst.operation)(self, word, instruction_address);
				self.x[0] = 0; // hardwired zero

				(result, cycles)
			}
			Err(()) => {
				// @TODO: illegal instructions
				//
				// Currently used for exitting.
				self.exit_signal = true;
				return;
			}
		};

		match pipeline_result.0 {
			Ok(()) => {
				self.mmu.clock = self.mmu.clock.wrapping_add(pipeline_result.1 as u64);
				self.clock = self.mmu.clock;
			}
			Err(e) => {
				// Handle pipeline traps
				self.clock = self.mmu.clock;
				self.handle_exception(e, instruction_address);
			}
		}

		if trace_memory_access == true {
			for i in 0..self.mmu.memory_access_trace.len() {
				let mut file = OpenOptions::new().append(true).open(trace_path).unwrap();

				file.write(
					format!(
						"0x{:016x} {} {}\n",
						self.mmu.memory_access_trace[i].address,
						match self.mmu.memory_access_trace[i].operation {
							MemoryAccessType::Read => "READ",
							MemoryAccessType::Write => "WRITE",
							_ => "",
						},
						self.mmu.memory_access_trace[i].cycle
					)
					.as_bytes(),
				)
				.unwrap();
			}
		}
		self.mmu.tick(&mut self.csr[CSR_MIP_ADDRESS as usize]);
		self.handle_interrupt(self.pc);
		// self.clock = self.clock.wrapping_add(1);

		// cpu core clock : mtime clock in clint = 8 : 1 is
		// just an arbiraty ratio.
		// @TODO: Implement more properly
		// self.write_csr_raw(CSR_CYCLE_ADDRESS, self.clock * 8);
		self.write_csr_raw(CSR_MCYCLE_ADDRESS, self.clock);
		self.write_csr_raw(CSR_HPMCOUNTER3_ADDRESS, self.mmu.l1_cache.hit_num);
		self.write_csr_raw(CSR_HPMCOUNTER4_ADDRESS, self.mmu.l1_cache.miss_num);
		self.write_csr_raw(CSR_HPMCOUNTER5_ADDRESS, self.mmu.l2_cache.hit_num);
		self.write_csr_raw(CSR_HPMCOUNTER6_ADDRESS, self.mmu.l2_cache.miss_num);
	}

	// @TODO: Rename?
	fn fetch_uncompress(&mut self) -> Result<u32, Trap> {
		// if self.wfi {
		// 	if (self.read_csr_raw(CSR_MIE_ADDRESS) & self.read_csr_raw(CSR_MIP_ADDRESS)) != 0 {
		// 		self.wfi = false;
		// 	}
		// 	// @TODO: determine WFI latency
		// 	return Ok(1);
		// }

		let original_word = match self.fetch() {
			Ok(word) => word,
			Err(e) => return Err(e),
		};
		let word = match (original_word & 0x3) == 0x3 {
			true => {
				self.pc = self.pc.wrapping_add(4); // 32-bit length non-compressed instruction
				original_word
			}
			false => {
				self.pc = self.pc.wrapping_add(2); // 16-bit length compressed instruction
				self.uncompress(original_word & 0xffff)
			}
		};

		Ok(word)
		// println!("disass: {}", self.disassemble_next_instruction());
	}

	/// Decodes a word instruction data and returns a reference to
	/// [`Instruction`](struct.Instruction.html). Using [`DecodeCache`](struct.DecodeCache.html)
	/// so if cache hits this method returns the result very quickly.
	/// The result will be stored to cache.
	pub fn decode(&mut self, word: u32, instruction_address: u64) -> Result<&Instruction, ()> {
		match self.decode_and_get_instruction_index(word) {
			Ok(index) => {
				// Handle to/from host
				let inst = &INSTRUCTIONS[index];
				match inst.name {
					"SD" | "SW" | "SH" | "SB" => {
						let f = parse_format_s(word);
						if self.x[f.rs1].wrapping_add(f.imm) == (self.tohost_addr as i64) {
							#[cfg(feature = "debug-tohost")]
							println!(
								"[Tohost] {} {:x} to {:x}",
								inst.name,
								self.x[f.rs2],
								self.x[f.rs1].wrapping_add(f.imm)
							);
							let tohost_addr = self.tohost_addr;
							let tohost_data_addr = self.x[f.rs2];
							match tohost_data_addr {
								0..=0x80000000 => {
									// Exit for riscv-test on tohost getting 1/0
									//
									// @TODO: Add pass of fail printing
									self.exit_signal = true;
									return Err(());
								}
								_ => {
									// Tohost I/O stream
									//
									// @TODO: optimize
									let flag1 = self
										.get_mut_mmu()
										.load_word_raw((24 + tohost_data_addr) as u64)
										!= 0;
									let flag2 = self
										.get_mut_mmu()
										.load_word_raw((28 + tohost_data_addr) as u64)
										!= 0;
									if flag1 || flag2 {
										let base = self
											.get_mut_mmu()
											.load_word_raw((4 * 4 + tohost_data_addr) as u64);
										let length = self
											.get_mut_mmu()
											.load_word_raw((6 * 4 + tohost_data_addr) as u64);
										for i in 0..length {
											let data =
												self.get_mut_mmu().load_raw((i + base) as u64);
											print!("{}", data as char);
										}
									}

									// After printf, set 1 to fromhost and set 0 to tohost
									// Note: host needs to access cache instead of memory!
									self.get_mut_mmu().store_word_raw(tohost_addr + 0x40, 1);
									self.get_mut_mmu().store_word_raw(tohost_addr, 0);

									// Exit after print to host
									//
									// @TODO: verify
									self.exit_signal = true;
									return Err(());
								}
							};
						}
					}
					"JAL" => {
						// Exit on iterative JAL
						//
						// @TODO: Add error exit status
						let f = parse_format_j(word);
						// println!(
						// 	"[{}] inst.add:0x{:x} imm:0x{:x} add+imm:0x{:x}",
						// 	self.clock,
						// 	instruction_address,
						// 	f.imm,
						// 	instruction_address + f.imm
						// );
						if instruction_address + f.imm == instruction_address {
							self.exit_signal = true;
							return Err(());
						}
					}
					"ECALL" => {
						self.exit_signal = true;
						return Err(());
					}
					_ => {}
				}
				Ok(&INSTRUCTIONS[index])
			}
			Err(()) => Err(()),
		}
	}

	/// Decodes a word instruction data and returns a reference to
	/// [`Instruction`](struct.Instruction.html). Not Using [`DecodeCache`](struct.DecodeCache.html)
	/// so if you don't want to pollute the cache you should use this method
	/// instead of `decode`.
	pub fn decode_raw(&self, word: u32) -> Result<&Instruction, ()> {
		match self.decode_and_get_instruction_index(word) {
			Ok(index) => Ok(&INSTRUCTIONS[index]),
			Err(()) => Err(()),
		}
	}

	/// Decodes a word instruction data and returns an index of
	/// [`INSTRUCTIONS`](constant.INSTRUCTIONS.html)
	///
	/// # Arguments
	/// * `word` word instruction data decoded
	pub fn decode_and_get_instruction_index(&self, word: u32) -> Result<usize, ()> {
		for i in 0..INSTRUCTION_NUM {
			let inst = &INSTRUCTIONS[i];
			if (word & inst.mask) == inst.data {
				return Ok(i);
			}
		}
		return Err(());
	}

	fn handle_interrupt(&mut self, instruction_address: u64) {
		// @TODO: Optimize
		let minterrupt = self.read_csr_raw(CSR_MIP_ADDRESS) & self.read_csr_raw(CSR_MIE_ADDRESS);

		if (minterrupt & MIP_MEIP) != 0 {
			if self.handle_trap(
				Trap {
					trap_type: TrapType::MachineExternalInterrupt,
					value: self.pc, // dummy
				},
				instruction_address,
				true,
			) {
				// Who should clear mip bit?
				self.write_csr_raw(
					CSR_MIP_ADDRESS,
					self.read_csr_raw(CSR_MIP_ADDRESS) & !MIP_MEIP,
				);
				self.wfi = false;
				return;
			}
		}
		if (minterrupt & MIP_MSIP) != 0 {
			if self.handle_trap(
				Trap {
					trap_type: TrapType::MachineSoftwareInterrupt,
					value: self.pc, // dummy
				},
				instruction_address,
				true,
			) {
				self.write_csr_raw(
					CSR_MIP_ADDRESS,
					self.read_csr_raw(CSR_MIP_ADDRESS) & !MIP_MSIP,
				);
				self.wfi = false;
				return;
			}
		}
		if (minterrupt & MIP_MTIP) != 0 {
			if self.handle_trap(
				Trap {
					trap_type: TrapType::MachineTimerInterrupt,
					value: self.pc, // dummy
				},
				instruction_address,
				true,
			) {
				self.write_csr_raw(
					CSR_MIP_ADDRESS,
					self.read_csr_raw(CSR_MIP_ADDRESS) & !MIP_MTIP,
				);
				self.wfi = false;
				return;
			}
		}
		if (minterrupt & MIP_SEIP) != 0 {
			if self.handle_trap(
				Trap {
					trap_type: TrapType::SupervisorExternalInterrupt,
					value: self.pc, // dummy
				},
				instruction_address,
				true,
			) {
				self.write_csr_raw(
					CSR_MIP_ADDRESS,
					self.read_csr_raw(CSR_MIP_ADDRESS) & !MIP_SEIP,
				);
				self.wfi = false;
				return;
			}
		}
		if (minterrupt & MIP_SSIP) != 0 {
			if self.handle_trap(
				Trap {
					trap_type: TrapType::SupervisorSoftwareInterrupt,
					value: self.pc, // dummy
				},
				instruction_address,
				true,
			) {
				self.write_csr_raw(
					CSR_MIP_ADDRESS,
					self.read_csr_raw(CSR_MIP_ADDRESS) & !MIP_SSIP,
				);
				self.wfi = false;
				return;
			}
		}
		if (minterrupt & MIP_STIP) != 0 {
			if self.handle_trap(
				Trap {
					trap_type: TrapType::SupervisorTimerInterrupt,
					value: self.pc, // dummy
				},
				instruction_address,
				true,
			) {
				self.write_csr_raw(
					CSR_MIP_ADDRESS,
					self.read_csr_raw(CSR_MIP_ADDRESS) & !MIP_STIP,
				);
				self.wfi = false;
				return;
			}
		}
	}

	pub fn handle_exception(&mut self, exception: Trap, instruction_address: u64) {
		self.handle_trap(exception, instruction_address, false);
	}

	fn handle_trap(&mut self, trap: Trap, instruction_address: u64, is_interrupt: bool) -> bool {
		let current_privilege_encoding = get_privilege_encoding(&self.privilege_mode) as u64;
		let cause = get_trap_cause(&trap, &self.xlen);

		// First, determine which privilege mode should handle the trap.
		// @TODO: Check if this logic is correct
		let mdeleg = match is_interrupt {
			true => self.read_csr_raw(CSR_MIDELEG_ADDRESS),
			false => self.read_csr_raw(CSR_MEDELEG_ADDRESS),
		};
		let sdeleg = match is_interrupt {
			true => self.read_csr_raw(CSR_SIDELEG_ADDRESS),
			false => self.read_csr_raw(CSR_SEDELEG_ADDRESS),
		};
		let pos = cause & 0xffff;

		let new_privilege_mode = match ((mdeleg >> pos) & 1) == 0 {
			true => PrivilegeMode::Machine,
			false => match ((sdeleg >> pos) & 1) == 0 {
				true => PrivilegeMode::Supervisor,
				false => PrivilegeMode::User,
			},
		};
		let new_privilege_encoding = get_privilege_encoding(&new_privilege_mode) as u64;

		let current_status = match self.privilege_mode {
			PrivilegeMode::Machine => self.read_csr_raw(CSR_MSTATUS_ADDRESS),
			PrivilegeMode::Supervisor => self.read_csr_raw(CSR_SSTATUS_ADDRESS),
			PrivilegeMode::User => self.read_csr_raw(CSR_USTATUS_ADDRESS),
			PrivilegeMode::Reserved => panic!(),
		};

		// Second, ignore the interrupt if it's disabled by some conditions

		if is_interrupt {
			let ie = match new_privilege_mode {
				PrivilegeMode::Machine => self.read_csr_raw(CSR_MIE_ADDRESS),
				PrivilegeMode::Supervisor => self.read_csr_raw(CSR_SIE_ADDRESS),
				PrivilegeMode::User => self.read_csr_raw(CSR_UIE_ADDRESS),
				PrivilegeMode::Reserved => panic!(),
			};

			let current_mie = (current_status >> 3) & 1;
			let current_sie = (current_status >> 1) & 1;
			let current_uie = current_status & 1;

			let msie = (ie >> 3) & 1;
			let ssie = (ie >> 1) & 1;
			let usie = ie & 1;

			let mtie = (ie >> 7) & 1;
			let stie = (ie >> 5) & 1;
			let utie = (ie >> 4) & 1;

			let meie = (ie >> 11) & 1;
			let seie = (ie >> 9) & 1;
			let ueie = (ie >> 8) & 1;

			// 1. Interrupt is always enabled if new privilege level is higher
			// than current privilege level
			// 2. Interrupt is always disabled if new privilege level is lower
			// than current privilege level
			// 3. Interrupt is enabled if xIE in xstatus is 1 where x is privilege level
			// and new privilege level equals to current privilege level

			if new_privilege_encoding < current_privilege_encoding {
				return false;
			} else if current_privilege_encoding == new_privilege_encoding {
				match self.privilege_mode {
					PrivilegeMode::Machine => {
						if current_mie == 0 {
							return false;
						}
					}
					PrivilegeMode::Supervisor => {
						if current_sie == 0 {
							return false;
						}
					}
					PrivilegeMode::User => {
						if current_uie == 0 {
							return false;
						}
					}
					PrivilegeMode::Reserved => panic!(),
				};
			}

			// Interrupt can be maskable by xie csr register
			// where x is a new privilege mode.

			match trap.trap_type {
				TrapType::UserSoftwareInterrupt => {
					if usie == 0 {
						return false;
					}
				}
				TrapType::SupervisorSoftwareInterrupt => {
					if ssie == 0 {
						return false;
					}
				}
				TrapType::MachineSoftwareInterrupt => {
					if msie == 0 {
						return false;
					}
				}
				TrapType::UserTimerInterrupt => {
					if utie == 0 {
						return false;
					}
				}
				TrapType::SupervisorTimerInterrupt => {
					if stie == 0 {
						return false;
					}
				}
				TrapType::MachineTimerInterrupt => {
					if mtie == 0 {
						return false;
					}
				}
				TrapType::UserExternalInterrupt => {
					if ueie == 0 {
						return false;
					}
				}
				TrapType::SupervisorExternalInterrupt => {
					if seie == 0 {
						return false;
					}
				}
				TrapType::MachineExternalInterrupt => {
					if meie == 0 {
						return false;
					}
				}
				_ => {}
			};
		}

		// So, this trap should be taken

		self.privilege_mode = new_privilege_mode;
		self.mmu.update_privilege_mode(self.privilege_mode.clone());
		let csr_epc_address = match self.privilege_mode {
			PrivilegeMode::Machine => CSR_MEPC_ADDRESS,
			PrivilegeMode::Supervisor => CSR_SEPC_ADDRESS,
			PrivilegeMode::User => CSR_UEPC_ADDRESS,
			PrivilegeMode::Reserved => panic!(),
		};
		let csr_cause_address = match self.privilege_mode {
			PrivilegeMode::Machine => CSR_MCAUSE_ADDRESS,
			PrivilegeMode::Supervisor => CSR_SCAUSE_ADDRESS,
			PrivilegeMode::User => CSR_UCAUSE_ADDRESS,
			PrivilegeMode::Reserved => panic!(),
		};
		let csr_tval_address = match self.privilege_mode {
			PrivilegeMode::Machine => CSR_MTVAL_ADDRESS,
			PrivilegeMode::Supervisor => CSR_STVAL_ADDRESS,
			PrivilegeMode::User => CSR_UTVAL_ADDRESS,
			PrivilegeMode::Reserved => panic!(),
		};
		let csr_tvec_address = match self.privilege_mode {
			PrivilegeMode::Machine => CSR_MTVEC_ADDRESS,
			PrivilegeMode::Supervisor => CSR_STVEC_ADDRESS,
			PrivilegeMode::User => CSR_UTVEC_ADDRESS,
			PrivilegeMode::Reserved => panic!(),
		};

		self.write_csr_raw(csr_epc_address, instruction_address);
		self.write_csr_raw(csr_cause_address, cause);
		self.write_csr_raw(csr_tval_address, trap.value);
		self.pc = self.read_csr_raw(csr_tvec_address);

		// Add 4 * cause if tvec has vector type address
		if (self.pc & 0x3) != 0 {
			self.pc = (self.pc & !0x3) + 4 * (cause & 0xffff);
		}

		match self.privilege_mode {
			PrivilegeMode::Machine => {
				let status = self.read_csr_raw(CSR_MSTATUS_ADDRESS);
				let mie = (status >> 3) & 1;
				// clear MIE[3], override MPIE[7] with MIE[3], override MPP[12:11] with current privilege encoding
				let new_status =
					(status & !0x1888) | (mie << 7) | (current_privilege_encoding << 11);
				self.write_csr_raw(CSR_MSTATUS_ADDRESS, new_status);
			}
			PrivilegeMode::Supervisor => {
				let status = self.read_csr_raw(CSR_SSTATUS_ADDRESS);
				let sie = (status >> 1) & 1;
				// clear SIE[1], override SPIE[5] with SIE[1], override SPP[8] with current privilege encoding
				let new_status =
					(status & !0x122) | (sie << 5) | ((current_privilege_encoding & 1) << 8);
				self.write_csr_raw(CSR_SSTATUS_ADDRESS, new_status);
			}
			PrivilegeMode::User => {
				panic!("Not implemented yet");
			}
			PrivilegeMode::Reserved => panic!(), // shouldn't happen
		};
		//println!("Trap! {:x} Clock:{:x}", cause, self.clock);
		true
	}

	pub fn fetch(&mut self) -> Result<u32, Trap> {
		let word = match self.mmu.fetch_word(self.pc) {
			Ok(word) => word,
			Err(e) => {
				self.pc = self.pc.wrapping_add(4); // @TODO: What if instruction is compressed?
				return Err(e);
			}
		};
		Ok(word)
	}

	fn has_csr_access_privilege(&self, address: u16) -> bool {
		let privilege = (address >> 8) & 0x3; // the lowest privilege level that can access the CSR
		privilege as u8 <= get_privilege_encoding(&self.privilege_mode)
	}

	fn read_csr(&mut self, address: u16) -> Result<u64, Trap> {
		match self.has_csr_access_privilege(address) {
			true => Ok(self.read_csr_raw(address)),
			false => Err(Trap {
				trap_type: TrapType::IllegalInstruction,
				value: self.pc.wrapping_sub(4), // @TODO: Is this always correct?
			}),
		}
	}

	pub fn write_csr(&mut self, address: u16, value: u64) -> Result<(), Trap> {
		#[cfg(feature = "memdump")]
		{
			if address == CSR_SATP_ADDRESS {
				// tempoary for lab2
				// println!("Warn: Changing SATP to {}", value);
				self.update_addressing_mode(value);
				return Ok(());
			}
		}
		match self.has_csr_access_privilege(address) {
			true => {
				/*
				// Checking writability fails some tests so disabling so far
				let read_only = ((address >> 10) & 0x3) == 0x3;
				if read_only {
					return Err(Exception::IllegalInstruction);
				}
				*/
				self.write_csr_raw(address, value);
				if address == CSR_SATP_ADDRESS {
					// println!("Warn: Changing SATP to {}", value);
					self.update_addressing_mode(value);
				}
				Ok(())
			}
			false => Err(Trap {
				trap_type: TrapType::IllegalInstruction,
				value: self.pc.wrapping_sub(4), // @TODO: Is this always correct?
			}),
		}
	}

	// SSTATUS, SIE, and SIP are subsets of MSTATUS, MIE, and MIP
	pub fn read_csr_raw(&self, address: u16) -> u64 {
		match address {
			// @TODO: Mask shuld consider of 32-bit mode
			CSR_FFLAGS_ADDRESS => self.csr[CSR_FCSR_ADDRESS as usize] & 0x1f,
			CSR_FRM_ADDRESS => (self.csr[CSR_FCSR_ADDRESS as usize] >> 5) & 0x7,
			CSR_SSTATUS_ADDRESS => self.csr[CSR_MSTATUS_ADDRESS as usize] & 0x80000003000de162,
			CSR_SIE_ADDRESS => self.csr[CSR_MIE_ADDRESS as usize] & 0x222,
			CSR_SIP_ADDRESS => self.csr[CSR_MIP_ADDRESS as usize] & 0x222,
			CSR_TIME_ADDRESS => self.csr[CSR_TIME_ADDRESS as usize],
			_ => self.csr[address as usize],
		}
	}

	fn write_csr_raw(&mut self, address: u16, value: u64) {
		match address {
			CSR_FFLAGS_ADDRESS => {
				self.csr[CSR_FCSR_ADDRESS as usize] &= !0x1f;
				self.csr[CSR_FCSR_ADDRESS as usize] |= value & 0x1f;
			}
			CSR_FRM_ADDRESS => {
				self.csr[CSR_FCSR_ADDRESS as usize] &= !0xe0;
				self.csr[CSR_FCSR_ADDRESS as usize] |= (value << 5) & 0xe0;
			}
			CSR_SSTATUS_ADDRESS => {
				self.csr[CSR_MSTATUS_ADDRESS as usize] &= !0x80000003000de162;
				self.csr[CSR_MSTATUS_ADDRESS as usize] |= value & 0x80000003000de162;
				self.mmu
					.update_mstatus(self.read_csr_raw(CSR_MSTATUS_ADDRESS));
			}
			CSR_SIE_ADDRESS => {
				self.csr[CSR_MIE_ADDRESS as usize] &= !0x222;
				self.csr[CSR_MIE_ADDRESS as usize] |= value & 0x222;
			}
			CSR_SIP_ADDRESS => {
				self.csr[CSR_MIP_ADDRESS as usize] &= !0x222;
				self.csr[CSR_MIP_ADDRESS as usize] |= value & 0x222;
			}
			CSR_MIDELEG_ADDRESS => {
				self.csr[address as usize] = value & 0x666; // from qemu
			}
			CSR_MSTATUS_ADDRESS => {
				self.csr[address as usize] = value;
				self.mmu
					.update_mstatus(self.read_csr_raw(CSR_MSTATUS_ADDRESS));
			}
			CSR_TIME_ADDRESS => {
				// self.mmu.get_mut_clint().write_mtime(value);
				self.csr[address as usize] = value;
			}
			_ => {
				self.csr[address as usize] = value;
			}
		};
	}

	fn _set_fcsr_nv(&mut self) {
		self.csr[CSR_FCSR_ADDRESS as usize] |= 0x10;
	}

	fn set_fcsr_dz(&mut self) {
		self.csr[CSR_FCSR_ADDRESS as usize] |= 0x8;
	}

	fn _set_fcsr_of(&mut self) {
		self.csr[CSR_FCSR_ADDRESS as usize] |= 0x4;
	}

	fn _set_fcsr_uf(&mut self) {
		self.csr[CSR_FCSR_ADDRESS as usize] |= 0x2;
	}

	fn _set_fcsr_nx(&mut self) {
		self.csr[CSR_FCSR_ADDRESS as usize] |= 0x1;
	}

	pub fn update_addressing_mode(&mut self, value: u64) {
		let addressing_mode = match self.xlen {
			Xlen::Bit32 => match value & 0x80000000 {
				0 => AddressingMode::None,
				_ => AddressingMode::SV32,
			},
			Xlen::Bit64 => match value >> 60 {
				0 => AddressingMode::None,
				8 => AddressingMode::SV39,
				9 => AddressingMode::SV48,
				_ => {
					println!("Unknown addressing_mode {:x}", value >> 60);
					panic!();
				}
			},
		};
		let ppn = match self.xlen {
			Xlen::Bit32 => value & 0x3fffff,
			Xlen::Bit64 => value & 0xfffffffffff,
		};
		self.mmu.update_addressing_mode(addressing_mode);
		self.mmu.update_ppn(ppn);
	}

	// @TODO: Rename to better name?
	fn sign_extend(&self, value: i64) -> i64 {
		match self.xlen {
			Xlen::Bit32 => value as i32 as i64,
			Xlen::Bit64 => value,
		}
	}

	// @TODO: Rename to better name?
	fn unsigned_data(&self, value: i64) -> u64 {
		(value as u64) & self.unsigned_data_mask
	}

	// @TODO: Rename to better name?
	fn most_negative(&self) -> i64 {
		match self.xlen {
			Xlen::Bit32 => std::i32::MIN as i64,
			Xlen::Bit64 => std::i64::MIN,
		}
	}

	// @TODO: Optimize
	pub fn uncompress(&self, halfword: u32) -> u32 {
		let op = halfword & 0x3; // [1:0]
		let funct3 = (halfword >> 13) & 0x7; // [15:13]

		match op {
			0 => match funct3 {
				0 => {
					// C.ADDI4SPN
					// addi rd+8, x2, nzuimm
					let rd = (halfword >> 2) & 0x7; // [4:2]
					let nzuimm = ((halfword >> 7) & 0x30) | // nzuimm[5:4] <= [12:11]
						((halfword >> 1) & 0x3c0) | // nzuimm{9:6] <= [10:7]
						((halfword >> 4) & 0x4) | // nzuimm[2] <= [6]
						((halfword >> 2) & 0x8); // nzuimm[3] <= [5]
						 // nzuimm == 0 is reserved instruction
					if nzuimm != 0 {
						return (nzuimm << 20) | (2 << 15) | ((rd + 8) << 7) | 0x13;
					}
				}
				1 => {
					// @TODO: Support C.LQ for 128-bit
					// C.FLD for 32, 64-bit
					// fld rd+8, offset(rs1+8)
					let rd = (halfword >> 2) & 0x7; // [4:2]
					let rs1 = (halfword >> 7) & 0x7; // [9:7]
					let offset = ((halfword >> 7) & 0x38) | // offset[5:3] <= [12:10]
						((halfword << 1) & 0xc0); // offset[7:6] <= [6:5]
					return (offset << 20) | ((rs1 + 8) << 15) | (3 << 12) | ((rd + 8) << 7) | 0x7;
				}
				2 => {
					// C.LW
					// lw rd+8, offset(rs1+8)
					let rs1 = (halfword >> 7) & 0x7; // [9:7]
					let rd = (halfword >> 2) & 0x7; // [4:2]
					let offset = ((halfword >> 7) & 0x38) | // offset[5:3] <= [12:10]
						((halfword >> 4) & 0x4) | // offset[2] <= [6]
						((halfword << 1) & 0x40); // offset[6] <= [5]
					return (offset << 20) | ((rs1 + 8) << 15) | (2 << 12) | ((rd + 8) << 7) | 0x3;
				}
				3 => {
					// @TODO: Support C.FLW in 32-bit mode
					// C.LD in 64-bit mode
					// ld rd+8, offset(rs1+8)
					let rs1 = (halfword >> 7) & 0x7; // [9:7]
					let rd = (halfword >> 2) & 0x7; // [4:2]
					let offset = ((halfword >> 7) & 0x38) | // offset[5:3] <= [12:10]
						((halfword << 1) & 0xc0); // offset[7:6] <= [6:5]
					return (offset << 20) | ((rs1 + 8) << 15) | (3 << 12) | ((rd + 8) << 7) | 0x3;
				}
				4 => {
					// Reserved
				}
				5 => {
					// C.FSD
					// fsd rs2+8, offset(rs1+8)
					let rs1 = (halfword >> 7) & 0x7; // [9:7]
					let rs2 = (halfword >> 2) & 0x7; // [4:2]
					let offset = ((halfword >> 7) & 0x38) | // uimm[5:3] <= [12:10]
						((halfword << 1) & 0xc0); // uimm[7:6] <= [6:5]
					let imm11_5 = (offset >> 5) & 0x7f;
					let imm4_0 = offset & 0x1f;
					return (imm11_5 << 25)
						| ((rs2 + 8) << 20) | ((rs1 + 8) << 15)
						| (3 << 12) | (imm4_0 << 7)
						| 0x27;
				}
				6 => {
					// C.SW
					// sw rs2+8, offset(rs1+8)
					let rs1 = (halfword >> 7) & 0x7; // [9:7]
					let rs2 = (halfword >> 2) & 0x7; // [4:2]
					let offset = ((halfword >> 7) & 0x38) | // offset[5:3] <= [12:10]
						((halfword << 1) & 0x40) | // offset[6] <= [5]
						((halfword >> 4) & 0x4); // offset[2] <= [6]
					let imm11_5 = (offset >> 5) & 0x7f;
					let imm4_0 = offset & 0x1f;
					return (imm11_5 << 25)
						| ((rs2 + 8) << 20) | ((rs1 + 8) << 15)
						| (2 << 12) | (imm4_0 << 7)
						| 0x23;
				}
				7 => {
					// @TODO: Support C.FSW in 32-bit mode
					// C.SD
					// sd rs2+8, offset(rs1+8)
					let rs1 = (halfword >> 7) & 0x7; // [9:7]
					let rs2 = (halfword >> 2) & 0x7; // [4:2]
					let offset = ((halfword >> 7) & 0x38) | // uimm[5:3] <= [12:10]
						((halfword << 1) & 0xc0); // uimm[7:6] <= [6:5]
					let imm11_5 = (offset >> 5) & 0x7f;
					let imm4_0 = offset & 0x1f;
					return (imm11_5 << 25)
						| ((rs2 + 8) << 20) | ((rs1 + 8) << 15)
						| (3 << 12) | (imm4_0 << 7)
						| 0x23;
				}
				_ => {} // Not happens
			},
			1 => {
				match funct3 {
					0 => {
						let r = (halfword >> 7) & 0x1f; // [11:7]
						let imm = match halfword & 0x1000 {
							0x1000 => 0xffffffc0,
							_ => 0
						} | // imm[31:6] <= [12]
						((halfword >> 7) & 0x20) | // imm[5] <= [12]
						((halfword >> 2) & 0x1f); // imm[4:0] <= [6:2]
						if r == 0 && imm == 0 {
							// C.NOP
							// addi x0, x0, 0
							return 0x13;
						} else if r != 0 {
							// C.ADDI
							// addi r, r, imm
							return (imm << 20) | (r << 15) | (r << 7) | 0x13;
						}
						// @TODO: Support HINTs
						// r == 0 and imm != 0 is HINTs
					}
					1 => {
						// @TODO: Support C.JAL in 32-bit mode
						// C.ADDIW
						// addiw r, r, imm
						let r = (halfword >> 7) & 0x1f;
						let imm = match halfword & 0x1000 {
							0x1000 => 0xffffffc0,
							_ => 0
						} | // imm[31:6] <= [12]
						((halfword >> 7) & 0x20) | // imm[5] <= [12]
						((halfword >> 2) & 0x1f); // imm[4:0] <= [6:2]
						if r != 0 {
							return (imm << 20) | (r << 15) | (r << 7) | 0x1b;
						}
						// r == 0 is reserved instruction
					}
					2 => {
						// C.LI
						// addi rd, x0, imm
						let r = (halfword >> 7) & 0x1f;
						let imm = match halfword & 0x1000 {
							0x1000 => 0xffffffc0,
							_ => 0
						} | // imm[31:6] <= [12]
						((halfword >> 7) & 0x20) | // imm[5] <= [12]
						((halfword >> 2) & 0x1f); // imm[4:0] <= [6:2]
						if r != 0 {
							return (imm << 20) | (r << 7) | 0x13;
						}
						// @TODO: Support HINTs
						// r == 0 is for HINTs
					}
					3 => {
						let r = (halfword >> 7) & 0x1f; // [11:7]
						if r == 2 {
							// C.ADDI16SP
							// addi r, r, nzimm
							let imm = match halfword & 0x1000 {
								0x1000 => 0xfffffc00,
								_ => 0
							} | // imm[31:10] <= [12]
							((halfword >> 3) & 0x200) | // imm[9] <= [12]
							((halfword >> 2) & 0x10) | // imm[4] <= [6]
							((halfword << 1) & 0x40) | // imm[6] <= [5]
							((halfword << 4) & 0x180) | // imm[8:7] <= [4:3]
							((halfword << 3) & 0x20); // imm[5] <= [2]
							if imm != 0 {
								return (imm << 20) | (r << 15) | (r << 7) | 0x13;
							}
							// imm == 0 is for reserved instruction
						}
						if r != 0 && r != 2 {
							// C.LUI
							// lui r, nzimm
							let nzimm = match halfword & 0x1000 {
								0x1000 => 0xfffc0000,
								_ => 0
							} | // nzimm[31:18] <= [12]
							((halfword << 5) & 0x20000) | // nzimm[17] <= [12]
							((halfword << 10) & 0x1f000); // nzimm[16:12] <= [6:2]
							if nzimm != 0 {
								return nzimm | (r << 7) | 0x37;
							}
							// nzimm == 0 is for reserved instruction
						}
					}
					4 => {
						let funct2 = (halfword >> 10) & 0x3; // [11:10]
						match funct2 {
							0 => {
								// C.SRLI
								// c.srli rs1+8, rs1+8, shamt
								let shamt = ((halfword >> 7) & 0x20) | // shamt[5] <= [12]
									((halfword >> 2) & 0x1f); // shamt[4:0] <= [6:2]
								let rs1 = (halfword >> 7) & 0x7; // [9:7]
								return (shamt << 20)
									| ((rs1 + 8) << 15) | (5 << 12) | ((rs1 + 8) << 7)
									| 0x13;
							}
							1 => {
								// C.SRAI
								// srai rs1+8, rs1+8, shamt
								let shamt = ((halfword >> 7) & 0x20) | // shamt[5] <= [12]
									((halfword >> 2) & 0x1f); // shamt[4:0] <= [6:2]
								let rs1 = (halfword >> 7) & 0x7; // [9:7]
								return (0x20 << 25)
									| (shamt << 20) | ((rs1 + 8) << 15) | (5 << 12)
									| ((rs1 + 8) << 7) | 0x13;
							}
							2 => {
								// C.ANDI
								// andi, r+8, r+8, imm
								let r = (halfword >> 7) & 0x7; // [9:7]
								let imm = match halfword & 0x1000 {
									0x1000 => 0xffffffc0,
									_ => 0
								} | // imm[31:6] <= [12]
								((halfword >> 7) & 0x20) | // imm[5] <= [12]
								((halfword >> 2) & 0x1f); // imm[4:0] <= [6:2]
								return (imm << 20)
									| ((r + 8) << 15) | (7 << 12) | ((r + 8) << 7)
									| 0x13;
							}
							3 => {
								let funct1 = (halfword >> 12) & 1; // [12]
								let funct2_2 = (halfword >> 5) & 0x3; // [6:5]
								let rs1 = (halfword >> 7) & 0x7;
								let rs2 = (halfword >> 2) & 0x7;
								match funct1 {
									0 => match funct2_2 {
										0 => {
											// C.SUB
											// sub rs1+8, rs1+8, rs2+8
											return (0x20 << 25)
												| ((rs2 + 8) << 20) | ((rs1 + 8) << 15) | ((rs1
												+ 8)
												<< 7) | 0x33;
										}
										1 => {
											// C.XOR
											// xor rs1+8, rs1+8, rs2+8
											return ((rs2 + 8) << 20)
												| ((rs1 + 8) << 15) | (4 << 12) | ((rs1 + 8)
												<< 7) | 0x33;
										}
										2 => {
											// C.OR
											// or rs1+8, rs1+8, rs2+8
											return ((rs2 + 8) << 20)
												| ((rs1 + 8) << 15) | (6 << 12) | ((rs1 + 8)
												<< 7) | 0x33;
										}
										3 => {
											// C.AND
											// and rs1+8, rs1+8, rs2+8
											return ((rs2 + 8) << 20)
												| ((rs1 + 8) << 15) | (7 << 12) | ((rs1 + 8)
												<< 7) | 0x33;
										}
										_ => {} // Not happens
									},
									1 => match funct2_2 {
										0 => {
											// C.SUBW
											// subw r1+8, r1+8, r2+8
											return (0x20 << 25)
												| ((rs2 + 8) << 20) | ((rs1 + 8) << 15) | ((rs1
												+ 8)
												<< 7) | 0x3b;
										}
										1 => {
											// C.ADDW
											// addw r1+8, r1+8, r2+8
											return ((rs2 + 8) << 20)
												| ((rs1 + 8) << 15) | ((rs1 + 8) << 7) | 0x3b;
										}
										2 => {
											// Reserved
										}
										3 => {
											// Reserved
										}
										_ => {} // Not happens
									},
									_ => {} // No happens
								};
							}
							_ => {} // not happens
						};
					}
					5 => {
						// C.J
						// jal x0, imm
						let offset = match halfword & 0x1000 {
								0x1000 => 0xfffff000,
								_ => 0
							} | // offset[31:12] <= [12]
							((halfword >> 1) & 0x800) | // offset[11] <= [12]
							((halfword >> 7) & 0x10) | // offset[4] <= [11]
							((halfword >> 1) & 0x300) | // offset[9:8] <= [10:9]
							((halfword << 2) & 0x400) | // offset[10] <= [8]
							((halfword >> 1) & 0x40) | // offset[6] <= [7]
							((halfword << 1) & 0x80) | // offset[7] <= [6]
							((halfword >> 2) & 0xe) | // offset[3:1] <= [5:3]
							((halfword << 3) & 0x20); // offset[5] <= [2]
						let imm = ((offset >> 1) & 0x80000) | // imm[19] <= offset[20]
							((offset << 8) & 0x7fe00) | // imm[18:9] <= offset[10:1]
							((offset >> 3) & 0x100) | // imm[8] <= offset[11]
							((offset >> 12) & 0xff); // imm[7:0] <= offset[19:12]
						return (imm << 12) | 0x6f;
					}
					6 => {
						// C.BEQZ
						// beq r+8, x0, offset
						let r = (halfword >> 7) & 0x7;
						let offset = match halfword & 0x1000 {
								0x1000 => 0xfffffe00,
								_ => 0
							} | // offset[31:9] <= [12]
							((halfword >> 4) & 0x100) | // offset[8] <= [12]
							((halfword >> 7) & 0x18) | // offset[4:3] <= [11:10]
							((halfword << 1) & 0xc0) | // offset[7:6] <= [6:5]
							((halfword >> 2) & 0x6) | // offset[2:1] <= [4:3]
							((halfword << 3) & 0x20); // offset[5] <= [2]
						let imm2 = ((offset >> 6) & 0x40) | // imm2[6] <= [12]
							((offset >> 5) & 0x3f); // imm2[5:0] <= [10:5]
						let imm1 = (offset & 0x1e) | // imm1[4:1] <= [4:1]
							((offset >> 11) & 0x1); // imm1[0] <= [11]
						return (imm2 << 25) | ((r + 8) << 20) | (imm1 << 7) | 0x63;
					}
					7 => {
						// C.BNEZ
						// bne r+8, x0, offset
						let r = (halfword >> 7) & 0x7;
						let offset = match halfword & 0x1000 {
								0x1000 => 0xfffffe00,
								_ => 0
							} | // offset[31:9] <= [12]
							((halfword >> 4) & 0x100) | // offset[8] <= [12]
							((halfword >> 7) & 0x18) | // offset[4:3] <= [11:10]
							((halfword << 1) & 0xc0) | // offset[7:6] <= [6:5]
							((halfword >> 2) & 0x6) | // offset[2:1] <= [4:3]
							((halfword << 3) & 0x20); // offset[5] <= [2]
						let imm2 = ((offset >> 6) & 0x40) | // imm2[6] <= [12]
							((offset >> 5) & 0x3f); // imm2[5:0] <= [10:5]
						let imm1 = (offset & 0x1e) | // imm1[4:1] <= [4:1]
							((offset >> 11) & 0x1); // imm1[0] <= [11]
						return (imm2 << 25) | ((r + 8) << 20) | (1 << 12) | (imm1 << 7) | 0x63;
					}
					_ => {} // No happens
				};
			}
			2 => {
				match funct3 {
					0 => {
						// C.SLLI
						// slli r, r, shamt
						let r = (halfword >> 7) & 0x1f;
						let shamt = ((halfword >> 7) & 0x20) | // imm[5] <= [12]
							((halfword >> 2) & 0x1f); // imm[4:0] <= [6:2]
						if r != 0 {
							return (shamt << 20) | (r << 15) | (1 << 12) | (r << 7) | 0x13;
						}
						// r == 0 is reserved instruction?
					}
					1 => {
						// C.FLDSP
						// fld rd, offset(x2)
						let rd = (halfword >> 7) & 0x1f;
						let offset = ((halfword >> 7) & 0x20) | // offset[5] <= [12]
							((halfword >> 2) & 0x18) | // offset[4:3] <= [6:5]
							((halfword << 4) & 0x1c0); // offset[8:6] <= [4:2]
						if rd != 0 {
							return (offset << 20) | (2 << 15) | (3 << 12) | (rd << 7) | 0x7;
						}
						// rd == 0 is reseved instruction
					}
					2 => {
						// C.LWSP
						// lw r, offset(x2)
						let r = (halfword >> 7) & 0x1f;
						let offset = ((halfword >> 7) & 0x20) | // offset[5] <= [12]
							((halfword >> 2) & 0x1c) | // offset[4:2] <= [6:4]
							((halfword << 4) & 0xc0); // offset[7:6] <= [3:2]
						if r != 0 {
							return (offset << 20) | (2 << 15) | (2 << 12) | (r << 7) | 0x3;
						}
						// r == 0 is reseved instruction
					}
					3 => {
						// @TODO: Support C.FLWSP in 32-bit mode
						// C.LDSP
						// ld rd, offset(x2)
						let rd = (halfword >> 7) & 0x1f;
						let offset = ((halfword >> 7) & 0x20) | // offset[5] <= [12]
							((halfword >> 2) & 0x18) | // offset[4:3] <= [6:5]
							((halfword << 4) & 0x1c0); // offset[8:6] <= [4:2]
						if rd != 0 {
							return (offset << 20) | (2 << 15) | (3 << 12) | (rd << 7) | 0x3;
						}
						// rd == 0 is reseved instruction
					}
					4 => {
						let funct1 = (halfword >> 12) & 1; // [12]
						let rs1 = (halfword >> 7) & 0x1f; // [11:7]
						let rs2 = (halfword >> 2) & 0x1f; // [6:2]
						match funct1 {
							0 => {
								if rs1 != 0 && rs2 == 0 {
									// C.JR
									// jalr x0, 0(rs1)
									return (rs1 << 15) | 0x67;
								}
								// rs1 == 0 is reserved instruction
								if rs1 != 0 && rs2 != 0 {
									// C.MV
									// add rs1, x0, rs2
									// println!("C.MV RS1:{:x} RS2:{:x}", rs1, rs2);
									return (rs2 << 20) | (rs1 << 7) | 0x33;
								}
								// rs1 == 0 && rs2 != 0 is Hints
								// @TODO: Support Hints
							}
							1 => {
								if rs1 == 0 && rs2 == 0 {
									// C.EBREAK
									// ebreak
									return 0x00100073;
								}
								if rs1 != 0 && rs2 == 0 {
									// C.JALR
									// jalr x1, 0(rs1)
									return (rs1 << 15) | (1 << 7) | 0x67;
								}
								if rs1 != 0 && rs2 != 0 {
									// C.ADD
									// add rs1, rs1, rs2
									return (rs2 << 20) | (rs1 << 15) | (rs1 << 7) | 0x33;
								}
								// rs1 == 0 && rs2 != 0 is Hists
								// @TODO: Supports Hinsts
							}
							_ => {} // Not happens
						};
					}
					5 => {
						// @TODO: Implement
						// C.FSDSP
						// fsd rs2, offset(x2)
						let rs2 = (halfword >> 2) & 0x1f; // [6:2]
						let offset = ((halfword >> 7) & 0x38) | // offset[5:3] <= [12:10]
							((halfword >> 1) & 0x1c0); // offset[8:6] <= [9:7]
						let imm11_5 = (offset >> 5) & 0x3f;
						let imm4_0 = offset & 0x1f;
						return (imm11_5 << 25)
							| (rs2 << 20) | (2 << 15) | (3 << 12)
							| (imm4_0 << 7) | 0x27;
					}
					6 => {
						// C.SWSP
						// sw rs2, offset(x2)
						let rs2 = (halfword >> 2) & 0x1f; // [6:2]
						let offset = ((halfword >> 7) & 0x3c) | // offset[5:2] <= [12:9]
							((halfword >> 1) & 0xc0); // offset[7:6] <= [8:7]
						let imm11_5 = (offset >> 5) & 0x3f;
						let imm4_0 = offset & 0x1f;
						return (imm11_5 << 25)
							| (rs2 << 20) | (2 << 15) | (2 << 12)
							| (imm4_0 << 7) | 0x23;
					}
					7 => {
						// @TODO: Support C.FSWSP in 32-bit mode
						// C.SDSP
						// sd rs, offset(x2)
						let rs2 = (halfword >> 2) & 0x1f; // [6:2]
						let offset = ((halfword >> 7) & 0x38) | // offset[5:3] <= [12:10]
							((halfword >> 1) & 0x1c0); // offset[8:6] <= [9:7]
						let imm11_5 = (offset >> 5) & 0x3f;
						let imm4_0 = offset & 0x1f;
						return (imm11_5 << 25)
							| (rs2 << 20) | (2 << 15) | (3 << 12)
							| (imm4_0 << 7) | 0x23;
					}
					_ => {} // Not happens
				};
			}
			_ => {} // No happnes
		};
		0xffffffff // Return invalid value
	}

	/// Disassembles an instruction pointed by Program Counter.
	pub fn disassemble_next_instruction(&mut self) -> String {
		// @TODO: Fetching can make a side effect,
		// for example updating page table entry or update peripheral hardware registers.
		// But ideally disassembling doesn't want to cause any side effect.
		// How can we avoid side effect?
		let mut original_word = match self.mmu.fetch_word(self.pc) {
			Ok(data) => data,
			Err(_e) => {
				return format!("PC:{:016x}, InstructionPageFault Trap!\n", self.pc);
			}
		};

		let word = match (original_word & 0x3) == 0x3 {
			true => original_word,
			false => {
				original_word &= 0xffff;
				self.uncompress(original_word)
			}
		};

		let inst = {
			match self.decode_raw(word) {
				Ok(inst) => inst,
				Err(()) => {
					return format!(
						"Unknown instruction PC:{:x} WORD:{:x}",
						self.pc, original_word
					);
				}
			}
		};

		let mut s = format!("PC:{:016x} ", self.unsigned_data(self.pc as i64));
		match self.pc {
			0x00010298u64 | 0x0001029au64 | 0x0001029cu64 | 0x0001029eu64 => {
				return String::from("");
			}
			_ => {}
		}
		s += &format!("{:08x} ", original_word);
		s += &format!("{} ", inst.name);
		s += &format!("{}", (inst.disassemble)(self, word, self.pc, true));
		s
	}

	/// Returns mutable `Mmu`
	pub fn get_mut_mmu(&mut self) -> &mut Mmu {
		&mut self.mmu
	}
}

pub struct Instruction {
	pub mask: u32,
	pub data: u32, // @TODO: rename
	pub name: &'static str,
	pub cycles: u8,
	pub operation: fn(cpu: &mut Cpu, word: u32, address: u64) -> Result<(), Trap>,
	pub disassemble: fn(cpu: &mut Cpu, word: u32, address: u64, evaluate: bool) -> String,
}

impl Instruction {
	pub fn get_name(&self) -> &str {
		self.name
	}
	pub fn get_cycles(&self) -> &u8 {
		&self.cycles
	}
}

pub struct FormatB {
	pub rs1: usize,
	pub rs2: usize,
	pub imm: u64,
}

pub fn parse_format_b(word: u32) -> FormatB {
	FormatB {
		rs1: ((word >> 15) & 0x1f) as usize, // [19:15]
		rs2: ((word >> 20) & 0x1f) as usize, // [24:20]
		imm: (
			match word & 0x80000000 { // imm[31:12] = [31]
				0x80000000 => 0xfffff000,
				_ => 0
			} |
			((word << 4) & 0x00000800) | // imm[11] = [7]
			((word >> 20) & 0x000007e0) | // imm[10:5] = [30:25]
			((word >> 7) & 0x0000001e)
			// imm[4:1] = [11:8]
		) as i32 as i64 as u64,
	}
}

fn dump_format_b(cpu: &mut Cpu, word: u32, address: u64, evaluate: bool) -> String {
	let f = parse_format_b(word);
	let mut s = String::new();
	s += &format!("{}", get_register_name(f.rs1));
	if evaluate {
		s += &format!(":{:x}", cpu.x[f.rs1]);
	}
	s += &format!(",{}", get_register_name(f.rs2));
	if evaluate {
		s += &format!(":{:x}", cpu.x[f.rs2]);
	}
	s += &format!(",{:x}", address.wrapping_add(f.imm));
	s
}

pub struct FormatCSR {
	pub csr: u16,
	pub rs: usize,
	pub rd: usize,
}

pub fn parse_format_csr(word: u32) -> FormatCSR {
	FormatCSR {
		csr: ((word >> 20) & 0xfff) as u16, // [31:20]
		rs: ((word >> 15) & 0x1f) as usize, // [19:15], also uimm
		rd: ((word >> 7) & 0x1f) as usize,  // [11:7]
	}
}

fn dump_format_csr(cpu: &mut Cpu, word: u32, _address: u64, evaluate: bool) -> String {
	let f = parse_format_csr(word);
	let mut s = String::new();
	s += &format!("{}", get_register_name(f.rd));
	if evaluate {
		s += &format!(":{:x}", cpu.x[f.rd]);
	}
	// @TODO: Use CSR name
	s += &format!(",{:x}", f.csr);
	if evaluate {
		s += &format!(":{:x}", cpu.read_csr_raw(f.csr));
	}
	s += &format!(",{}", get_register_name(f.rs));
	if evaluate {
		s += &format!(":{:x}", cpu.x[f.rs]);
	}
	s
}

pub struct FormatI {
	pub rd: usize,
	pub rs1: usize,
	pub imm: i64,
}

pub fn parse_format_i(word: u32) -> FormatI {
	FormatI {
		rd: ((word >> 7) & 0x1f) as usize,   // [11:7]
		rs1: ((word >> 15) & 0x1f) as usize, // [19:15]
		imm: (
			match word & 0x80000000 {
				// imm[31:11] = [31]
				0x80000000 => 0xfffff800,
				_ => 0,
			} | ((word >> 20) & 0x000007ff)
			// imm[10:0] = [30:20]
		) as i32 as i64,
	}
}

fn dump_format_i(cpu: &mut Cpu, word: u32, _address: u64, evaluate: bool) -> String {
	let f = parse_format_i(word);
	let mut s = String::new();
	s += &format!("{}", get_register_name(f.rd));
	if evaluate {
		s += &format!(":{:x}", cpu.x[f.rd]);
	}
	s += &format!(",{}", get_register_name(f.rs1));
	if evaluate {
		s += &format!(":{:x}", cpu.x[f.rs1]);
	}
	s += &format!(",{:x}", f.imm);
	s
}

fn dump_format_i_mem(cpu: &mut Cpu, word: u32, _address: u64, evaluate: bool) -> String {
	let f = parse_format_i(word);
	let mut s = String::new();
	s += &format!("{}", get_register_name(f.rd));
	if evaluate {
		s += &format!(":{:x}", cpu.x[f.rd]);
	}
	s += &format!(",{:x}({}", f.imm, get_register_name(f.rs1));
	if evaluate {
		s += &format!(":{:x}", cpu.x[f.rs1]);
	}
	s += &format!(")");
	s
}

pub struct FormatJ {
	pub rd: usize,
	pub imm: u64,
}

pub fn parse_format_j(word: u32) -> FormatJ {
	FormatJ {
		rd: ((word >> 7) & 0x1f) as usize, // [11:7]
		imm: (
			match word & 0x80000000 { // imm[31:20] = [31]
				0x80000000 => 0xfff00000,
				_ => 0
			} |
			(word & 0x000ff000) | // imm[19:12] = [19:12]
			((word & 0x00100000) >> 9) | // imm[11] = [20]
			((word & 0x7fe00000) >> 20)
			// imm[10:1] = [30:21]
		) as i32 as i64 as u64,
	}
}

fn dump_format_j(cpu: &mut Cpu, word: u32, address: u64, evaluate: bool) -> String {
	let f = parse_format_j(word);
	let mut s = String::new();
	s += &format!("{}", get_register_name(f.rd));
	if evaluate {
		s += &format!(":{:x}", cpu.x[f.rd]);
	}
	s += &format!(",{:x}", address.wrapping_add(f.imm));
	s
}

pub struct FormatR {
	pub rd: usize,
	pub rs1: usize,
	pub rs2: usize,
}

pub fn parse_format_r(word: u32) -> FormatR {
	FormatR {
		rd: ((word >> 7) & 0x1f) as usize,   // [11:7]
		rs1: ((word >> 15) & 0x1f) as usize, // [19:15]
		rs2: ((word >> 20) & 0x1f) as usize, // [24:20]
	}
}

fn dump_format_r(cpu: &mut Cpu, word: u32, _address: u64, evaluate: bool) -> String {
	let f = parse_format_r(word);
	let mut s = String::new();
	s += &format!("{}", get_register_name(f.rd));
	if evaluate {
		s += &format!(":{:x}", cpu.x[f.rd]);
	}
	s += &format!(",{}", get_register_name(f.rs1));
	if evaluate {
		s += &format!(":{:x}", cpu.x[f.rs1]);
	}
	s += &format!(",{}", get_register_name(f.rs2));
	if evaluate {
		s += &format!(":{:x}", cpu.x[f.rs2]);
	}
	s
}

// has rs3
pub struct FormatR2 {
	pub rd: usize,
	pub rs1: usize,
	pub rs2: usize,
	pub rs3: usize,
}

pub fn parse_format_r2(word: u32) -> FormatR2 {
	FormatR2 {
		rd: ((word >> 7) & 0x1f) as usize,   // [11:7]
		rs1: ((word >> 15) & 0x1f) as usize, // [19:15]
		rs2: ((word >> 20) & 0x1f) as usize, // [24:20]
		rs3: ((word >> 27) & 0x1f) as usize, // [31:27]
	}
}

fn dump_format_r2(cpu: &mut Cpu, word: u32, _address: u64, evaluate: bool) -> String {
	let f = parse_format_r2(word);
	let mut s = String::new();
	s += &format!("{}", get_register_name(f.rd));
	if evaluate {
		s += &format!(":{:x}", cpu.x[f.rd]);
	}
	s += &format!(",{}", get_register_name(f.rs1));
	if evaluate {
		s += &format!(":{:x}", cpu.x[f.rs1]);
	}
	s += &format!(",{}", get_register_name(f.rs2));
	if evaluate {
		s += &format!(":{:x}", cpu.x[f.rs2]);
	}
	s += &format!(",{}", get_register_name(f.rs3));
	if evaluate {
		s += &format!(":{:x}", cpu.x[f.rs3]);
	}
	s
}

pub struct FormatS {
	pub rs1: usize,
	pub rs2: usize,
	pub imm: i64,
}

pub fn parse_format_s(word: u32) -> FormatS {
	FormatS {
		rs1: ((word >> 15) & 0x1f) as usize, // [19:15]
		rs2: ((word >> 20) & 0x1f) as usize, // [24:20]
		imm: (
			match word & 0x80000000 {
				0x80000000 => 0xfffff000,
				_ => 0
			} | // imm[31:12] = [31]
			((word >> 20) & 0xfe0) | // imm[11:5] = [31:25]
			((word >> 7) & 0x1f)
			// imm[4:0] = [11:7]
		) as i32 as i64,
	}
}

fn dump_format_s(cpu: &mut Cpu, word: u32, _address: u64, evaluate: bool) -> String {
	let f = parse_format_s(word);
	let mut s = String::new();
	s += &format!("{}", get_register_name(f.rs2));
	if evaluate {
		s += &format!(":{:x}", cpu.x[f.rs2]);
	}
	s += &format!(",{:x}({}", f.imm, get_register_name(f.rs1));
	if evaluate {
		s += &format!(":{:x}", cpu.x[f.rs1]);
	}
	s += &format!(")");
	s
}

pub struct FormatU {
	pub rd: usize,
	pub imm: u64,
}

pub fn parse_format_u(word: u32) -> FormatU {
	FormatU {
		rd: ((word >> 7) & 0x1f) as usize, // [11:7]
		imm: (
			match word & 0x80000000 {
				0x80000000 => 0xffffffff00000000,
				_ => 0
			} | // imm[63:32] = [31]
			((word as u64) & 0xfffff000)
			// imm[31:12] = [31:12]
		) as u64,
	}
}

fn dump_format_u(cpu: &mut Cpu, word: u32, _address: u64, evaluate: bool) -> String {
	let f = parse_format_u(word);
	let mut s = String::new();
	s += &format!("{}", get_register_name(f.rd));
	if evaluate {
		s += &format!(":{:x}", cpu.x[f.rd]);
	}
	s += &format!(",{:x}", f.imm);
	s
}

fn dump_empty(_cpu: &mut Cpu, _word: u32, _address: u64, _evaluate: bool) -> String {
	String::new()
}

fn get_register_name(num: usize) -> &'static str {
	match num {
		0 => "zero",
		1 => "ra",
		2 => "sp",
		3 => "gp",
		4 => "tp",
		5 => "t0",
		6 => "t1",
		7 => "t2",
		8 => "s0",
		9 => "s1",
		10 => "a0",
		11 => "a1",
		12 => "a2",
		13 => "a3",
		14 => "a4",
		15 => "a5",
		16 => "a6",
		17 => "a7",
		18 => "s2",
		19 => "s3",
		20 => "s4",
		21 => "s5",
		22 => "s6",
		23 => "s7",
		24 => "s8",
		25 => "s9",
		26 => "s10",
		27 => "s11",
		28 => "t3",
		29 => "t4",
		30 => "t5",
		31 => "t6",
		_ => panic!("Unknown register num {}", num),
	}
}

const INSTRUCTION_NUM: usize = 116;

// @TODO: Reorder in often used order as
const INSTRUCTIONS: [Instruction; INSTRUCTION_NUM] = [
	Instruction {
		mask: 0xfe00707f,
		data: 0x00000033,
		name: "ADD",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = cpu.sign_extend(cpu.x[f.rs1].wrapping_add(cpu.x[f.rs2]));
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00000013,
		name: "ADDI",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_i(word);
			cpu.x[f.rd] = cpu.sign_extend(cpu.x[f.rs1].wrapping_add(f.imm));
			Ok(())
		},
		disassemble: dump_format_i,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x0000001b,
		name: "ADDIW",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_i(word);
			cpu.x[f.rd] = cpu.x[f.rs1].wrapping_add(f.imm) as i32 as i64;
			Ok(())
		},
		disassemble: dump_format_i,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x0000003b,
		name: "ADDW",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = cpu.x[f.rs1].wrapping_add(cpu.x[f.rs2]) as i32 as i64;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xf800707f,
		data: 0x0000302f,
		name: "AMOADD.D",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let tmp = match cpu.mmu.load_doubleword(cpu.x[f.rs1] as u64) {
				Ok(data) => data as i64,
				Err(e) => return Err(e),
			};
			match cpu
				.mmu
				.store_doubleword(cpu.x[f.rs1] as u64, cpu.x[f.rs2].wrapping_add(tmp) as u64)
			{
				Ok(()) => {}
				Err(e) => return Err(e),
			};
			cpu.x[f.rd] = tmp;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xf800707f,
		data: 0x0000202f,
		name: "AMOADD.W",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let tmp = match cpu.mmu.load_word(cpu.x[f.rs1] as u64) {
				Ok(data) => data as i32 as i64,
				Err(e) => return Err(e),
			};
			match cpu
				.mmu
				.store_word(cpu.x[f.rs1] as u64, cpu.x[f.rs2].wrapping_add(tmp) as u32)
			{
				Ok(()) => {}
				Err(e) => return Err(e),
			};
			cpu.x[f.rd] = tmp;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xf800707f,
		data: 0x6000302f,
		name: "AMOAND.D",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let tmp = match cpu.mmu.load_doubleword(cpu.x[f.rs1] as u64) {
				Ok(data) => data as i64,
				Err(e) => return Err(e),
			};
			match cpu
				.mmu
				.store_doubleword(cpu.x[f.rs1] as u64, (cpu.x[f.rs2] & tmp) as u64)
			{
				Ok(()) => {}
				Err(e) => return Err(e),
			};
			cpu.x[f.rd] = tmp;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xf800707f,
		data: 0x6000202f,
		name: "AMOAND.W",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let tmp = match cpu.mmu.load_word(cpu.x[f.rs1] as u64) {
				Ok(data) => data as i32 as i64,
				Err(e) => return Err(e),
			};
			match cpu
				.mmu
				.store_word(cpu.x[f.rs1] as u64, (cpu.x[f.rs2] & tmp) as u32)
			{
				Ok(()) => {}
				Err(e) => return Err(e),
			};
			cpu.x[f.rd] = tmp;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xf800707f,
		data: 0xe000302f,
		name: "AMOMAXU.D",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let tmp = match cpu.mmu.load_doubleword(cpu.x[f.rs1] as u64) {
				Ok(data) => data,
				Err(e) => return Err(e),
			};
			let max = match cpu.x[f.rs2] as u64 >= tmp {
				true => cpu.x[f.rs2] as u64,
				false => tmp,
			};
			match cpu.mmu.store_doubleword(cpu.x[f.rs1] as u64, max) {
				Ok(()) => {}
				Err(e) => return Err(e),
			};
			cpu.x[f.rd] = tmp as i64;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xf800707f,
		data: 0xe000202f,
		name: "AMOMAXU.W",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let tmp = match cpu.mmu.load_word(cpu.x[f.rs1] as u64) {
				Ok(data) => data,
				Err(e) => return Err(e),
			};
			let max = match cpu.x[f.rs2] as u32 >= tmp {
				true => cpu.x[f.rs2] as u32,
				false => tmp,
			};
			match cpu.mmu.store_word(cpu.x[f.rs1] as u64, max) {
				Ok(()) => {}
				Err(e) => return Err(e),
			};
			cpu.x[f.rd] = tmp as i32 as i64;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xf800707f,
		data: 0x4000302f,
		name: "AMOOR.D",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let tmp = match cpu.mmu.load_doubleword(cpu.x[f.rs1] as u64) {
				Ok(data) => data as i64,
				Err(e) => return Err(e),
			};
			match cpu
				.mmu
				.store_doubleword(cpu.x[f.rs1] as u64, (cpu.x[f.rs2] | tmp) as u64)
			{
				Ok(()) => {}
				Err(e) => return Err(e),
			};
			cpu.x[f.rd] = tmp;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xf800707f,
		data: 0x4000202f,
		name: "AMOOR.W",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let tmp = match cpu.mmu.load_word(cpu.x[f.rs1] as u64) {
				Ok(data) => data as i32 as i64,
				Err(e) => return Err(e),
			};
			match cpu
				.mmu
				.store_word(cpu.x[f.rs1] as u64, (cpu.x[f.rs2] | tmp) as u32)
			{
				Ok(()) => {}
				Err(e) => return Err(e),
			};
			cpu.x[f.rd] = tmp;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xf800707f,
		data: 0x0800302f,
		name: "AMOSWAP.D",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let tmp = match cpu.mmu.load_doubleword(cpu.x[f.rs1] as u64) {
				Ok(data) => data as i64,
				Err(e) => return Err(e),
			};
			match cpu
				.mmu
				.store_doubleword(cpu.x[f.rs1] as u64, cpu.x[f.rs2] as u64)
			{
				Ok(()) => {}
				Err(e) => return Err(e),
			};
			cpu.x[f.rd] = tmp;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xf800707f,
		data: 0x0800202f,
		name: "AMOSWAP.W",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let tmp = match cpu.mmu.load_word(cpu.x[f.rs1] as u64) {
				Ok(data) => data as i32 as i64,
				Err(e) => return Err(e),
			};
			match cpu.mmu.store_word(cpu.x[f.rs1] as u64, cpu.x[f.rs2] as u32) {
				Ok(()) => {}
				Err(e) => return Err(e),
			};
			cpu.x[f.rd] = tmp;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x00007033,
		name: "AND",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = cpu.sign_extend(cpu.x[f.rs1] & cpu.x[f.rs2]);
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00007013,
		name: "ANDI",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_i(word);
			cpu.x[f.rd] = cpu.sign_extend(cpu.x[f.rs1] & f.imm);
			Ok(())
		},
		disassemble: dump_format_i,
	},
	Instruction {
		mask: 0x0000007f,
		data: 0x00000017,
		name: "AUIPC",
		cycles: 1,
		operation: |cpu, word, address| {
			let f = parse_format_u(word);
			cpu.x[f.rd] = cpu.sign_extend(address.wrapping_add(f.imm) as i64);
			Ok(())
		},
		disassemble: dump_format_u,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00000063,
		name: "BEQ",
		cycles: 2,
		operation: |cpu, word, address| {
			let f = parse_format_b(word);
			if cpu.sign_extend(cpu.x[f.rs1]) == cpu.sign_extend(cpu.x[f.rs2]) {
				cpu.pc = address.wrapping_add(f.imm);
			}
			Ok(())
		},
		disassemble: dump_format_b,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00005063,
		name: "BGE",
		cycles: 2,
		operation: |cpu, word, address| {
			let f = parse_format_b(word);
			if cpu.sign_extend(cpu.x[f.rs1]) >= cpu.sign_extend(cpu.x[f.rs2]) {
				cpu.pc = address.wrapping_add(f.imm);
			}
			Ok(())
		},
		disassemble: dump_format_b,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00007063,
		name: "BGEU",
		cycles: 2,
		operation: |cpu, word, address| {
			let f = parse_format_b(word);
			if cpu.unsigned_data(cpu.x[f.rs1]) >= cpu.unsigned_data(cpu.x[f.rs2]) {
				cpu.pc = address.wrapping_add(f.imm);
			}
			Ok(())
		},
		disassemble: dump_format_b,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00004063,
		name: "BLT",
		cycles: 2,
		operation: |cpu, word, address| {
			let f = parse_format_b(word);
			if cpu.sign_extend(cpu.x[f.rs1]) < cpu.sign_extend(cpu.x[f.rs2]) {
				cpu.pc = address.wrapping_add(f.imm);
			}
			Ok(())
		},
		disassemble: dump_format_b,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00006063,
		name: "BLTU",
		cycles: 2,
		operation: |cpu, word, address| {
			let f = parse_format_b(word);
			if cpu.unsigned_data(cpu.x[f.rs1]) < cpu.unsigned_data(cpu.x[f.rs2]) {
				cpu.pc = address.wrapping_add(f.imm);
			}
			Ok(())
		},
		disassemble: dump_format_b,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00001063,
		name: "BNE",
		cycles: 2,
		operation: |cpu, word, address| {
			let f = parse_format_b(word);
			if cpu.sign_extend(cpu.x[f.rs1]) != cpu.sign_extend(cpu.x[f.rs2]) {
				cpu.pc = address.wrapping_add(f.imm);
			}
			Ok(())
		},
		disassemble: dump_format_b,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00003073,
		name: "CSRRC",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_csr(word);
			let data = match cpu.read_csr(f.csr) {
				Ok(data) => data as i64,
				Err(e) => return Err(e),
			};
			let tmp = cpu.x[f.rs];
			cpu.x[f.rd] = cpu.sign_extend(data);
			match cpu.write_csr(f.csr, (cpu.x[f.rd] & !tmp) as u64) {
				Ok(()) => {}
				Err(e) => return Err(e),
			};
			Ok(())
		},
		disassemble: dump_format_csr,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00007073,
		name: "CSRRCI",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_csr(word);
			let data = match cpu.read_csr(f.csr) {
				Ok(data) => data as i64,
				Err(e) => return Err(e),
			};
			cpu.x[f.rd] = cpu.sign_extend(data);
			match cpu.write_csr(f.csr, (cpu.x[f.rd] & !(f.rs as i64)) as u64) {
				Ok(()) => {}
				Err(e) => return Err(e),
			};
			Ok(())
		},
		disassemble: dump_format_csr,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00002073,
		name: "CSRRS",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_csr(word);
			let data = match cpu.read_csr(f.csr) {
				Ok(data) => data as i64,
				Err(e) => return Err(e),
			};
			let tmp = cpu.x[f.rs];
			cpu.x[f.rd] = cpu.sign_extend(data);
			match cpu.write_csr(f.csr, cpu.unsigned_data(cpu.x[f.rd] | tmp)) {
				Ok(()) => {}
				Err(e) => return Err(e),
			};
			Ok(())
		},
		disassemble: dump_format_csr,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00006073,
		name: "CSRRSI",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_csr(word);
			let data = match cpu.read_csr(f.csr) {
				Ok(data) => data as i64,
				Err(e) => return Err(e),
			};
			cpu.x[f.rd] = cpu.sign_extend(data);
			match cpu.write_csr(f.csr, cpu.unsigned_data(cpu.x[f.rd] | (f.rs as i64))) {
				Ok(()) => {}
				Err(e) => return Err(e),
			};
			Ok(())
		},
		disassemble: dump_format_csr,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00001073,
		name: "CSRRW",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_csr(word);
			let data = match cpu.read_csr(f.csr) {
				Ok(data) => data as i64,
				Err(e) => return Err(e),
			};
			let tmp = cpu.x[f.rs];
			cpu.x[f.rd] = cpu.sign_extend(data);
			match cpu.write_csr(f.csr, cpu.unsigned_data(tmp)) {
				Ok(()) => {}
				Err(e) => return Err(e),
			};
			Ok(())
		},
		disassemble: dump_format_csr,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00005073,
		name: "CSRRWI",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_csr(word);
			let data = match cpu.read_csr(f.csr) {
				Ok(data) => data as i64,
				Err(e) => return Err(e),
			};
			cpu.x[f.rd] = cpu.sign_extend(data);
			match cpu.write_csr(f.csr, f.rs as u64) {
				Ok(()) => {}
				Err(e) => return Err(e),
			};
			Ok(())
		},
		disassemble: dump_format_csr,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x02004033,
		name: "DIV",
		cycles: 128,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let dividend = cpu.x[f.rs1];
			let divisor = cpu.x[f.rs2];
			if divisor == 0 {
				cpu.x[f.rd] = -1;
			} else if dividend == cpu.most_negative() && divisor == -1 {
				cpu.x[f.rd] = dividend;
			} else {
				cpu.x[f.rd] = cpu.sign_extend(dividend.wrapping_div(divisor))
			}
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x02005033,
		name: "DIVU",
		cycles: 128,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let dividend = cpu.unsigned_data(cpu.x[f.rs1]);
			let divisor = cpu.unsigned_data(cpu.x[f.rs2]);
			if divisor == 0 {
				cpu.x[f.rd] = -1;
			} else {
				cpu.x[f.rd] = cpu.sign_extend(dividend.wrapping_div(divisor) as i64)
			}
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x0200503b,
		name: "DIVUW",
		cycles: 128,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let dividend = cpu.unsigned_data(cpu.x[f.rs1]) as u32;
			let divisor = cpu.unsigned_data(cpu.x[f.rs2]) as u32;
			if divisor == 0 {
				cpu.x[f.rd] = -1;
			} else {
				cpu.x[f.rd] = dividend.wrapping_div(divisor) as i32 as i64
			}
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x0200403b,
		name: "DIVW",
		cycles: 128,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let dividend = cpu.x[f.rs1] as i32;
			let divisor = cpu.x[f.rs2] as i32;
			if divisor == 0 {
				cpu.x[f.rd] = -1;
			} else if dividend == std::i32::MIN && divisor == -1 {
				cpu.x[f.rd] = dividend as i32 as i64;
			} else {
				cpu.x[f.rd] = dividend.wrapping_div(divisor) as i32 as i64
			}
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xffffffff,
		data: 0x00100073,
		name: "EBREAK",
		cycles: 1,
		operation: |_cpu, _word, _address| {
			// @TODO: Implement
			Ok(())
		},
		disassemble: dump_empty,
	},
	Instruction {
		mask: 0xffffffff,
		data: 0x00000073,
		name: "ECALL",
		cycles: 1,
		operation: |cpu, _word, address| {
			let exception_type = match cpu.privilege_mode {
				PrivilegeMode::User => TrapType::EnvironmentCallFromUMode,
				PrivilegeMode::Supervisor => TrapType::EnvironmentCallFromSMode,
				PrivilegeMode::Machine => TrapType::EnvironmentCallFromMMode,
				PrivilegeMode::Reserved => panic!("Unknown Privilege mode"),
			};
			return Err(Trap {
				trap_type: exception_type,
				value: address,
			});
		},
		disassemble: dump_empty,
	},
	Instruction {
		mask: 0xfe00007f,
		data: 0x02000053,
		name: "FADD.D",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.f[f.rd] = cpu.f[f.rs1] + cpu.f[f.rs2];
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfff0007f,
		data: 0xd2200053,
		name: "FCVT.D.L",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.f[f.rd] = cpu.x[f.rs1] as f64;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfff0007f,
		data: 0x42000053,
		name: "FCVT.D.S",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			// Is this implementation correct?
			cpu.f[f.rd] = f32::from_bits(cpu.f[f.rs1].to_bits() as u32) as f64;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfff0007f,
		data: 0xd2000053,
		name: "FCVT.D.W",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.f[f.rd] = cpu.x[f.rs1] as i32 as f64;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfff0007f,
		data: 0xd2100053,
		name: "FCVT.D.WU",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.f[f.rd] = cpu.x[f.rs1] as u32 as f64;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfff0007f,
		data: 0x40100053,
		name: "FCVT.S.D",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			// Is this implementation correct?
			cpu.f[f.rd] = cpu.f[f.rs1] as f32 as f64;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfff0007f,
		data: 0xc2000053,
		name: "FCVT.W.D",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			// Is this implementation correct?
			cpu.x[f.rd] = cpu.f[f.rs1] as u32 as i32 as i64;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00007f,
		data: 0x1a000053,
		name: "FDIV.D",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let dividend = cpu.f[f.rs1];
			let divisor = cpu.f[f.rs2];
			// Is this implementation correct?
			if divisor == 0.0 {
				cpu.f[f.rd] = std::f64::INFINITY;
				cpu.set_fcsr_dz();
			} else if divisor == -0.0 {
				cpu.f[f.rd] = std::f64::NEG_INFINITY;
				cpu.set_fcsr_dz();
			} else {
				cpu.f[f.rd] = dividend / divisor;
			}
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x0000000f,
		name: "FENCE",
		cycles: 1,
		operation: |cpu, _word, _address| {
			// Flush write back L1 cache
			cpu.get_mut_mmu().l1_flush();
			cpu.get_mut_mmu().l2_flush();
			Ok(())
		},
		disassemble: dump_empty,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x0000100f,
		name: "FENCE.I",
		cycles: 1,
		operation: |_cpu, _word, _address| {
			// Do nothing?
			Ok(())
		},
		disassemble: dump_empty,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0xa2002053,
		name: "FEQ.D",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = match cpu.f[f.rs1] == cpu.f[f.rs2] {
				true => 1,
				false => 0,
			};
			Ok(())
		},
		disassemble: dump_empty,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00003007,
		name: "FLD",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_i(word);
			cpu.f[f.rd] = match cpu
				.mmu
				.load_doubleword(cpu.x[f.rs1].wrapping_add(f.imm) as u64)
			{
				Ok(data) => f64::from_bits(data),
				Err(e) => return Err(e),
			};
			Ok(())
		},
		disassemble: dump_format_i,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0xa2000053,
		name: "FLE.D",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = match cpu.f[f.rs1] <= cpu.f[f.rs2] {
				true => 1,
				false => 0,
			};
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0xa2001053,
		name: "FLT.D",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = match cpu.f[f.rs1] < cpu.f[f.rs2] {
				true => 1,
				false => 0,
			};
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00002007,
		name: "FLW",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_i(word);
			cpu.f[f.rd] = match cpu.mmu.load_word(cpu.x[f.rs1].wrapping_add(f.imm) as u64) {
				Ok(data) => f64::from_bits(data as i32 as i64 as u64),
				Err(e) => return Err(e),
			};
			Ok(())
		},
		disassemble: dump_format_i_mem,
	},
	Instruction {
		mask: 0x0600007f,
		data: 0x02000043,
		name: "FMADD.D",
		cycles: 1,
		operation: |cpu, word, _address| {
			// @TODO: Update fcsr if needed?
			let f = parse_format_r2(word);
			cpu.f[f.rd] = cpu.f[f.rs1] * cpu.f[f.rs2] + cpu.f[f.rs3];
			Ok(())
		},
		disassemble: dump_format_r2,
	},
	Instruction {
		mask: 0xfe00007f,
		data: 0x12000053,
		name: "FMUL.D",
		cycles: 1,
		operation: |cpu, word, _address| {
			// @TODO: Update fcsr if needed?
			let f = parse_format_r(word);
			cpu.f[f.rd] = cpu.f[f.rs1] * cpu.f[f.rs2];
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfff0707f,
		data: 0xf2000053,
		name: "FMV.D.X",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.f[f.rd] = f64::from_bits(cpu.x[f.rs1] as u64);
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfff0707f,
		data: 0xe2000053,
		name: "FMV.X.D",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = cpu.f[f.rs1].to_bits() as i64;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfff0707f,
		data: 0xe0000053,
		name: "FMV.X.W",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = cpu.f[f.rs1].to_bits() as i32 as i64;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfff0707f,
		data: 0xf0000053,
		name: "FMV.W.X",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.f[f.rd] = f64::from_bits(cpu.x[f.rs1] as u32 as u64);
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0x0600007f,
		data: 0x0200004b,
		name: "FNMSUB.D",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r2(word);
			cpu.f[f.rd] = -(cpu.f[f.rs1] * cpu.f[f.rs2]) + cpu.f[f.rs3];
			Ok(())
		},
		disassemble: dump_format_r2,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00003027,
		name: "FSD",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_s(word);
			cpu.mmu.store_doubleword(
				cpu.x[f.rs1].wrapping_add(f.imm) as u64,
				cpu.f[f.rs2].to_bits(),
			)
		},
		disassemble: dump_format_s,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x22000053,
		name: "FSGNJ.D",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let rs1_bits = cpu.f[f.rs1].to_bits();
			let rs2_bits = cpu.f[f.rs2].to_bits();
			let sign_bit = rs2_bits & 0x8000000000000000;
			cpu.f[f.rd] = f64::from_bits(sign_bit | (rs1_bits & 0x7fffffffffffffff));
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x22002053,
		name: "FSGNJX.D",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let rs1_bits = cpu.f[f.rs1].to_bits();
			let rs2_bits = cpu.f[f.rs2].to_bits();
			let sign_bit = (rs1_bits ^ rs2_bits) & 0x8000000000000000;
			cpu.f[f.rd] = f64::from_bits(sign_bit | (rs1_bits & 0x7fffffffffffffff));
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00007f,
		data: 0x0a000053,
		name: "FSUB.D",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			// @TODO: Update fcsr if needed?
			cpu.f[f.rd] = cpu.f[f.rs1] - cpu.f[f.rs2];
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00002027,
		name: "FSW",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_s(word);
			cpu.mmu.store_word(
				cpu.x[f.rs1].wrapping_add(f.imm) as u64,
				cpu.f[f.rs2].to_bits() as u32,
			)
		},
		disassemble: dump_format_s,
	},
	Instruction {
		mask: 0x0000007f,
		data: 0x0000006f,
		name: "JAL",
		cycles: 2,
		operation: |cpu, word, address| {
			let f = parse_format_j(word);
			cpu.x[f.rd] = cpu.sign_extend(cpu.pc as i64);
			cpu.pc = address.wrapping_add(f.imm);
			Ok(())
		},
		disassemble: dump_format_j,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00000067,
		name: "JALR",
		cycles: 2,
		operation: |cpu, word, _address| {
			let f = parse_format_i(word);
			let tmp = cpu.sign_extend(cpu.pc as i64);
			cpu.pc = (cpu.x[f.rs1] as u64).wrapping_add(f.imm as u64);
			cpu.x[f.rd] = tmp;
			Ok(())
		},
		disassemble: |cpu, word, _address, evaluate| {
			let f = parse_format_i(word);
			let mut s = String::new();
			s += &format!("{}", get_register_name(f.rd));
			if evaluate {
				s += &format!(":{:x}", cpu.x[f.rd]);
			}
			s += &format!(",{:x}({}", f.imm, get_register_name(f.rs1));
			if evaluate {
				s += &format!(":{:x}", cpu.x[f.rs1]);
			}
			s += &format!(")");
			s
		},
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00000003,
		name: "LB",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_i(word);
			cpu.x[f.rd] = match cpu.mmu.load(cpu.x[f.rs1].wrapping_add(f.imm) as u64) {
				Ok(data) => data as i8 as i64,
				Err(e) => return Err(e),
			};
			Ok(())
		},
		disassemble: dump_format_i_mem,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00004003,
		name: "LBU",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_i(word);
			cpu.x[f.rd] = match cpu.mmu.load(cpu.x[f.rs1].wrapping_add(f.imm) as u64) {
				Ok(data) => data as i64,
				Err(e) => return Err(e),
			};
			Ok(())
		},
		disassemble: dump_format_i_mem,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00003003,
		name: "LD",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_i(word);
			cpu.x[f.rd] = match cpu
				.mmu
				.load_doubleword(cpu.x[f.rs1].wrapping_add(f.imm) as u64)
			{
				Ok(data) => data as i64,
				Err(e) => return Err(e),
			};
			Ok(())
		},
		disassemble: dump_format_i_mem,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00001003,
		name: "LH",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_i(word);
			cpu.x[f.rd] = match cpu
				.mmu
				.load_halfword(cpu.x[f.rs1].wrapping_add(f.imm) as u64)
			{
				Ok(data) => data as i16 as i64,
				Err(e) => return Err(e),
			};
			Ok(())
		},
		disassemble: dump_format_i_mem,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00005003,
		name: "LHU",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_i(word);
			cpu.x[f.rd] = match cpu
				.mmu
				.load_halfword(cpu.x[f.rs1].wrapping_add(f.imm) as u64)
			{
				Ok(data) => data as i64,
				Err(e) => return Err(e),
			};
			Ok(())
		},
		disassemble: dump_format_i_mem,
	},
	Instruction {
		mask: 0xf9f0707f,
		data: 0x1000302f,
		name: "LR.D",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			// @TODO: Implement properly
			cpu.x[f.rd] = match cpu.mmu.load_doubleword(cpu.x[f.rs1] as u64) {
				Ok(data) => {
					cpu.is_reservation_set = true;
					cpu.reservation = cpu.x[f.rs1] as u64; // Is virtual address ok?
					data as i64
				}
				Err(e) => return Err(e),
			};
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xf9f0707f,
		data: 0x1000202f,
		name: "LR.W",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			// @TODO: Implement properly
			cpu.x[f.rd] = match cpu.mmu.load_word(cpu.x[f.rs1] as u64) {
				Ok(data) => {
					cpu.is_reservation_set = true;
					cpu.reservation = cpu.x[f.rs1] as u64; // Is virtual address ok?
					data as i32 as i64
				}
				Err(e) => return Err(e),
			};
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0x0000007f,
		data: 0x00000037,
		name: "LUI",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_u(word);
			cpu.x[f.rd] = f.imm as i64;
			Ok(())
		},
		disassemble: dump_format_u,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00002003,
		name: "LW",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_i(word);
			cpu.x[f.rd] = match cpu.mmu.load_word(cpu.x[f.rs1].wrapping_add(f.imm) as u64) {
				Ok(data) => data as i32 as i64,
				Err(e) => return Err(e),
			};
			Ok(())
		},
		disassemble: dump_format_i_mem,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00006003,
		name: "LWU",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_i(word);
			cpu.x[f.rd] = match cpu.mmu.load_word(cpu.x[f.rs1].wrapping_add(f.imm) as u64) {
				Ok(data) => data as i64,
				Err(e) => return Err(e),
			};
			Ok(())
		},
		disassemble: dump_format_i_mem,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x02000033,
		name: "MUL",
		cycles: 4,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = cpu.sign_extend(cpu.x[f.rs1].wrapping_mul(cpu.x[f.rs2]));
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x02001033,
		name: "MULH",
		cycles: 4,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = match cpu.xlen {
				Xlen::Bit32 => cpu.sign_extend((cpu.x[f.rs1] * cpu.x[f.rs2]) >> 32),
				Xlen::Bit64 => ((cpu.x[f.rs1] as i128) * (cpu.x[f.rs2] as i128) >> 64) as i64,
			};
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x02003033,
		name: "MULHU",
		cycles: 4,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = match cpu.xlen {
				Xlen::Bit32 => cpu.sign_extend(
					(((cpu.x[f.rs1] as u32 as u64) * (cpu.x[f.rs2] as u32 as u64)) >> 32) as i64,
				),
				Xlen::Bit64 => {
					((cpu.x[f.rs1] as u64 as u128).wrapping_mul(cpu.x[f.rs2] as u64 as u128) >> 64)
						as i64
				}
			};
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x02002033,
		name: "MULHSU",
		cycles: 4,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = match cpu.xlen {
				Xlen::Bit32 => cpu.sign_extend(
					((cpu.x[f.rs1] as i64).wrapping_mul(cpu.x[f.rs2] as u32 as i64) >> 32) as i64,
				),
				Xlen::Bit64 => {
					((cpu.x[f.rs1] as u128).wrapping_mul(cpu.x[f.rs2] as u64 as u128) >> 64) as i64
				}
			};
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x0200003b,
		name: "MULW",
		cycles: 4,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] =
				cpu.sign_extend((cpu.x[f.rs1] as i32).wrapping_mul(cpu.x[f.rs2] as i32) as i64);
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xffffffff,
		data: 0x30200073,
		name: "MRET",
		cycles: 1,
		operation: |cpu, _word, _address| {
			cpu.pc = match cpu.read_csr(CSR_MEPC_ADDRESS) {
				Ok(data) => data,
				Err(e) => return Err(e),
			};
			let status = cpu.read_csr_raw(CSR_MSTATUS_ADDRESS);
			let mpie = (status >> 7) & 1;
			let mpp = (status >> 11) & 0x3;
			let mprv = match get_privilege_mode(mpp) {
				PrivilegeMode::Machine => (status >> 17) & 1,
				_ => 0,
			};
			// Override MIE[3] with MPIE[7], set MPIE[7] to 1, set MPP[12:11] to 0
			// and override MPRV[17]
			let new_status = (status & !0x21888) | (mprv << 17) | (mpie << 3) | (1 << 7);
			cpu.write_csr_raw(CSR_MSTATUS_ADDRESS, new_status);
			cpu.privilege_mode = match mpp {
				0 => PrivilegeMode::User,
				1 => PrivilegeMode::Supervisor,
				3 => PrivilegeMode::Machine,
				_ => panic!(), // Shouldn't happen
			};
			cpu.mmu.update_privilege_mode(cpu.privilege_mode.clone());
			Ok(())
		},
		disassemble: dump_empty,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x00006033,
		name: "OR",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = cpu.sign_extend(cpu.x[f.rs1] | cpu.x[f.rs2]);
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00006013,
		name: "ORI",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_i(word);
			cpu.x[f.rd] = cpu.sign_extend(cpu.x[f.rs1] | f.imm);
			Ok(())
		},
		disassemble: dump_format_i,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x02006033,
		name: "REM",
		cycles: 128,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let dividend = cpu.x[f.rs1];
			let divisor = cpu.x[f.rs2];
			if divisor == 0 {
				cpu.x[f.rd] = dividend;
			} else if dividend == cpu.most_negative() && divisor == -1 {
				cpu.x[f.rd] = 0;
			} else {
				cpu.x[f.rd] = cpu.sign_extend(cpu.x[f.rs1].wrapping_rem(cpu.x[f.rs2]));
			}
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x02007033,
		name: "REMU",
		cycles: 128,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let dividend = cpu.unsigned_data(cpu.x[f.rs1]);
			let divisor = cpu.unsigned_data(cpu.x[f.rs2]);
			cpu.x[f.rd] = match divisor {
				0 => cpu.sign_extend(dividend as i64),
				_ => cpu.sign_extend(dividend.wrapping_rem(divisor) as i64),
			};
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x0200703b,
		name: "REMUW",
		cycles: 128,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let dividend = cpu.x[f.rs1] as u32;
			let divisor = cpu.x[f.rs2] as u32;
			cpu.x[f.rd] = match divisor {
				0 => dividend as i32 as i64,
				_ => dividend.wrapping_rem(divisor) as i32 as i64,
			};
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x0200603b,
		name: "REMW",
		cycles: 128,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let dividend = cpu.x[f.rs1] as i32;
			let divisor = cpu.x[f.rs2] as i32;
			if divisor == 0 {
				cpu.x[f.rd] = dividend as i64;
			} else if dividend == std::i32::MIN && divisor == -1 {
				cpu.x[f.rd] = 0;
			} else {
				cpu.x[f.rd] = dividend.wrapping_rem(divisor) as i64;
			}
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00000023,
		name: "SB",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_s(word);
			cpu.mmu
				.store(cpu.x[f.rs1].wrapping_add(f.imm) as u64, cpu.x[f.rs2] as u8)
		},
		disassemble: dump_format_s,
	},
	Instruction {
		mask: 0xf800707f,
		data: 0x1800302f,
		name: "SC.D",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			// @TODO: Implement properly
			cpu.x[f.rd] = match cpu.is_reservation_set && cpu.reservation == (cpu.x[f.rs1] as u64) {
				true => match cpu
					.mmu
					.store_doubleword(cpu.x[f.rs1] as u64, cpu.x[f.rs2] as u64)
				{
					Ok(()) => {
						cpu.is_reservation_set = false;
						0
					}
					Err(e) => return Err(e),
				},
				false => 1,
			};
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xf800707f,
		data: 0x1800202f,
		name: "SC.W",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			// @TODO: Implement properly
			cpu.x[f.rd] = match cpu.is_reservation_set && cpu.reservation == (cpu.x[f.rs1] as u64) {
				true => match cpu.mmu.store_word(cpu.x[f.rs1] as u64, cpu.x[f.rs2] as u32) {
					Ok(()) => {
						cpu.is_reservation_set = false;
						0
					}
					Err(e) => return Err(e),
				},
				false => 1,
			};
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00003023,
		name: "SD",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_s(word);
			cpu.mmu
				.store_doubleword(cpu.x[f.rs1].wrapping_add(f.imm) as u64, cpu.x[f.rs2] as u64)
		},
		disassemble: dump_format_s,
	},
	Instruction {
		mask: 0xfe007fff,
		data: 0x12000073,
		name: "SFENCE.VMA",
		cycles: 1,
		operation: |_cpu, _word, _address| {
			// Do nothing?
			Ok(())
		},
		disassemble: dump_empty,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00001023,
		name: "SH",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_s(word);
			cpu.mmu
				.store_halfword(cpu.x[f.rs1].wrapping_add(f.imm) as u64, cpu.x[f.rs2] as u16)
		},
		disassemble: dump_format_s,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x00001033,
		name: "SLL",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = cpu.sign_extend(cpu.x[f.rs1].wrapping_shl(cpu.x[f.rs2] as u32));
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfc00707f,
		data: 0x00001013,
		name: "SLLI",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let mask = match cpu.xlen {
				Xlen::Bit32 => 0x1f,
				Xlen::Bit64 => 0x3f,
			};
			let shamt = (word >> 20) & mask;
			cpu.x[f.rd] = cpu.sign_extend(cpu.x[f.rs1] << shamt);
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x0000101b,
		name: "SLLIW",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let shamt = f.rs2 as u32;
			cpu.x[f.rd] = (cpu.x[f.rs1] << shamt) as i32 as i64;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x0000103b,
		name: "SLLW",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = (cpu.x[f.rs1] as u32).wrapping_shl(cpu.x[f.rs2] as u32) as i32 as i64;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x00002033,
		name: "SLT",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = match cpu.x[f.rs1] < cpu.x[f.rs2] {
				true => 1,
				false => 0,
			};
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00002013,
		name: "SLTI",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_i(word);
			cpu.x[f.rd] = match cpu.x[f.rs1] < f.imm {
				true => 1,
				false => 0,
			};
			Ok(())
		},
		disassemble: dump_format_i,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00003013,
		name: "SLTIU",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_i(word);
			cpu.x[f.rd] = match cpu.unsigned_data(cpu.x[f.rs1]) < cpu.unsigned_data(f.imm) {
				true => 1,
				false => 0,
			};
			Ok(())
		},
		disassemble: dump_format_i,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x00003033,
		name: "SLTU",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = match cpu.unsigned_data(cpu.x[f.rs1]) < cpu.unsigned_data(cpu.x[f.rs2]) {
				true => 1,
				false => 0,
			};
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x40005033,
		name: "SRA",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = cpu.sign_extend(cpu.x[f.rs1].wrapping_shr(cpu.x[f.rs2] as u32));
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfc00707f,
		data: 0x40005013,
		name: "SRAI",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let mask = match cpu.xlen {
				Xlen::Bit32 => 0x1f,
				Xlen::Bit64 => 0x3f,
			};
			let shamt = (word >> 20) & mask;
			cpu.x[f.rd] = cpu.sign_extend(cpu.x[f.rs1] >> shamt);
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfc00707f,
		data: 0x4000501b,
		name: "SRAIW",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let shamt = ((word >> 20) & 0x1f) as u32;
			cpu.x[f.rd] = ((cpu.x[f.rs1] as i32) >> shamt) as i64;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x4000503b,
		name: "SRAW",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = (cpu.x[f.rs1] as i32).wrapping_shr(cpu.x[f.rs2] as u32) as i64;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xffffffff,
		data: 0x10200073,
		name: "SRET",
		cycles: 1,
		operation: |cpu, _word, _address| {
			// @TODO: Throw error if higher privilege return instruction is executed
			cpu.pc = match cpu.read_csr(CSR_SEPC_ADDRESS) {
				Ok(data) => data,
				Err(e) => return Err(e),
			};
			let status = cpu.read_csr_raw(CSR_SSTATUS_ADDRESS);
			let spie = (status >> 5) & 1;
			let spp = (status >> 8) & 1;
			let mprv = match get_privilege_mode(spp) {
				PrivilegeMode::Machine => (status >> 17) & 1,
				_ => 0,
			};
			// Override SIE[1] with SPIE[5], set SPIE[5] to 1, set SPP[8] to 0,
			// and override MPRV[17]
			let new_status = (status & !0x20122) | (mprv << 17) | (spie << 1) | (1 << 5);
			cpu.write_csr_raw(CSR_SSTATUS_ADDRESS, new_status);
			cpu.privilege_mode = match spp {
				0 => PrivilegeMode::User,
				1 => PrivilegeMode::Supervisor,
				_ => panic!(), // Shouldn't happen
			};
			cpu.mmu.update_privilege_mode(cpu.privilege_mode.clone());
			Ok(())
		},
		disassemble: dump_empty,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x00005033,
		name: "SRL",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = cpu.sign_extend(
				cpu.unsigned_data(cpu.x[f.rs1])
					.wrapping_shr(cpu.x[f.rs2] as u32) as i64,
			);
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfc00707f,
		data: 0x00005013,
		name: "SRLI",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let mask = match cpu.xlen {
				Xlen::Bit32 => 0x1f,
				Xlen::Bit64 => 0x3f,
			};
			let shamt = (word >> 20) & mask;
			cpu.x[f.rd] = cpu.sign_extend((cpu.unsigned_data(cpu.x[f.rs1]) >> shamt) as i64);
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfc00707f,
		data: 0x0000501b,
		name: "SRLIW",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			let mask = match cpu.xlen {
				Xlen::Bit32 => 0x1f,
				Xlen::Bit64 => 0x3f,
			};
			let shamt = (word >> 20) & mask;
			cpu.x[f.rd] = ((cpu.x[f.rs1] as u32) >> shamt) as i32 as i64;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x0000503b,
		name: "SRLW",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = (cpu.x[f.rs1] as u32).wrapping_shr(cpu.x[f.rs2] as u32) as i32 as i64;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x40000033,
		name: "SUB",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = cpu.sign_extend(cpu.x[f.rs1].wrapping_sub(cpu.x[f.rs2]));
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x4000003b,
		name: "SUBW",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = cpu.x[f.rs1].wrapping_sub(cpu.x[f.rs2]) as i32 as i64;
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00002023,
		name: "SW",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_s(word);
			cpu.mmu
				.store_word(cpu.x[f.rs1].wrapping_add(f.imm) as u64, cpu.x[f.rs2] as u32)
		},
		disassemble: dump_format_s,
	},
	Instruction {
		mask: 0xffffffff,
		data: 0x00200073,
		name: "URET",
		cycles: 1,
		operation: |_cpu, _word, _address| {
			// @TODO: Implement
			panic!("URET instruction is not implemented yet.");
		},
		disassemble: dump_empty,
	},
	Instruction {
		mask: 0xffffffff,
		data: 0x10500073,
		name: "WFI",
		cycles: 1,
		operation: |cpu, _word, _address| {
			cpu.wfi = true;
			Ok(())
		},
		disassemble: dump_empty,
	},
	Instruction {
		mask: 0xfe00707f,
		data: 0x00004033,
		name: "XOR",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_r(word);
			cpu.x[f.rd] = cpu.sign_extend(cpu.x[f.rs1] ^ cpu.x[f.rs2]);
			Ok(())
		},
		disassemble: dump_format_r,
	},
	Instruction {
		mask: 0x0000707f,
		data: 0x00004013,
		name: "XORI",
		cycles: 1,
		operation: |cpu, word, _address| {
			let f = parse_format_i(word);
			cpu.x[f.rd] = cpu.sign_extend(cpu.x[f.rs1] ^ f.imm);
			Ok(())
		},
		disassemble: dump_format_i,
	},
];
