// Copyright 2022-2024, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Raw host I/Os for low-level access to the Stylus runtime.
//!
//! This module is only available when the `hostio` feature flag is enabled, which exposes
//! low-level functions for accessing the VM directly. Most users should instead use the
//! high-level equivalents of [`block`](crate::block), [`contract`](crate::contract),
//!
use cfg_if::cfg_if;

// When the `revive` feature is enabled, provide implementations that call
// pallet-revive-uapi host functions instead of WASM imports.
#[cfg(feature = "revive")]
mod revive_impl {
    use pallet_revive_uapi::{HostFn, HostFnImpl as api, StorageFlags};

    /// Helper: read a U256 ([u8; 32]) from pallet-revive and extract the low u64.
    /// pallet-revive returns values in big-endian format.
    #[inline]
    fn u256_to_u64(bytes: &[u8; 32]) -> u64 {
        u64::from_be_bytes(bytes[24..32].try_into().unwrap())
    }

    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn account_balance(address: *const u8, dest: *mut u8) {
        let addr: &[u8; 20] = &*(address as *const [u8; 20]);
        let mut output = [0u8; 32];
        api::balance_of(addr, &mut output);
        core::ptr::copy_nonoverlapping(output.as_ptr(), dest, 32);
    }

    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn account_code(_address: *const u8, _offset: usize, _size: usize, _dest: *mut u8) -> usize {
        // Not available in pallet-revive
        0
    }

    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn account_code_size(address: *const u8) -> usize {
        let addr: &[u8; 20] = &*(address as *const [u8; 20]);
        api::code_size(addr) as usize
    }

    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn account_codehash(address: *const u8, dest: *mut u8) {
        let addr: &[u8; 20] = &*(address as *const [u8; 20]);
        let mut output = [0u8; 32];
        api::code_hash(addr, &mut output);
        core::ptr::copy_nonoverlapping(output.as_ptr(), dest, 32);
    }

    #[allow(dead_code, clippy::missing_safety_doc)]
    pub unsafe fn storage_load_bytes32(key: *const u8, dest: *mut u8) {
        let key_ref: &[u8; 32] = &*(key as *const [u8; 32]);
        let dest_ref: &mut [u8; 32] = &mut *(dest as *mut [u8; 32]);
        api::get_storage_or_zero(StorageFlags::empty(), key_ref, dest_ref);
    }

    #[allow(dead_code, clippy::missing_safety_doc)]
    pub unsafe fn storage_cache_bytes32(key: *const u8, value: *const u8) {
        let key_ref: &[u8; 32] = &*(key as *const [u8; 32]);
        let value_ref: &[u8; 32] = &*(value as *const [u8; 32]);
        api::set_storage_or_clear(StorageFlags::empty(), key_ref, value_ref);
    }

    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn storage_flush_cache(_clear: bool) {
        // No-op: pallet-revive writes storage immediately
    }

    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn block_basefee(basefee: *mut u8) {
        let dest: &mut [u8; 32] = &mut *(basefee as *mut [u8; 32]);
        api::base_fee(dest);
    }

    pub unsafe fn chainid() -> u64 {
        let mut output = [0u8; 32];
        api::chain_id(&mut output);
        u256_to_u64(&output)
    }

    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn block_coinbase(coinbase: *mut u8) {
        let dest: &mut [u8; 20] = &mut *(coinbase as *mut [u8; 20]);
        api::block_author(dest);
    }

    pub unsafe fn block_gas_limit() -> u64 {
        api::gas_limit()
    }

    pub unsafe fn block_number() -> u64 {
        let mut output = [0u8; 32];
        api::block_number(&mut output);
        u256_to_u64(&output)
    }

