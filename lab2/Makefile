WORK_DIR=/home/cwang/work
PROJ_DIR=$(WORK_DIR)/riscv-rust

DEBUG_PATH=$(PROJ_DIR)/target/debug
RELEASE_PATH=$(PROJ_DIR)/target/release

COREMARK_PATH = /home/zfu/rios/apd_from_lab1/apd/tb/test_program/coremark/coremark.riscv
DHRYSTONE_PATH = /home/zfu/rios/apd_from_lab1/apd/tb/test_program/benchmarks/dhrystone.riscv
ADD_PATH = /home/zfu/rios/apd_from_lab1/apd/tb/riscv-tests/isa/rv64ui-v-add
PHYDUMP_PATH = /home/cwang/work/riscv-rust/lab2/phydump.txt

TRACE_PATH = /home/cwang/work/riscv-rust/lab2/trace.dump

cmodule = interface


run_phydump:
	cargo run --release -- -m $(PHYDUMP_PATH) -i $(COREMARK_PATH)

run_coremark:
	cargo run --release -- -m $(PHYDUMP_PATH) -i $(COREMARK_PATH) -t $(TRACE_PATH)

run_dhrystone:
	cargo run --release -- -i $(DHRYSTONE_PATH) 


run_add:
	RUSTFLAGS="-L $(PROJ_DIR)/lab2/csrc"  cargo build --release 
	LD_LIBRARY_PATH="$(PROJ_DIR)/lab2/csrc" $(RELEASE_PATH)/lab2 -m $(PHYDUMP_PATH) -i $(ADD_PATH) 

test:
	$(MAKE) -C csrc $(cmodule)
	RUSTFLAGS="-L $(PROJ_DIR)/lab2/csrc" cargo build
	LD_LIBRARY_PATH="$(PROJ_DIR)/lab2/csrc" $(DEBUG_PATH)/lab2