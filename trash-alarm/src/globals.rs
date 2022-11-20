use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout;
use core::cell::RefCell;
use critical_section::Mutex;
use lcd_1602_i2c::Lcd;
use rp_pico::hal::gpio::bank0::{Gpio0, Gpio1, Gpio13, Gpio14, Gpio28};
use rp_pico::hal::gpio::Pin;
use rp_pico::hal::gpio::{FunctionI2C, PullUpInput, PushPullOutput};
use rp_pico::hal::multicore::Stack;
use rp_pico::hal::I2C;
use rp_pico::pac::I2C0;

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
pub type MyLcdDataPin = Pin<Gpio0, FunctionI2C>;
pub type MyLcdClockPin = Pin<Gpio1, FunctionI2C>;
pub type MyLcdPins = (MyLcdDataPin, MyLcdClockPin);
pub type MyLcdI2C = I2C<I2C0, MyLcdPins>;
pub type MyLcd = Lcd<MyLcdI2C>;

pub type LedPin = Pin<Gpio13, PushPullOutput>;
pub type ButtonPin = Pin<Gpio14, PullUpInput>;
pub type PIRPin = Pin<Gpio28, PullUpInput>;

pub type LedAndButton = (LedPin, ButtonPin);
pub type LedAndPIR = (LedPin, PIRPin);

pub static GLOBAL_PINS: Mutex<RefCell<Option<LedAndPIR>>> = Mutex::new(RefCell::new(None));
