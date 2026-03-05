# PVM Support (PolkaVM / pallet-revive)

The Stylus SDK supports compiling contracts to **PolkaVM** (PVM), targeting the RISC-V based `pallet-revive` runtime. This allows the same Rust contracts to run on Polkadot-compatible chains.

## Prerequisites

- [Rust nightly toolchain](https://rustup.rs/) (e.g. `nightly-2026-02-01`)
- Cargo Stylus CLI — install from this repository: `cargo install --path cargo-stylus`
- [anvil-polkadot](https://github.com/nicholasgasior/foundry-polkadot) (for local testing)
- [Foundry](https://getfoundry.sh/) (`cast` for deploying and interacting with contracts)

## Building for PVM

PVM builds require a nightly Rust toolchain. Set the `RUSTUP_TOOLCHAIN` environment variable before building:

```bash
export RUSTUP_TOOLCHAIN=nightly-2026-02-01
cargo stylus build --target pvm
```

This compiles the contract to a `.polkavm` binary in the `target/riscv64emac-unknown-none-polkavm/release/` directory.

WASM builds continue to use the stable toolchain configured in `rust-toolchain.toml` — no changes are needed for existing workflows:

```bash
# WASM build (uses stable toolchain from rust-toolchain.toml)
cargo stylus build

# PVM build (uses nightly from RUSTUP_TOOLCHAIN)
RUSTUP_TOOLCHAIN=nightly-2026-02-01 cargo stylus build --target pvm
```

## Deploying to anvil-polkadot

Start a local node and deploy using `cast`:

```bash
# Start local node
anvil-polkadot &

# Deploy the contract
BYTECODE=$(xxd -p < target/riscv64emac-unknown-none-polkavm/release/your_contract.polkavm | tr -d '\n')
ADDR=$(cast send --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --create "0x${BYTECODE}" --json | jq -r '.contractAddress')

echo "Deployed at: $ADDR"

# Interact with the contract
cast call --rpc-url http://127.0.0.1:8545 "$ADDR" "get()(uint256)"
cast send --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  "$ADDR" "setCount(uint256)" 42
```

## Making contracts PVM-compatible

Contracts written for WASM work on PVM with minimal changes:

### 1. Enable `no_std` for PVM

Add to the top of your `lib.rs`:

```rust
#![cfg_attr(target_env = "polkavm", no_std)]

extern crate alloc;
```

### 2. Set `default-features = false` for alloy crates

In your `Cargo.toml`:

```toml
[dependencies]
alloy-primitives = { version = "1.0.1", default-features = false }
alloy-sol-types = { version = "1.0.1", default-features = false }
stylus-sdk = { version = "0.10.0" }
```

### 3. Import `String` and `ToString` where needed

On `no_std`, `String` is not in the prelude. Add explicit imports in files that use them:

```rust
use alloc::string::String;
use alloc::string::ToString;
```

> **Note:** `Vec`, `vec!`, and `format!` are re-exported by the SDK prelude, so they work without extra imports.

### 4. Add `Stylus.toml`

Each contract directory needs a `Stylus.toml` file:

```toml
[contract]
```

## How it works

The PVM build pipeline:

1. **Compile** — Cargo builds the contract to a RISC-V ELF binary (`riscv64emac-unknown-none-polkavm` target) using `-Z build-std=core,alloc`
2. **Link** — `polkavm-linker` converts the ELF to a `.polkavm` binary with optimizations and stripping
3. **Feature injection** — The build system automatically enables the `stylus-sdk/revive` feature, which activates `pallet-revive-uapi` host functions and `polkavm-derive` for entry point exports
4. **Entry points** — The `#[entrypoint]` macro generates `deploy()` and `call()` functions marked with `#[polkavm_derive::polkavm_export]`

## Examples

All examples in the [`examples/`](examples/) directory are PVM-compatible. To build and test any example:

```bash
cd examples/first_app
RUSTUP_TOOLCHAIN=nightly-2026-02-01 cargo stylus build --target pvm
```

See [`examples/revive_counter`](examples/revive_counter/) for a standalone PVM example with its own README.

A comprehensive test script is available at [`test_revive_on_anvil.sh`](test_revive_on_anvil.sh) that builds, deploys, and tests all examples on anvil-polkadot.
