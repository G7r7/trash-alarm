use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout;
use rp_pico::hal::multicore::Stack;

#[global_allocator]
pub static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[alloc_error_handler]
pub fn oom(_: Layout) -> ! {
    loop {}
}

// Device I2C Addresses
pub const LCD_ADDRESS: u8 = 0x7c >> 1;
pub const RGB_ADDRESS: u8 = 0xc0 >> 1;

/// Stack for core 1
///
/// Core 0 gets its stack via the normal route - any memory not used by static
/// values is reserved for stack and initialised by cortex-m-rt.
/// To get the same for Core 1, we would need to compile everything seperately
/// and modify the linker file for both programs, and that's quite annoying.
/// So instead, core1.spawn takes a [usize] which gets used for the stack.
/// NOTE: We use the `Stack` struct here to ensure that it has 32-byte
/// alignment, which allows the stack guard to take up the least amount of
/// usable RAM.
pub static mut CORE1_STACK: Stack<4096> = Stack::new();
