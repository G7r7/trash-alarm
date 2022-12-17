#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

pub mod callbacks;
pub mod core_tasks;
pub mod datetime;
pub mod globals;
pub mod lcd;
pub mod led;

extern crate alloc;

use alarm::alarm_manager::AlarmManager;
use alloc::rc::Rc;
use alloc::vec;
use arrayvec::ArrayString;
use callbacks::CallbackBuzzerAndWriteText;
use core::cell::RefCell;
use core::ops::DerefMut;
use core::u8;
use datetime::FromScreenAndButtons;
use embedded_hal::digital::v2::InputPin;
use embedded_hal::digital::v2::OutputPin;
use lcd_1602_i2c::Lcd;

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

// Time handling traits:
use alarm::{Alarm, WeeklyDate};
use callbacks::{CallbackWriteText, StopperButton};
use fugit::RateExtU32;
use lcd::RainbowAnimation;
use lcd::WriteCurrentDayAndTime;
use rp_pico::hal::multicore::Multicore;
use rp_pico::hal::rtc::{DateTime, DayOfWeek, RealTimeClock};
use rp_pico::hal::Timer;

use globals::ALLOCATOR;
use globals::CORE1_STACK;
use globals::LCD_ADDRESS;
use globals::RGB_ADDRESS;

use core_tasks::blink_led;

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
    let mut validate_button = pins.gpio1.into_pull_up_input();
    let mut increment_button = pins.gpio5.into_pull_up_input();
    let mut led = pins.gpio9.into_push_pull_output();
    let motion_sensor = pins.gpio16.into_pull_up_input();
    let buzzer_pin = pins.gpio28.into_push_pull_output();

    let sda_pin = pins.gpio12.into_mode::<rp_pico::hal::gpio::FunctionI2C>();
    let scl_pin = pins.gpio13.into_mode::<rp_pico::hal::gpio::FunctionI2C>();
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
    let mut lcd = Lcd::new(i2c, LCD_ADDRESS, RGB_ADDRESS, &mut delay).unwrap();
    let mut embedded_led = pins.led.into_push_pull_output();

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

    // // Start up the second core to blink the second LED
    // let mut mc = Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
    // let cores = mc.cores();
    // let core1 = &mut cores[1];
    // let _test = core1.spawn(unsafe { &mut CORE1_STACK.mem }, move || {
    //     blink_led(&mut embedded_led, 500);
    // });

    // Smart Pointers ----------------------------------------------------
    let rc_delay = Rc::new(RefCell::new(delay));
    let rc_lcd = Rc::new(RefCell::new(lcd));
    let rc_valid_button = Rc::new(RefCell::new(validate_button));
    let rc_buzzer = Rc::new(RefCell::new(buzzer_pin));

    // Alarms ---------------------------------------------------------------
    let alarm = Alarm::new(
        WeeklyDate::new(DayOfWeek::Sunday, 18, 0, 0), // Green trash
        ArrayString::<16>::from("Poubelle verte !").unwrap(),
        6 * 3600, // 6 hours of uptime
        0,
        0,
        CallbackBuzzerAndWriteText::new(
            ArrayString::<16>::from("Poubelle verte !").unwrap(),
            Rc::clone(&rc_lcd),
            Rc::clone(&rc_delay),
            3 * 1000,
            Rc::clone(&rc_buzzer),
            1 * 1000,
            StopperButton::new(Rc::clone(&rc_valid_button)),
            (0, 255, 0),
        ),
        CallbackWriteText::new(
            ArrayString::<16>::from("Merci <3").unwrap(),
            Rc::clone(&rc_lcd),
            Rc::clone(&rc_delay),
            5000,
        ),
    );

    let alarm2 = Alarm::new(
        WeeklyDate::new(DayOfWeek::Wednesday, 18, 0, 0), // Yellow trash
        ArrayString::<16>::from("Poubelle jaune !").unwrap(),
        6 * 3600, // 6 hours of uptime
        0,
        0,
        CallbackBuzzerAndWriteText::new(
            ArrayString::<16>::from("Poubelle jaune !").unwrap(),
            Rc::clone(&rc_lcd),
            Rc::clone(&rc_delay),
            3 * 1000,
            Rc::clone(&rc_buzzer),
            1 * 1000,
            StopperButton::new(Rc::clone(&rc_valid_button)),
            (255, 255, 0),
        ),
        CallbackWriteText::new(
            ArrayString::<16>::from("Merci <3").unwrap(),
            Rc::clone(&rc_lcd),
            Rc::clone(&rc_delay),
            5000,
        ),
    );

    let mut alarm_manager = AlarmManager::new(vec![alarm, alarm2]);

    loop {
        let now = match real_time_clock.now() {
            Ok(value) => value,
            Err(_err) => {
                continue; // We skip a loop
            }
        };
        (*rc_lcd).borrow_mut().animate_rainbow(10000, &mut timer);
        (*rc_lcd).borrow_mut().write_current_day_and_time(&now);
        alarm_manager.rearm_all(&now);
        // Trigger if movement is detected
        if let Some(true) = motion_sensor.is_high().ok() {
            led.set_high().ok();
            (*rc_delay).borrow_mut().delay_ms(100);
            led.set_low().ok();
            (*rc_delay).borrow_mut().delay_ms(100);
            alarm_manager.trigger_all(&now);
        }
        (*rc_delay).borrow_mut().delay_ms(20);
        // Clear the display
        (*rc_lcd)
            .borrow_mut()
            .clear((*rc_delay).borrow_mut().deref_mut())
            .ok();
    }
}
