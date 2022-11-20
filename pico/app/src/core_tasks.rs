use cortex_m::delay::Delay;
use embedded_hal::digital::v2::OutputPin;
use rp_pico::hal::gpio::{Output, Pin, PinId, PushPull};

pub fn blink_led<T: PinId>(led_pin: &mut Pin<T, Output<PushPull>>, ms: u32) -> ! {
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
    let mut delay = Delay::new(
        core.SYST,
        rp_pico::hal::Clock::freq(&clocks.system_clock).to_Hz(),
    );

    loop {
        led_pin.set_high().unwrap();
        led_pin.set_high().unwrap();
        delay.delay_ms(ms);
        led_pin.set_low().unwrap();
        delay.delay_ms(ms);
    }
}
