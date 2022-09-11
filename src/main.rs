#![no_std]
#![no_main]

// Device I2C Addresses
const LCD_ADDRESS: u8 = 0x7c >> 1;
const RGB_ADDRESS: u8 = 0xc0 >> 1;

// Colors
const REG_RED: u8 = 0x04;
const REG_GREEN: u8 = 0x03;
const REG_BLUE: u8 = 0x02;
const REG_MODE1: u8 = 0x00;
const REG_MODE2: u8 = 0x01;
const REG_OUTPUT: u8 = 0x08;
const LCD_CLEARDISPLAY: u8 = 0x01;
const LCD_RETURNHOME: u8 = 0x02;
const LCD_ENTRYMODESET: u8 = 0x04;
const LCD_DISPLAYCONTROL: u8 = 0x08;
const LCD_CURSORSHIFT: u8 = 0x10;
const LCD_FUNCTIONSET: u8 = 0x20;
const LCD_SETCGRAMADDR: u8 = 0x40;
const LCD_SETDDRAMADDR: u8 = 0x80;

// Flags for display entry mode
const LCD_ENTRYRIGHT: u8 = 0x00;
const LCD_ENTRYLEFT: u8 = 0x02;
const LCD_ENTRYSHIFTINCREMENT: u8 = 0x01;
const LCD_ENTRYSHIFTDECREMENT: u8 = 0x00;

// Flags for display on/off control
const LCD_DISPLAYON: u8 = 0x04;
const LCD_DISPLAYOFF: u8 = 0x00;
const LCD_CURSORON: u8 = 0x02;
const LCD_CURSOROFF: u8 = 0x00;
const LCD_BLINKON: u8 = 0x01;
const LCD_BLINKOFF: u8 = 0x00;

// Flags for display/cursor shift
const LCD_DISPLAYMOVE: u8 = 0x08;
const LCD_CURSORMOVE: u8 = 0x00;
const LCD_MOVERIGHT: u8 = 0x04;
const LCD_MOVELEFT: u8 = 0x00;

// Flags for function set
const LCD_8BITMODE: u8 = 0x10;
const LCD_4BITMODE: u8 = 0x00;
const LCD_2LINE: u8 = 0x08;
const LCD_1LINE: u8 = 0x00;
const LCD_5x8DOTS: u8 = 0x00;

use cortex_m::{delay::Delay, prelude::_embedded_hal_blocking_i2c_Write};
use embedded_hal::digital::v2::OutputPin;
// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

// Time handling traits:
use fugit::{ExtU32, RateExtU32};
use rp_pico::{
    hal::gpio::{Output, Pin, PinId, PushPull},
    pac::I2C0,
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

    // Leds
    let mut led = pins.led.into_push_pull_output();
    let mut led2 = pins.gpio5.into_push_pull_output();

    // SCTREEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEN

    // Configure two pins as being I²C, not GPIO
    let sda_pin = pins.gpio0.into_mode::<rp_pico::hal::gpio::FunctionI2C>();
    let scl_pin = pins.gpio1.into_mode::<rp_pico::hal::gpio::FunctionI2C>();

    // Create the I²C driver, using the two pre-configured pins. This will fail
    // at compile time if the pins are in the wrong mode, or if this I²C
    // peripheral isn't available on these pins!
    let mut i2c = rp_pico::hal::I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        400.kHz(),
        &mut pac.RESETS,
        &clocks.peripheral_clock,
    );

    // let mut lcd = lcd_1602_i2c::Lcd::new(i2c, LCD_ADDRESS, RGB_ADDRESS, &mut delay).unwrap();
    // lcd.set_rgb(255, 255, 255).unwrap();
    // lcd.write_str("Hello world!").unwrap();

    pub struct RGB1602 {
        pub col: u8,
        pub row: u8,
        pub show_function: u8,
        pub show_control: u8,
        pub show_mode: u8,
        pub current_line: u8,
        pub num_lines: u8,
    }

    let mut rgb1602 = RGB1602 {
        col: 16,
        row: 2,
        show_function: LCD_4BITMODE | LCD_1LINE | LCD_5x8DOTS,
        show_control: LCD_DISPLAYON | LCD_CURSOROFF | LCD_BLINKOFF, // Turn the display on with no cursor or blinking default
        show_mode: LCD_ENTRYLEFT | LCD_ENTRYSHIFTDECREMENT, // Initialize to default text direction (for romance languages)
        current_line: 0,
        num_lines: 16,
    };

    // begin
    if rgb1602.row > 1 {
        rgb1602.show_function |= LCD_2LINE;
    }
    delay.delay_ms(50);
    //  Send function set command sequence
    i2c.write(
        LCD_ADDRESS,
        &[0x80, LCD_FUNCTIONSET | rgb1602.show_function],
    )
    .unwrap();
    delay.delay_ms(50);
    // second try
    i2c.write(
        LCD_ADDRESS,
        &[0x80, LCD_FUNCTIONSET | rgb1602.show_function],
    )
    .unwrap();
    delay.delay_ms(50);
    // third go
    i2c.write(
        LCD_ADDRESS,
        &[0x80, LCD_FUNCTIONSET | rgb1602.show_function],
    )
    .unwrap();
    // finally, set # lines, font size, etc.
    i2c.write(
        LCD_ADDRESS,
        &[0x80, LCD_FUNCTIONSET | rgb1602.show_function],
    )
    .unwrap();
    // // turn the display on with no cursor or blinking default
    i2c.write(
        LCD_ADDRESS,
        &[0x80, LCD_DISPLAYCONTROL | rgb1602.show_control],
    )
    .unwrap();
    // clear it off
    i2c.write(LCD_ADDRESS, &[0x80, LCD_CLEARDISPLAY]).unwrap();
    delay.delay_ms(2);
    // set the entry mode
    i2c.write(LCD_ADDRESS, &[0x80, LCD_ENTRYMODESET | rgb1602.show_mode])
        .unwrap();
    // backlight init
    i2c.write(RGB_ADDRESS, &[REG_MODE1, 0]).unwrap();
    // set LEDs controllable by both PWM and GRPPWM registers
    i2c.write(RGB_ADDRESS, &[REG_OUTPUT, 0xFF]).unwrap();
    // set MODE2 values
    // 0010 0000 -> 0x20  (DMBLNK to 1, ie blinky mode)
    i2c.write(RGB_ADDRESS, &[REG_MODE2, 0x20]).unwrap();
    // WHITE DISPLAY
    i2c.write(RGB_ADDRESS, &[REG_BLUE, 0]).unwrap();
    i2c.write(RGB_ADDRESS, &[REG_GREEN, 0]).unwrap();
    i2c.write(RGB_ADDRESS, &[REG_RED, 255]).unwrap();

    // LEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEED

    // Blink the LED at 1 Hz
    loop {
        led.set_high().unwrap();
        led2.set_high().unwrap();
        delay.delay_ms(500);
        led.set_low().unwrap();
        led2.set_low().unwrap();
        delay.delay_ms(500);
    }
}
