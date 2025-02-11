WORK_DIR=$${HOME}/work
PROJ_DIR=$(WORK_DIR)/riscv-rust

DEBUG_PATH=$(PROJ_DIR)/target/debug
RELEASE_PATH=$(PROJ_DIR)/target/release

COREMARK_PATH = /home/zfu/rios/apd_from_lab1/apd/tb/test_program/coremark/coremark.riscv
DHRYSTONE_PATH = /home/zfu/rios/apd_from_lab1/apd/tb/test_program/benchmarks/dhrystone.riscv
PHYDUMP_PATH = $(PROJ_DIR)/lab2/phydump.txt
RV64UI_PATH =$(PROJ_DIR)/lab2/rv64ui-v-add
TRACE_PATH = $(PROJ_DIR)/lab2/trace.dump
REQ_PIPE = $(PROJ_DIR)/lab2/rqst_to_memory
RESP_PIPE = $(PROJ_DIR)/lab2/resp_to_cpu
SAMPLEDUMP_PATH=$(PROJ_DIR)/lab2/stream.txt
SAMPLE_PATH=$(PROJ_DIR)/lab2/stream.o

TEST_PATH = /opt/cputest/lab2/pattern2.0

run_sample:
	cargo run --release -- -m $(SAMPLEDUMP_PATH) -i $(SAMPLE_PATH)

run_rv64ui-v-add:
	cargo run --release -- -m $(PHYDUMP_PATH) -i $(RV64UI_PATH)

run_phydump:
	cargo run --release -- -m $(PHYDUMP_PATH) -i $(COREMARK_PATH)

run_coremark:
	cargo run --release --features debug-disassemble -- -m $(PHYDUMP_PATH) -i $(COREMARK_PATH)


run_dhrystone:
	cargo run --release -- -i $(DHRYSTONE_PATH) 


run_add:
	RUSTFLAGS="-L $(PROJ_DIR)/lab2/csrc"  cargo build --release 
	LD_LIBRARY_PATH="$(PROJ_DIR)/lab2/csrc" $(RELEASE_PATH)/lab2 -m $(PHYDUMP_PATH) -i $(ADD_PATH) 


cmodule = libinterface.a

test:
	$(MAKE) -C csrc $(cmodule)
	RUSTFLAGS="-L $(PROJ_DIR)/lab2/csrc" cargo build --release --features memdump,dramsim,debug-disassemble
	LD_LIBRARY_PATH="$(PROJ_DIR)/lab2/csrc" $(RELEASE_PATH)/lab2 -m $(SAMPLEDUMP_PATH) -i $(SAMPLE_PATH) -t $(REQ_PIPE) -T $(RESP_PIPE)

build-release:
	$(MAKE) -C csrc $(cmodule)
	RUSTFLAGS="-L $(PROJ_DIR)/lab2/csrc -linterface" cargo build --release --features memdump,dramsim

build:
	$(MAKE) -C csrc $(cmodule)
	RUSTFLAGS="-L $(PROJ_DIR)/lab2/csrc -linterface" cargo build --features memdump,dramsim


run:
	$(RELEASE_PATH)/lab2 -m $(TEST_PATH)/$(test).txt -i $(TEST_PATH)/$(test).o -t $(REQ_PIPE) -T $(RESP_PIPE)