    pub unsafe fn block_timestamp() -> u64 {
        let mut output = [0u8; 32];
        api::now(&mut output);
        u256_to_u64(&output)
    }

    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn call_contract(
        contract: *const u8,
        calldata: *const u8,
        calldata_len: usize,
        value: *const u8,
        gas: u64,
        return_data_len: *mut usize,
    ) -> u8 {
        let callee: &[u8; 20] = &*(contract as *const [u8; 20]);
        let input = core::slice::from_raw_parts(calldata, calldata_len);
        let value_ref: &[u8; 32] = &*(value as *const [u8; 32]);

        let result = api::call_evm(
            pallet_revive_uapi::CallFlags::empty(),
            callee,
            gas,
            value_ref,
            input,
            None,
        );

        let ret_data_size = api::return_data_size() as usize;
        *return_data_len = ret_data_size;

        match result {
            Ok(()) => 0,
            Err(_) => 1,
        }
    }

    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn contract_address(address: *mut u8) {
        let dest: &mut [u8; 20] = &mut *(address as *mut [u8; 20]);
        api::address(dest);
    }

    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn create1(
        _code: *const u8,
        _code_len: usize,
        _endowment: *const u8,
        _contract: *mut u8,
        _revert_data_len: *mut usize,
    ) {
        // TODO: implement via api::instantiate
    }

    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn create2(
        _code: *const u8,
        _code_len: usize,
        _endowment: *const u8,
        _salt: *const u8,
        _contract: *mut u8,
        _revert_data_len: *mut usize,
    ) {
        // TODO: implement via api::instantiate
    }

    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn delegate_call_contract(
        contract: *const u8,
        calldata: *const u8,
        calldata_len: usize,
        gas: u64,
        return_data_len: *mut usize,
    ) -> u8 {
        let callee: &[u8; 20] = &*(contract as *const [u8; 20]);
        let input = core::slice::from_raw_parts(calldata, calldata_len);

        let result = api::delegate_call_evm(
            pallet_revive_uapi::CallFlags::empty(),
            callee,
            gas,
            input,
            None,
        );

        let ret_data_size = api::return_data_size() as usize;
        *return_data_len = ret_data_size;

        match result {
            Ok(()) => 0,
            Err(_) => 1,
        }
    }

    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn emit_log(data: *const u8, len: usize, topics: usize) {
        let all_data = core::slice::from_raw_parts(data, len);
        let topic_bytes = topics * 32;
        let topic_data = &all_data[..topic_bytes];
        let event_data = &all_data[topic_bytes..];

        // Convert flat bytes to &[[u8; 32]]
        let topics_slice: &[[u8; 32]] = core::slice::from_raw_parts(
            topic_data.as_ptr() as *const [u8; 32],
            topics,
        );

        api::deposit_event(topics_slice, event_data);
    }

    pub unsafe fn evm_gas_left() -> u64 {
        api::gas_left()
    }

    pub unsafe fn evm_ink_left() -> u64 {
        0 // Stylus-specific, no equivalent in pallet-revive
    }

    #[allow(unused_variables)]
    pub unsafe fn pay_for_memory_grow(_pages: u16) {
        // No-op: not needed in PolkaVM
    }

    pub unsafe fn msg_reentrant() -> bool {
        false // No equivalent in pallet-revive
    }

    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn msg_sender(sender: *mut u8) {
        let dest: &mut [u8; 20] = &mut *(sender as *mut [u8; 20]);
        api::caller(dest);
    }

    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn msg_value(value: *mut u8) {
        let dest: &mut [u8; 32] = &mut *(value as *mut [u8; 32]);
        api::value_transferred(dest);
    }

    #[allow(unused, clippy::missing_safety_doc)]
    pub unsafe fn native_keccak256(bytes: *const u8, len: usize, output: *mut u8) {
        let input = core::slice::from_raw_parts(bytes, len);
        let dest: &mut [u8; 32] = &mut *(output as *mut [u8; 32]);
        api::hash_keccak_256(input, dest);
    }

    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn read_args(dest: *mut u8) {
        let len = api::call_data_size() as usize;
        if len > 0 {
            let buf = core::slice::from_raw_parts_mut(dest, len);
            api::call_data_copy(buf, 0);
        }
    }

    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn read_return_data(dest: *mut u8, offset: usize, size: usize) -> usize {
        let ret_size = api::return_data_size() as usize;
        let available = ret_size.saturating_sub(offset);
        let to_copy = core::cmp::min(size, available);
        if to_copy > 0 {
            let mut buf_vec = alloc::vec![0u8; to_copy];
            let mut buf_slice: &mut [u8] = &mut buf_vec;
            api::return_data_copy(&mut buf_slice, offset as u32);
            let copied = buf_slice.len();
            core::ptr::copy_nonoverlapping(buf_vec.as_ptr(), dest, copied);
            copied
        } else {
            0
        }
    }

