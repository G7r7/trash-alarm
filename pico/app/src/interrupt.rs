use embedded_hal::digital::v2::OutputPin;
use rp_pico::hal::gpio::Interrupt::{EdgeHigh, EdgeLow};
use rp_pico::pac::interrupt;

use crate::globals::{LedAndPIR, GLOBAL_PINS};

#[interrupt]
fn IO_IRQ_BANK0() {
    // The `#[interrupt]` attribute covertly converts this to `&'static mut Option<LedAndButton>`
    //static mut LED_AND_BUTTON: Option<LedAndButton> = None;
    static mut LED_AND_PIR: Option<LedAndPIR> = None;

    // This is one-time lazy initialisation. We steal the variables given to us
    // via `GLOBAL_PINS`.
    if LED_AND_PIR.is_none() {
        critical_section::with(|cs| {
            *LED_AND_PIR = GLOBAL_PINS.borrow(cs).take();
        });
    }

    // Need to check if our Option<LedAndButton> contains our pins
    if let Some(gpios) = LED_AND_PIR {
        // borrow led and button by *destructuring* the tuple
        // these will be of type `&mut LedPin` and `&mut ButtonPin`, so we don't have
        // to move them back into the static after we use them
        let (led, pir) = gpios;
        // Check if the interrupt source is from the pushbutton going from high-to-low.
        // Note: this will always be true in this example, as that is the only enabled GPIO interrupt source
        if pir.interrupt_status(EdgeHigh) {
            // toggle can't fail, but the embedded-hal traits always allow for it
            // we can discard the return value by assigning it to an unnamed variable
            let _ = led.set_high();

            // Our interrupt doesn't clear itself.
            // Do that now so we don't immediately jump back to this interrupt handler.
            pir.clear_interrupt(EdgeHigh);
        } else if pir.interrupt_status(EdgeLow) {
            let _ = led.set_low();
            pir.clear_interrupt(EdgeLow);
        }
    }
}
