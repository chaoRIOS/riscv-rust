RUST_MODEL += target/debug/liblab1.so

rs: $(RUST_MODEL)
	cd ..
$(RUST_MODEL): lab1/src/*.rs
	cd lab1
	cargo build

vcs: $(RUST_MODEL)
	vcs -full64 -sverilog test.sverilog
	./simv -sv_root ./dpi/target/debug -sv_lib libdpi