    #[allow(unused_variables)]
    pub unsafe fn write_result(_data: *const u8, _len: usize) {
        // No-op: in pallet-revive, return data is passed via return_value() in the entrypoint
    }

    pub unsafe fn return_data_size() -> usize {
        api::return_data_size() as usize
    }

    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn static_call_contract(
        contract: *const u8,
        calldata: *const u8,
        calldata_len: usize,
        gas: u64,
        return_data_len: *mut usize,
    ) -> u8 {
        let callee: &[u8; 20] = &*(contract as *const [u8; 20]);
        let input = core::slice::from_raw_parts(calldata, calldata_len);
        let zero_value = [0u8; 32];

        let result = api::call_evm(
            pallet_revive_uapi::CallFlags::READ_ONLY,
            callee,
            gas,
            &zero_value,
            input,
            None,
        );

        let ret_data_size = api::return_data_size() as usize;
        *return_data_len = ret_data_size;

        match result {
            Ok(()) => 0,
            Err(_) => 1,
        }
    }

    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn tx_gas_price(gas_price: *mut u8) {
        // pallet-revive returns gas_price as u64, we need to write it as U256 (32 bytes BE)
        let price = api::gas_price();
        let dest: &mut [u8; 32] = &mut *(gas_price as *mut [u8; 32]);
        *dest = [0u8; 32];
        dest[24..32].copy_from_slice(&price.to_be_bytes());
    }

    pub unsafe fn tx_ink_price() -> u32 {
        1 // Stylus-specific, no equivalent
    }

    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn tx_origin(origin: *mut u8) {
        let dest: &mut [u8; 20] = &mut *(origin as *mut [u8; 20]);
        api::origin(dest);
    }
}

#[cfg(feature = "revive")]
pub use revive_impl::*;

// Console stubs for revive (no-ops)
#[cfg(feature = "revive")]
mod revive_console {
    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn log_f32(_value: f32) {}
    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn log_f64(_value: f64) {}
    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn log_i32(_value: i32) {}
    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn log_i64(_value: i64) {}
    #[allow(unused_variables, clippy::missing_safety_doc)]
    pub unsafe fn log_txt(_text: *const u8, _len: usize) {}
}

#[cfg(feature = "revive")]
pub use revive_console::*;

