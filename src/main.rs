#![no_std]
#![no_main]

// Device I2C Addresses
const LCD_ADDRESS: u8 = 0x7c >> 1;
const RGB_ADDRESS: u8 = 0xc0 >> 1;

use arrayvec::ArrayString;
use core::{fmt::Write, u8};

use cortex_m::delay::Delay;
use embedded_hal::digital::v2::OutputPin;
// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

// Time handling traits:
use fugit::RateExtU32;
use rp_pico::hal::{
    self,
    gpio::{Output, Pin, PinId, PushPull},
    rtc::{DateTime, RealTimeClock},
};

fn blink_led<T: PinId>(led_pin: &mut Pin<T, Output<PushPull>>, ms: u32, delay: &mut Delay) {
    led_pin.set_high().unwrap();
    delay.delay_ms(ms);
    led_pin.set_low().unwrap();
    delay.delay_ms(ms);
}

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

    // Real Time Clock
    let date_time = DateTime {
        day: 10,
        month: 09,
        year: 2022,
        day_of_week: hal::rtc::DayOfWeek::Sunday,
        hour: 23,
        minute: 55,
        second: 0,
    };

    let real_time_clock =
        RealTimeClock::new(pac.RTC, clocks.rtc_clock, &mut pac.RESETS, date_time).unwrap();

    // Blink the LED at 1 Hz
    loop {
        let time = real_time_clock.now().unwrap();

        let date_string = datetime_to_date_array_string(&time);
        let time_string = datetime_to_time_array_string(&time);

        lcd.set_cursor_position(0, 0).unwrap();
        lcd.write_str(&date_string.as_str()).unwrap();
        lcd.set_cursor_position(0, 1).unwrap();
        lcd.write_str(time_string.as_str()).unwrap();
        delay.delay_ms(1000);
    }
}

fn datetime_to_date_array_string(time: &DateTime) -> ArrayString<10> {
    let mut time_string = ArrayString::<10>::new();
    write!(
        &mut time_string,
        "{:0>4}/{:0>2}/{:0>2}",
        time.year, time.month, time.day
    )
    .unwrap();
    return time_string;
}

fn datetime_to_time_array_string(time: &DateTime) -> ArrayString<8> {
    let mut time_string = ArrayString::<8>::new();
    write!(
        &mut time_string,
        "{:0>2}:{:0>2}:{:0>2}",
        time.hour, time.minute, time.second
    )
    .unwrap();
    return time_string;
}
