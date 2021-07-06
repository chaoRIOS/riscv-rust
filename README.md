# riscv-rust

[![Build Status](https://travis-ci.org/takahirox/riscv-rust.svg?branch=master)](https://travis-ci.org/takahirox/riscv-rust)
[![Crate](https://img.shields.io/crates/v/riscv_emu_rust.svg)](https://crates.io/crates/riscv_emu_rust)
[![npm version](https://badge.fury.io/js/riscv_emu_rust_wasm.svg)](https://badge.fury.io/js/riscv_emu_rust_wasm)

riscv-rust is a [RISC-V](https://riscv.org/) processor and peripheral devices emulator project written in Rust and compiled to WebAssembly. You can import RISC-V emulator into your Rust or JavaScript project. Refer to the [Slides](https://docs.google.com/presentation/d/1qeR6KMSaJTR0ZSa2kLxgcBuc_zBo3l-kYbOpq1Wqmi0/edit?usp=sharing) for more detail.

<!-- ## Online Demo

You can run Linux or xv6 on the emulator in your browser. [Online demo is here](https://takahirox.github.io/riscv-rust/wasm/web/index.html)

## Screenshots

![animation](./screenshots/animation.gif)
![debugger](./screenshots/debugger.gif) -->
## Getting the code

```bash
git clone git@github.com:chaoRIOS/riscv-rust.git
cd riscv-rust
git remote add rioslab git@github.com:chaoRIOS/riscv-rust.git
git checkout rioslab/lab1
```

## Setting up VS Code

```bash
## Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

## Install components
rustup install nightly  
cargo +nightly install racer
cargo install --force rustfmt
cargo install --force rls

rustup component add rls-preview
rustup component add rust-analysis
rustup component add rust-src
```

Install VS Code extension `Rust` with configuring

```json
{
"rust-client.channel": "stable",
    "rust-client.rustupPath": "~/.cargo/bin/rustup",
    "editor.formatOnSave": true,
}
```

**Note**: Multiple Rust extensions will be conflicting. Make sure you uninstalled any other Rust extensions.


## Toolchain

```bash
/opt/riscv-toolchain-bin-imac-9.2.0/bin
```

## Executing

```bash
cd lab3
make [target [flags]]
```