// Original vm_hooks macro for non-revive builds
#[cfg(not(feature = "revive"))]
macro_rules! vm_hooks {
    (
        $(#[$block_meta:meta])*             // macros & docstrings to apply to all funcs
        module($link:literal, $stub:ident); // configures the wasm_import_module to link

        // all the function declarations
        $($(#[$meta:meta])* $vis:vis fn $func:ident ($($arg:ident : $arg_type:ty),* ) $(-> $return_type:ty)?);*
    ) => {
        cfg_if! {
            if #[cfg(feature = "export-abi")] {
                // Generate a stub for each function.
                // We use a module for the block macros & docstrings.
                $(#[$block_meta])*
                mod $stub {
                    $(
                        $(#[$meta])*
                        #[allow(unused_variables, clippy::missing_safety_doc)]
                        $vis unsafe fn $func($($arg : $arg_type),*) $(-> $return_type)? {
                            unimplemented!()
                        }
                    )*
                }
                #[allow(unused_imports)]
                pub use $stub::*;
            } else if #[cfg(feature = "stylus-test")] {
                $(#[$block_meta])*
                $(
                    $(#[$meta])*
                    #[allow(unused, unused_variables, clippy::missing_safety_doc)]
                    $vis unsafe fn $func($($arg : $arg_type),*) $(-> $return_type)? {
                        panic!("HostIO functions are not available in stylus-test. Use TestVM functions instead.");
                    }
                )*
            } else {
                // Generate a wasm import for each function.
                $(#[$block_meta])*
                #[link(wasm_import_module = $link)]
                extern "C" {
                    $(
                        $(#[$meta])*
                        $vis fn $func($($arg : $arg_type),*) $(-> $return_type)?;
                    )*
                }
            }
        }
    };
}

#[cfg(not(feature = "revive"))]
vm_hooks! {
    module("vm_hooks", vm_hooks);

    /// Gets the ETH balance in wei of the account at the given address.
    /// The semantics are equivalent to that of the EVM's [`BALANCE`] opcode.
    ///
    /// [`BALANCE`]: https://www.evm.codes/#31
    pub fn account_balance(address: *const u8, dest: *mut u8);

    /// Gets a subset of the code from the account at the given address. The semantics are identical to that
    /// of the EVM's [`EXT_CODE_COPY`] opcode, aside from one small detail: the write to the buffer `dest` will
    /// stop after the last byte is written. This is unlike the EVM, which right pads with zeros in this scenario.
    /// The return value is the number of bytes written, which allows the caller to detect if this has occurred.
    ///
    /// [`EXT_CODE_COPY`]: https://www.evm.codes/#3C
    pub fn account_code(address: *const u8, offset: usize, size: usize, dest: *mut u8) -> usize;

    /// Gets the size of the code in bytes at the given address. The semantics are equivalent
    /// to that of the EVM's [`EXT_CODESIZE`].
    ///
    /// [`EXT_CODESIZE`]: https://www.evm.codes/#3B
    pub fn account_code_size(address: *const u8) -> usize;

    /// Gets the code hash of the account at the given address. The semantics are equivalent
    /// to that of the EVM's [`EXT_CODEHASH`] opcode. Note that the code hash of an account without
    /// code will be the empty hash
    /// `keccak("") = c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470`.
    ///
    /// [`EXT_CODEHASH`]: https://www.evm.codes/#3F
    pub fn account_codehash(address: *const u8, dest: *mut u8);

    /// Reads a 32-byte value from permanent storage. Stylus's storage format is identical to
    /// that of the EVM. This means that, under the hood, this hostio is accessing the 32-byte
    /// value stored in the EVM state trie at offset `key`, which will be `0` when not previously
    /// set. The semantics, then, are equivalent to that of the EVM's [`SLOAD`] opcode.
    ///
    /// Note: the Stylus VM implements storage caching. This means that repeated calls to the same key
    /// will cost less than in the EVM.
    ///
    /// [`SLOAD`]: https://www.evm.codes/#54
    #[allow(dead_code)]
    pub fn storage_load_bytes32(key: *const u8, dest: *mut u8);

    /// Writes a 32-byte value to the permanent storage cache. Stylus's storage format is identical to that
    /// of the EVM. This means that, under the hood, this hostio represents storing a 32-byte value into
    /// the EVM state trie at offset `key`. Refunds are tabulated exactly as in the EVM. The semantics, then,
    /// are equivalent to that of the EVM's [`SSTORE`] opcode.
    ///
    /// Note: because the value is cached, one must call `storage_flush_cache` to persist it.
    ///
    /// [`SSTORE`]: https://www.evm.codes/#55
    #[allow(dead_code)]
    pub fn storage_cache_bytes32(key: *const u8, value: *const u8);

    /// Persists any dirty values in the storage cache to the EVM state trie, dropping the cache entirely if requested.
    /// Analogous to repeated invocations of [`SSTORE`].
    ///
    /// [`SSTORE`]: https://www.evm.codes/#55
    pub fn storage_flush_cache(clear: bool);

    /// Gets the basefee of the current block. The semantics are equivalent to that of the EVM's
    /// [`BASEFEE`] opcode.
    ///
    /// [`BASEFEE`]: https://www.evm.codes/#48
    pub fn block_basefee(basefee: *mut u8);

    /// Gets the unique chain identifier of the Arbitrum chain. The semantics are equivalent to
    /// that of the EVM's [`CHAIN_ID`] opcode.
    ///
    /// [`CHAIN_ID`]: https://www.evm.codes/#46
    pub fn chainid() -> u64;

    /// Gets the coinbase of the current block, which on Arbitrum chains is the L1 batch poster's
    /// address. This differs from Ethereum where the validator including the transaction
    /// determines the coinbase.
    pub fn block_coinbase(coinbase: *mut u8);

    /// Gets the gas limit of the current block. The semantics are equivalent to that of the EVM's
    /// [`GAS_LIMIT`] opcode. Note that as of the time of this writing, `evm.codes` incorrectly
    /// implies that the opcode returns the gas limit of the current transaction.  When in doubt,
    /// consult [`The Ethereum Yellow Paper`].
    ///
    /// [`GAS_LIMIT`]: https://www.evm.codes/#45
    /// [`The Ethereum Yellow Paper`]: https://ethereum.github.io/yellowpaper/paper.pdf
    pub fn block_gas_limit() -> u64;

    /// Gets a bounded estimate of the L1 block number at which the Sequencer sequenced the
    /// transaction. See [`Block Numbers and Time`] for more information on how this value is
    /// determined.
    ///
    /// [`Block Numbers and Time`]: https://developer.arbitrum.io/time
    pub fn block_number() -> u64;

    /// Gets a bounded estimate of the Unix timestamp at which the Sequencer sequenced the
    /// transaction. See [`Block Numbers and Time`] for more information on how this value is
    /// determined.
    ///
    /// [`Block Numbers and Time`]: https://developer.arbitrum.io/time
    pub fn block_timestamp() -> u64;

    /// Calls the contract at the given address with options for passing value and to limit the
    /// amount of gas supplied. The return status indicates whether the call succeeded, and is
    /// nonzero on failure.
    ///
    /// In both cases `return_data_len` will store the length of the result, the bytes of which can
    /// be read via the `read_return_data` hostio. The bytes are not returned directly so that the
    /// programmer can potentially save gas by choosing which subset of the return result they'd
    /// like to copy.
    ///
    /// The semantics are equivalent to that of the EVM's [`CALL`] opcode, including callvalue
    /// stipends and the 63/64 gas rule. This means that supplying the `u64::MAX` gas can be used
    /// to send as much as possible.
    ///
    /// [`CALL`]: https://www.evm.codes/#f1
    pub fn call_contract(
        contract: *const u8,
        calldata: *const u8,
        calldata_len: usize,
        value: *const u8,
        gas: u64,
        return_data_len: *mut usize
    ) -> u8;

    /// Gets the address of the current program. The semantics are equivalent to that of the EVM's
    /// [`ADDRESS`] opcode.
    ///
    /// [`ADDRESS`]: https://www.evm.codes/#30
    pub fn contract_address(address: *mut u8);

    /// Deploys a new contract using the init code provided, which the EVM executes to construct
    /// the code of the newly deployed contract. The init code must be written in EVM bytecode, but
    /// the code it deploys can be that of a Stylus program. The code returned will be treated as
    /// WASM if it begins with the EOF-inspired header `0xEFF000`. Otherwise the code will be
    /// interpreted as that of a traditional EVM-style contract. See [`Deploying Stylus Programs`]
    /// for more information on writing init code.
    ///
    /// On success, this hostio returns the address of the newly created account whose address is
    /// a function of the sender and nonce. On failure the address will be `0`, `return_data_len`
    /// will store the length of the revert data, the bytes of which can be read via the
    /// `read_return_data` hostio. The semantics are equivalent to that of the EVM's [`CREATE`]
    /// opcode, which notably includes the exact address returned.
    ///
    /// [`Deploying Stylus Programs`]: https://docs.arbitrum.io/stylus/quickstart
    /// [`CREATE`]: https://www.evm.codes/#f0
    pub fn create1(
        code: *const u8,
        code_len: usize,
        endowment: *const u8,
        contract: *mut u8,
        revert_data_len: *mut usize
    );

    /// Deploys a new contract using the init code provided, which the EVM executes to construct
    /// the code of the newly deployed contract. The init code must be written in EVM bytecode, but
    /// the code it deploys can be that of a Stylus program. The code returned will be treated as
    /// WASM if it begins with the EOF-inspired header `0xEFF000`. Otherwise the code will be
    /// interpreted as that of a traditional EVM-style contract. See [`Deploying Stylus Programs`]
    /// for more information on writing init code.
    ///
    /// On success, this hostio returns the address of the newly created account whose address is a
    /// function of the sender, salt, and init code. On failure the address will be `0`,
    /// `return_data_len` will store the length of the revert data, the bytes of which can be read
    /// via the `read_return_data` hostio. The semantics are equivalent to that of the EVM's
    /// `[CREATE2`] opcode, which notably includes the exact address returned.
    ///
    /// [`Deploying Stylus Programs`]: https://docs.arbitrum.io/stylus/quickstart
    /// [`CREATE2`]: https://www.evm.codes/#f5
    pub fn create2(
        code: *const u8,
        code_len: usize,
        endowment: *const u8,
        salt: *const u8,
        contract: *mut u8,
        revert_data_len: *mut usize
    );

    /// Delegate calls the contract at the given address, with the option to limit the amount of
    /// gas supplied. The return status indicates whether the call succeeded, and is nonzero on
    /// failure.
    ///
    /// In both cases `return_data_len` will store the length of the result, the bytes of which
    /// can be read via the `read_return_data` hostio. The bytes are not returned directly so that
    /// the programmer can potentially save gas by choosing which subset of the return result
    /// they'd like to copy.
    ///
    /// The semantics are equivalent to that of the EVM's [`DELEGATE_CALL`] opcode, including the
    /// 63/64 gas rule. This means that supplying `u64::MAX` gas can be used to send as much as
    /// possible.
    ///
    /// [`DELEGATE_CALL`]: https://www.evm.codes/#F4
    pub fn delegate_call_contract(
        contract: *const u8,
        calldata: *const u8,
        calldata_len: usize,
        gas: u64,
        return_data_len: *mut usize
    ) -> u8;

    /// Emits an EVM log with the given number of topics and data, the first bytes of which should
    /// be the 32-byte-aligned topic data. The semantics are equivalent to that of the EVM's
    /// [`LOG0`], [`LOG1`], [`LOG2`], [`LOG3`], and [`LOG4`] opcodes based on the number of topics
    /// specified. Requesting more than `4` topics will induce a revert.
    ///
    /// [`LOG0`]: https://www.evm.codes/#a0
    /// [`LOG1`]: https://www.evm.codes/#a1
    /// [`LOG2`]: https://www.evm.codes/#a2
    /// [`LOG3`]: https://www.evm.codes/#a3
    /// [`LOG4`]: https://www.evm.codes/#a4
    pub fn emit_log(data: *const u8, len: usize, topics: usize);

    /// Gets the amount of gas left after paying for the cost of this hostio. The semantics are
    /// equivalent to that of the EVM's [`GAS`] opcode.
    ///
    /// [`GAS`]: https://www.evm.codes/#5a
    pub fn evm_gas_left() -> u64;

    /// Gets the amount of ink remaining after paying for the cost of this hostio. The semantics
    /// are equivalent to that of the EVM's [`GAS`] opcode, except the units are in ink. See
    /// [`Ink and Gas`] for more information on Stylus's compute pricing.
    ///
    /// [`GAS`]: https://www.evm.codes/#5a
    /// [`Ink and Gas`]: https://docs.arbitrum.io/stylus/concepts/gas-metering
    pub fn evm_ink_left() -> u64;

    /// The `entrypoint!` macro handles importing this hostio, which is required if the
    /// program's memory grows. Otherwise compilation through the `ArbWasm` precompile will revert.
    /// Internally the Stylus VM forces calls to this hostio whenever new WASM pages are allocated.
    /// Calls made voluntarily will unproductively consume gas.
    pub fn pay_for_memory_grow(pages: u16);

    /// Whether the current call is reentrant.
    pub fn msg_reentrant() -> bool;

    /// Gets the address of the account that called the program. For normal L2-to-L2 transactions
    /// the semantics are equivalent to that of the EVM's [`CALLER`] opcode, including in cases
    /// arising from [`DELEGATE_CALL`].
    ///
    /// For L1-to-L2 retryable ticket transactions, the top-level sender's address will be aliased.
    /// See [`Retryable Ticket Address Aliasing`] for more information on how this works.
    ///
    /// [`CALLER`]: https://www.evm.codes/#33
    /// [`DELEGATE_CALL`]: https://www.evm.codes/#f4
    /// [`Retryable Ticket Address Aliasing`]: https://developer.arbitrum.io/arbos/l1-to-l2-messaging#address-aliasing
    pub fn msg_sender(sender: *mut u8);

    /// Get the ETH value in wei sent to the program. The semantics are equivalent to that of the
    /// EVM's [`CALLVALUE`] opcode.
    ///
    /// [`CALLVALUE`]: https://www.evm.codes/#34
    pub fn msg_value(value: *mut u8);

    /// Efficiently computes the [`keccak256`] hash of the given preimage.
    /// The semantics are equivalent to that of the EVM's [`SHA3`] opcode.
    ///
    /// [`keccak256`]: https://en.wikipedia.org/wiki/SHA-3
    /// [`SHA3`]: https://www.evm.codes/#20
    #[allow(unused)]
    pub fn native_keccak256(bytes: *const u8, len: usize, output: *mut u8);

    /// Reads the program calldata. The semantics are equivalent to that of the EVM's
    /// [`CALLDATA_COPY`] opcode when requesting the entirety of the current call's calldata.
    ///
    /// [`CALLDATA_COPY`]: https://www.evm.codes/#37
    pub fn read_args(dest: *mut u8);

    /// Copies the bytes of the last EVM call or deployment return result. Does not revert if out of
    /// bounds, but rather copies the overlapping portion. The semantics are otherwise equivalent
    /// to that of the EVM's [`RETURN_DATA_COPY`] opcode.
    ///
    /// Returns the number of bytes written.
    ///
    /// [`RETURN_DATA_COPY`]: https://www.evm.codes/#3e
    pub fn read_return_data(dest: *mut u8, offset: usize, size: usize) -> usize;

    /// Writes the final return data. If not called before the program exists, the return data will
    /// be 0 bytes long. Note that this hostio does not cause the program to exit, which happens
    /// naturally when `user_entrypoint` returns.
    pub fn write_result(data: *const u8, len: usize);

    /// Returns the length of the last EVM call or deployment return result, or `0` if neither have
    /// happened during the program's execution. The semantics are equivalent to that of the EVM's
    /// [`RETURN_DATA_SIZE`] opcode.
    ///
    /// [`RETURN_DATA_SIZE`]: https://www.evm.codes/#3d
    pub fn return_data_size() -> usize;

    /// Static calls the contract at the given address, with the option to limit the amount of gas
    /// supplied. The return status indicates whether the call succeeded, and is nonzero on
    /// failure.
    ///
    /// In both cases `return_data_len` will store the length of the result, the bytes of which can
    /// be read via the `read_return_data` hostio. The bytes are not returned directly so that the
    /// programmer can potentially save gas by choosing which subset of the return result they'd
    /// like to copy.
    ///
    /// The semantics are equivalent to that of the EVM's [`STATIC_CALL`] opcode, including the
    /// 63/64 gas rule. This means that supplying `u64::MAX` gas can be used to send as much as
    /// possible.
    ///
    /// [`STATIC_CALL`]: https://www.evm.codes/#FA
    pub fn static_call_contract(
        contract: *const u8,
        calldata: *const u8,
        calldata_len: usize,
        gas: u64,
        return_data_len: *mut usize
    ) -> u8;

    /// Gets the gas price in wei per gas, which on Arbitrum chains equals the basefee. The
    /// semantics are equivalent to that of the EVM's [`GAS_PRICE`] opcode.
    ///
    /// [`GAS_PRICE`]: https://www.evm.codes/#3A
    pub fn tx_gas_price(gas_price: *mut u8);

    /// Gets the price of ink in evm gas basis points. See [`Ink and Gas`] for more information on
    /// Stylus's compute-pricing model.
    ///
    /// [`Ink and Gas`]: https://docs.arbitrum.io/stylus/concepts/gas-metering
    pub fn tx_ink_price() -> u32;

    /// Gets the top-level sender of the transaction. The semantics are equivalent to that of the
    /// EVM's [`ORIGIN`] opcode.
    ///
    /// [`ORIGIN`]: https://www.evm.codes/#32
    pub fn tx_origin(origin: *mut u8)
}

#[cfg(not(feature = "revive"))]
vm_hooks! {
    #[allow(dead_code)]
    module("console", console);

    /// Prints a 32-bit floating point number to the console. Only available in debug mode with
    /// floating point enabled.
    pub fn log_f32(value: f32);

    /// Prints a 64-bit floating point number to the console. Only available in debug mode with
    /// floating point enabled.
    pub fn log_f64(value: f64);

    /// Prints a 32-bit integer to the console, which can be either signed or unsigned.
    /// Only available in debug mode.
    pub fn log_i32(value: i32);

    /// Prints a 64-bit integer to the console, which can be either signed or unsigned.
    /// Only available in debug mode.
    pub fn log_i64(value: i64);

    /// Prints a UTF-8 encoded string to the console. Only available in debug mode.
    pub fn log_txt(text: *const u8, len: usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_hostio_panics_in_test() {
        // The test environment does not support mocking HostIO functions
        unsafe {
            _ = chainid();
        }
    }
}
