COREMARK_PATH = /home/zfu/rios/apd_from_lab1/apd/tb/test_program/coremark/coremark.riscv
DHRYSTONE_PATH = /home/zfu/rios/apd_from_lab1/apd/tb/test_program/benchmarks/dhrystone.riscv
PHYDUMP_PATH = /home/ychen/riscv-rust/lab2/phydump.txt


run_phydump:
	cargo run --release -- -m $(PHYDUMP_PATH) -i $(COREMARK_PATH)

run_coremark:
	cargo run --release -- -i $(COREMARK_PATH)

run_dhrystone:
	cargo run --release -- -i $(DHRYSTONE_PATH)