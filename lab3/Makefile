WORK_DIR=$${HOME}/work
PROJ_DIR=$(WORK_DIR)/riscv-rust

DEBUG_PATH=$(PROJ_DIR)/target/debug
RELEASE_PATH=$(PROJ_DIR)/target/release

COREMARK_PATH = /home/zfu/rios/apd_from_lab1/apd/tb/test_program/coremark/coremark.riscv

features=default
run_coremark:
	cargo run --release --features=$(features) -- -i $(COREMARK_PATH)