// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(feature = "contract-client-gen", allow(unused_imports))]
#![cfg_attr(target_env = "polkavm", no_std)]

extern crate alloc;

use stylus_sdk::{console, prelude::*, stylus_proc::entrypoint, ArbResult};

#[storage]
#[entrypoint]
pub struct Hello;

#[public]
impl Hello {
    fn hello() -> ArbResult {
        console!("Hello Stylus!");
        Ok(Vec::new())
    }
}
