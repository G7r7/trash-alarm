#![no_std]

use cortex_m::delay::Delay;
use embedded_hal::digital::v2::OutputPin;
use rp_pico::hal::gpio::{Output, Pin, PinId, PushPull};

pub trait Blinkable {
    fn blink(&mut self, delay: &mut Delay, ms: u32);
}

impl<T: PinId> Blinkable for Pin<T, Output<PushPull>> {
    fn blink(&mut self, delay: &mut Delay, ms: u32) {
        match self.set_high() {
            Ok(_) => {
                delay.delay_ms(ms);
                match self.set_low() {
                    Ok(_) => {
                        delay.delay_ms(ms);
                    }
                    Err(_) => {}
                };
            }
            Err(_) => {}
        }
    }
}
