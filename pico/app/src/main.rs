#![no_std]
#![no_main]

// Device I2C Addresses
const LCD_ADDRESS: u8 = 0x7c >> 1;
const RGB_ADDRESS: u8 = 0xc0 >> 1;

use core::u8;
use arrayvec::ArrayString;
use embedded_hal::prelude::_embedded_hal_blocking_spi_Write;

use datetime::{FormatToArrayString, FromScreenAndButtons};
// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

// Time handling traits:
use fugit::RateExtU32;
use lcd::RainbowAnimation;
use lcd::WriteCurrentDayAndTime;
use rp_pico::hal::rtc::{DateTime, DayOfWeek, RealTimeClock};
use rp_pico::hal::Timer;
use callbacks::{CallbackWriteText, CallbackDoNothing};
use alarm::{Alarm, Triggerable, WeeklyDate};

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

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let arraystr_description = ArrayString::<16>::from("caca").unwrap();

    // Blink the LED at 1 Hz
    loop {
        // {   let ref_lcd = &mut lcd;
        //     delay.delay_ms(5);
        //     ref_lcd.animate_rainbow(301, &mut timer);
        //     let time = real_time_clock.now().unwrap();
        //     //ref_lcd.write_current_day_and_time(time);
        // }
        {
            let ref_lcd = &mut lcd;
            let callback = CallbackWriteText::new(arraystr_description, ref_lcd, &mut delay);
            //let callback = CallbackDoNothing::new();
            let callback2 = CallbackDoNothing::new();
            let mut alarm = Alarm::new(WeeklyDate::new(DayOfWeek::Monday, 0, 0, 2), arraystr_description, 10, 0, 0, callback2, callback);
            let time = real_time_clock.now().unwrap();
            alarm.trigger(time);
            //ref_lcd.write_str(if alarm.trigger(time) {"1"} else {"0"}).unwrap();
            //if alarm.is_date_in_activation_period(time) {delay.delay_ms(10000)}
        }
        {
            delay.delay_ms(5);
            lcd.clear(&mut delay).unwrap();
        }

    }
}
