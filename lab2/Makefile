WORK_DIR=/home/ychen/
PROJ_DIR=$(WORK_DIR)/riscv-rust

DRAMSIM_PATH=$(WORK_DIR)/DRAMSim2

DEBUG_PATH=$(PROJ_DIR)/target/debug
RELEASE_PATH=$(PROJ_DIR)/target/release


run: build
	LD_LIBRARY_PATH=$(PROJ_DIR)/lab2/csrc $(DEBUG_PATH)/lab2

build: $(DRAMSIM_LIB)
	$(MAKE) -C csrc dramsim_interface
	RUSTFLAGS="-L $(PROJ_DIR)/lab2/csrc" cargo build


release: 
	RUSTFLAGS="-L $(DRAMSIM_PATH)" cargo build --release
	LD_LIBRARY_PATH=$(DRAMSIM_PATH) $(RELEASE_PATH)/lab2

test: 
	$(MAKE) -C csrc test
	RUSTFLAGS="-L $(PROJ_DIR)/lab2/csrc" cargo build
	LD_LIBRARY_PATH="$(PROJ_DIR)/lab2/csrc" $(DEBUG_PATH)/lab2
