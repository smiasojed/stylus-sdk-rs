#![no_std]
#![no_main]

extern crate alloc;

use alloy_primitives::{Address, FixedBytes, U256};
use stylus_sdk::deploy::RawDeploy;
use stylus_sdk::prelude::*;

/// Factory contract for deploying contracts via CREATE1 and CREATE2.
///
/// In pallet-revive, the `code` passed to deploy must be:
///   `[code_hash (32 bytes, keccak256 of PolkaVM bytecode)][constructor_data...]`
///
/// For a contract with no constructor args, pass the 32-byte code hash directly.
#[storage]
#[entrypoint]
pub struct Factory {}

#[public]
impl Factory {
    /// Deploy a contract via CREATE1 (nonce-based address).
    /// `code_hash` is the keccak256 hash of the uploaded PolkaVM bytecode.
    pub fn deploy(&mut self, code_hash: FixedBytes<32>) -> Result<Address, Vec<u8>> {
        let code: &[u8] = code_hash.as_slice();
        let addr = unsafe { RawDeploy::new().deploy(self.vm(), code, U256::ZERO)? };
        Ok(addr)
    }

    /// Deploy a contract via CREATE2 (salt-based deterministic address).
    /// `code_hash` is the keccak256 hash of the uploaded PolkaVM bytecode.
    /// `salt` is a 32-byte value that determines the deployed address.
    pub fn deploy2(
        &mut self,
        code_hash: FixedBytes<32>,
        salt: FixedBytes<32>,
    ) -> Result<Address, Vec<u8>> {
        let code: &[u8] = code_hash.as_slice();
        let addr = unsafe { RawDeploy::new().salt(salt).deploy(self.vm(), code, U256::ZERO)? };
        Ok(addr)
    }
}
