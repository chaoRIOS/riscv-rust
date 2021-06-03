WORK_DIR=$${HOME}/work
PROJ_DIR=$(WORK_DIR)/riscv-rust

DEBUG_PATH=$(PROJ_DIR)/target/debug
RELEASE_PATH=$(PROJ_DIR)/target/release

COREMARK_PATH = /home/zfu/rios/apd_from_lab1/apd/tb/test_program/coremark/coremark.riscv
DHRYSTONE_PATH = /home/zfu/rios/apd_from_lab1/apd/tb/test_program/benchmarks/dhrystone.riscv
TEST_PATH = /opt/cputest/lab3/64elfs

features=default
run_dhrystone:
	cargo run --release --features=$(features) -- -i $(DHRYSTONE_PATH)

run_coremark:
	cargo run --release --features=$(features) -- -i $(COREMARK_PATH)


testcase=all_arith0_elf64
run_test:
	cargo run --release --features=$(features) -- -i $(TEST_PATH)/$(testcase)