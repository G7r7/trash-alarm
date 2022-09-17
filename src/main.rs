#![no_std]
#![no_main]

mod datetime;
mod led;

// Device I2C Addresses
const LCD_ADDRESS: u8 = 0x7c >> 1;
const RGB_ADDRESS: u8 = 0xc0 >> 1;

use core::u8;

use datetime::{FormatToArrayString, FromScreenAndButtons};
// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

// Time handling traits:
use fugit::RateExtU32;
use rp_pico::hal::rtc::{DateTime, RealTimeClock};

/// The `#[entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables are initialised.
#[rp_pico::entry]
fn main() -> ! {
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
    let sio = rp_pico::hal::Sio::new(pac.SIO);

    // Set the pins up according to their function on this particular board
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Screen

    // Configure two pins as being I²C, not GPIO
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
    lcd.clear(&mut delay).unwrap();
    lcd.set_rgb(128, 128, 128).unwrap();

    // Ask for datetime
    let mut increment_button = pins.gpio16.into_pull_up_input();
    let mut validate_button = pins.gpio17.into_pull_up_input();
    let date_time = DateTime::from_screen_and_buttons(
        &mut lcd,
        &mut delay,
        &mut increment_button,
        &mut validate_button,
    );

    // Real Time Clock
    let real_time_clock =
        RealTimeClock::new(pac.RTC, clocks.rtc_clock, &mut pac.RESETS, date_time).unwrap();

    // Blink the LED at 1 Hz
    loop {
        let time = real_time_clock.now().unwrap();

        lcd.set_cursor_position(0, 0).unwrap();
        lcd.write_str(time.to_date_arraystring().as_str()).unwrap();
        lcd.set_cursor_position(0, 1).unwrap();
        lcd.write_str(time.to_time_arraystring().as_str()).unwrap();
        delay.delay_ms(1000);
    }
}
