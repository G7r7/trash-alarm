#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

extern crate alloc;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[alloc_error_handler]
fn oom(_: Layout) -> ! {
    loop {}
}

// Device I2C Addresses
const LCD_ADDRESS: u8 = 0x7c >> 1;
const RGB_ADDRESS: u8 = 0xc0 >> 1;

use alloc::rc::Rc;
use core::alloc::Layout;
use core::borrow::BorrowMut;
use core::cell::RefCell;
use core::ops::DerefMut;
use core::panic::PanicInfo;
use core::u8;
use alloc_cortex_m::CortexMHeap;
use arrayvec::ArrayString;
use cortex_m::delay::Delay;
use embedded_hal::digital::v2::{InputPin, OutputPin};

use datetime::{FromScreenAndButtons};
// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

// Time handling traits:
use fugit::RateExtU32;
use rp_pico::hal::gpio::{Output, Pin, PinId, PushPull};
use lcd::RainbowAnimation;
use lcd::WriteCurrentDayAndTime;
use rp_pico::hal::rtc::{DateTime, DayOfWeek, RealTimeClock};
use rp_pico::hal::Timer;
use callbacks::{CallbackBuzzer, CallbackWriteText, StopperButton};
use alarm::{Alarm, Triggerable, WeeklyDate};
use rp_pico::hal::multicore::{Multicore, Stack};

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
static mut CORE1_STACK: Stack<4096> = Stack::new();

fn blink_led<T: PinId>(led_pin: &mut Pin<T, Output<PushPull>>, ms: u32) -> ! {
    let mut pac = unsafe { rp_pico::pac::Peripherals::steal() };
    let core = unsafe { rp_pico::pac::CorePeripherals::steal() };

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = rp_pico::hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    // The default is to generate a 125 MHz system clock
    let clocks = rp_pico::hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
        .ok()
        .unwrap();

    // Set up the delay for the second core.
    let mut delay = Delay::new(core.SYST, rp_pico::hal::Clock::freq(&clocks.system_clock).to_Hz());

    loop {
        led_pin.set_high().unwrap();
        led_pin.set_high().unwrap();
        delay.delay_ms(ms);
        led_pin.set_low().unwrap();
        delay.delay_ms(ms);
    }
}

/// The `#[entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables are initialised.
#[rp_pico::entry]
fn main() -> ! {
    // Initialize the allocator BEFORE you use it
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 1024;
        static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE) }
    }

    // Grab our singleton objects
    let mut pac = rp_pico::pac::Peripherals::take().unwrap();
    let core = rp_pico::pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = rp_pico::hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    // The default is to generate a 125 MHz system clock
    let clocks = rp_pico::hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();
    // The delay object lets us wait for specified amounts of time (in
    // milliseconds)
    let mut delay = cortex_m::delay::Delay::new(
        core.SYST,
        rp_pico::hal::Clock::freq(&clocks.system_clock).to_Hz(),
    );

    // The single-cycle I/O block controls our GPIO pins
    // let mut sio = rp_pico::hal::Sio::new(pac.SIO);
    let mut sio = rp_pico::hal::Sio::new(pac.SIO);

    // Set the pins up according to their function on this particular board
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );


    // Pins -------------------------------------------------------------------------------------------------------
    let mut increment_button = pins.gpio16.into_pull_up_input();
    let mut validate_button = pins.gpio17.into_pull_up_input();
    let mut motion_sensor = pins.gpio14.into_pull_up_input();
    let sda_pin = pins.gpio0.into_mode::<rp_pico::hal::gpio::FunctionI2C>();
    let scl_pin = pins.gpio1.into_mode::<rp_pico::hal::gpio::FunctionI2C>();
    // Create the I²C driver, using the two pre-configured pins. This will fail
    // at compile time if the pins are in the wrong mode, or if this I²C
    // peripheral isn't available on these pins!
    let i2c = rp_pico::hal::I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        400.kHz(),
        &mut pac.RESETS,
        &clocks.peripheral_clock,
    );
    let mut lcd = lcd_1602_i2c::Lcd::new(i2c, LCD_ADDRESS, RGB_ADDRESS, &mut delay).unwrap();
    let buzzer_pin = pins.gpio15.into_push_pull_output();
    let mut led_pin = pins.led.into_push_pull_output();


    // Ask for datetime ---------------------------------------------------------------------------------
    lcd.clear(&mut delay).unwrap();
    lcd.set_rgb(128, 128, 128).unwrap();
    let date_time = DateTime::from_screen_and_buttons(
        &mut lcd,
        &mut delay,
        &mut increment_button,
        &mut validate_button,
    );

    // Real Time Clock
    let real_time_clock =
        RealTimeClock::new(pac.RTC, clocks.rtc_clock, &mut pac.RESETS, date_time).unwrap();

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let arraystr_description = ArrayString::<16>::from("caca").unwrap();


    // Start up the second core to blink the second LED
    let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
    let cores = mc.cores();
    let core1 = &mut cores[1];
    let _test = core1
        .spawn(unsafe { &mut CORE1_STACK.mem }, move || {
            blink_led(&mut led_pin, 500);
        });

    // Smart Pointers ----------------------------------------------------
    let mut rc_delay = Rc::new(RefCell::new(delay));
    let mut rc_lcd = Rc::new(RefCell::new(lcd));

    // Callbacks ---------------------------------------------------------
    let stopper = StopperButton::new(validate_button);
    let callback = CallbackBuzzer::new(
        buzzer_pin,1000,
        Rc::clone(&rc_delay), stopper
    );

    let deactivation_callback = CallbackWriteText::new(
        ArrayString::<16>::from("ALARME STOPPEE").unwrap(), Rc::clone(&rc_lcd), Rc::clone(&rc_delay), 3000
    );

    // Alarms ---------------------------------------------------------------
    let mut alarm = Alarm::new(
        WeeklyDate::new(DayOfWeek::Monday, 0, 0, 5),
        arraystr_description, 60, 0, 0, callback, deactivation_callback
    );


    loop {
        (*rc_lcd).borrow_mut().animate_rainbow(10000, &mut timer);
        (*rc_lcd).borrow_mut().write_current_day_and_time(real_time_clock.now().unwrap());
        if motion_sensor.is_low().unwrap() {
            alarm.trigger(real_time_clock.now().unwrap());
        }
        (*rc_delay).borrow_mut().delay_ms(20);
        (*rc_lcd).borrow_mut().clear((*rc_delay).borrow_mut().deref_mut()).unwrap();
    }
}

