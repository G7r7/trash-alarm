#![no_std]

use lcd_1602_i2c::Lcd;
use rp_pico::{
    hal::{
        gpio::{self, bank0::BankPinId, Function, Input, Pin, PinId, PullUp, PushPull, Output},
        rtc::{DateTime, DayOfWeek},
        I2C,
    },
    pac::I2C0,
};
use arrayvec::ArrayString;
use callback::Callback;
use cortex_m::delay::Delay;
use embedded_hal::digital::v2::OutputPin;

pub struct CallbackWriteText <'a, DP: PinId + BankPinId, CP: PinId + BankPinId, T: PinId>{
    text: ArrayString<16>,
    lcd: &'a mut Lcd<I2C<I2C0, (Pin<DP, Function<gpio::I2C>>, Pin<CP, Function<gpio::I2C>>)>>,
    delay: &'a mut Delay,
    buzzer: &'a mut Pin<T, Output<PushPull>>
}

impl<'a, DP: PinId + BankPinId, CP: PinId + BankPinId, T: PinId> CallbackWriteText<'a, DP, CP, T> {
    pub fn new(
        text: ArrayString<16>,
        lcd: &'a mut Lcd<I2C<I2C0, (Pin<DP, Function<gpio::I2C>>, Pin<CP, Function<gpio::I2C>>)>>,
        delay: &'a mut Delay, 
        buzzer: &'a mut Pin<T, Output<PushPull>>) 
            -> CallbackWriteText<'a, DP, CP, T> {
        Self{ text, lcd, delay, buzzer }
    }
}

impl <DP: PinId + BankPinId, CP: PinId + BankPinId, T: PinId> CallbackWriteText <'_,DP,CP, T>{
    pub fn text(&self) -> ArrayString<16> {
        self.text
    }

    pub fn set_text(&mut self, text: ArrayString<16>) {
        self.text = text;
    }
}

impl <DP: PinId + BankPinId, CP: PinId + BankPinId, T: PinId> Callback for CallbackWriteText <'_ ,DP,CP, T>{
    fn call(&mut self) {
        self.lcd.clear(self.delay).unwrap();
        self.lcd.set_rgb(255, 0, 0).unwrap();
        self.lcd.set_cursor_position(0,0).unwrap();
        self.lcd.write_str(self.text.as_str()).unwrap();
        for i in 0..3 {
            self.buzzer.set_high().unwrap();
            self.delay.delay_ms(300);
            self.buzzer.set_low().unwrap();
            self.delay.delay_ms(300);
        }
    }
}

pub struct CallbackDoNothing{}

impl CallbackDoNothing {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callback for CallbackDoNothing{
    fn call(&mut self) {
        // ⸸ CI JIT Guillaume ⸸ (Amen)
    }
}