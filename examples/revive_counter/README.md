# Revive Counter Example

A Stylus SDK counter contract targeting **pallet-revive** (PolkaVM/RISC-V) instead of Arbitrum's WASM runtime.

## Prerequisites

- Rust nightly toolchain (specified in `rust-toolchain.toml`)
- `rust-src` component: `rustup component add rust-src`
- [anvil-polkadot](https://github.com/paritytech/foundry-polkadot) for local testing
- [cast](https://book.getfoundry.sh/cast/) for interacting with contracts

## Install cargo-stylus

From the repository root, install the `cargo-stylus` CLI:

```bash
cargo install --path cargo-stylus
```

## Build

Navigate to this example directory and build:

```bash
cd examples/revive_counter
cargo stylus build --target pvm
```

This will:
1. Compile the contract to a RISC-V ELF using the custom PolkaVM target
2. Link the ELF into a `.polkavm` blob via `polkavm-linker`

Output: `target/riscv64emac-unknown-none-polkavm/release/revive_counter.polkavm`

## Deploy

Start a local node:

```bash
anvil-polkadot
```

Deploy the contract (using the default dev account):

```bash
export RPC_URL=http://127.0.0.1:8545
export PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

cast send \
  --rpc-url $RPC_URL \
  --private-key $PRIVATE_KEY \
  --create \
  0x$(cat target/riscv64emac-unknown-none-polkavm/release/revive_counter.polkavm | xxd -p | tr -d '\n')
```

Note the contract address from the output (`contractAddress` field).

## Interact

Set the contract address:

```bash
export CONTRACT=<deployed_contract_address>
```

### Read the counter

```bash
cast call --rpc-url $RPC_URL $CONTRACT "get()(uint256)"
```

Expected output: `0`

### Set the counter

```bash
cast send --rpc-url $RPC_URL --private-key $PRIVATE_KEY $CONTRACT "setCount(uint256)" 42
```

### Increment the counter

```bash
cast send --rpc-url $RPC_URL --private-key $PRIVATE_KEY $CONTRACT "increment()"
```

### Verify

```bash
cast call --rpc-url $RPC_URL $CONTRACT "get()(uint256)"
```

Expected output: `43`

## Contract Interface

| Method | Selector | Description |
|--------|----------|-------------|
| `get()` | `0x6d4ce63c` | Returns the current counter value |
| `setCount(uint256)` | `0xd14e62b8` | Sets the counter to a specific value |
| `increment()` | `0xd09de08a` | Increments the counter by 1 |

## How It Works

The contract uses the Stylus SDK with the `revive` feature, which swaps the WASM host I/O layer for pallet-revive's UAPI. The `#[entrypoint]` macro generates two PolkaVM exports:

- `deploy()` -- called once during contract instantiation (prepends `CONSTRUCTOR_SELECTOR` and routes through the SDK's router)
- `call()` -- called for every subsequent transaction/query
