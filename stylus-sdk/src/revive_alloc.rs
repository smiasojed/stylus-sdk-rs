// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Bump allocator and panic handler for PolkaVM (pallet-revive) contracts.

use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

const HEAP_SIZE: usize = 64 * 1024; // 64 KiB

pub struct BumpAllocator {
    offset: AtomicUsize,
    heap: core::cell::UnsafeCell<[u8; HEAP_SIZE]>,
}

unsafe impl Sync for BumpAllocator {}

impl BumpAllocator {
    pub const fn new() -> Self {
        Self {
            offset: AtomicUsize::new(0),
            heap: core::cell::UnsafeCell::new([0u8; HEAP_SIZE]),
        }
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        let mut current = self.offset.load(Ordering::Relaxed);
        loop {
            let aligned = (current + align - 1) & !(align - 1);
            let Some(next) = aligned.checked_add(size) else {
                return core::ptr::null_mut();
            };
            if next > HEAP_SIZE {
                return core::ptr::null_mut();
            }
            match self.offset.compare_exchange_weak(
                current,
                next,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return unsafe { (self.heap.get() as *mut u8).add(aligned) },
                Err(observed) => current = observed,
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[panic_handler]
#[cfg(not(any(test, feature = "export-abi", feature = "stylus-test")))]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
    unsafe {
        // RISC-V illegal instruction to trap immediately
        core::arch::asm!("unimp");
        core::hint::unreachable_unchecked()
    }
    #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
    loop {}
}
