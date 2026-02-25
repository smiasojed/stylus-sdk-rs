#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use alloy_primitives::U256;
use stylus_sdk::prelude::*;
use stylus_sdk::storage::StorageU256;

#[storage]
#[entrypoint]
pub struct Counter {
    count: StorageU256,
}

#[public]
impl Counter {
    pub fn get(&self) -> Result<U256, Vec<u8>> {
        Ok(self.count.get())
    }

    pub fn set_count(&mut self, count: U256) -> Result<(), Vec<u8>> {
        self.count.set(count);
        Ok(())
    }

    pub fn increment(&mut self) -> Result<(), Vec<u8>> {
        let current = self.count.get();
        self.count.set(current + U256::from(1));
        Ok(())
    }
}
