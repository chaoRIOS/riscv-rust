WORK_DIR=$${HOME}/work
PROJ_DIR=$(WORK_DIR)/riscv-rust

DEBUG_PATH=$(PROJ_DIR)/target/debug
RELEASE_PATH=$(PROJ_DIR)/target/release


TEST_PATH = /opt/riscv-tests

RISCV_TEST_PATH = $(TEST_PATH)/benchmarks
rvtest=dhrystone

COREMARK_PATH = $(TEST_PATH)/coremark/coremark.riscv


features=default
run_rvtest:
	cargo run --release --features=$(features) -- -i $(RISCV_TEST_PATH)/$(rvtest).riscv

run_coremark:
	cargo run --release --features=$(features) -- -i $(COREMARK_PATH)


RIOS_TEST_PATH = /opt/cputest/lab3/64elfs
testcase=all_arith0_elf64
run_test:
	cargo run --release --features=$(features) -- -i $(RIOS_TEST_PATH)/$(testcase)

rls:
	cargo build --release --features=$(features)
	cp $(RELEASE_PATH)/lab3 $(RELEASE_PATH)/lab3_LTS