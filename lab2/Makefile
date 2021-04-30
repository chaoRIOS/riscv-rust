COREMARK_PATH = /home/zfu/rios/apd_from_lab1/apd/tb/test_program/coremark/coremark.riscv
DHRYSTONE_PATH = /home/zfu/rios/apd_from_lab1/apd/tb/test_program/benchmarks/dhrystone.riscv

TRACE_PATH = /home/cwang/work/riscv-rust/lab2/trace.dump


run_coremark:
	cargo run --release -- -i $(COREMARK_PATH) -t $(TRACE_PATH)

run_dhrystone:
	cargo run --release -- -i $(DHRYSTONE_PATH) 