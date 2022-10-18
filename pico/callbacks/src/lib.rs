#![no_std]

use lcd_1602_i2c::Lcd;
use rp_pico::{
    hal::{
        gpio::{self, bank0::BankPinId, Function, Input, Pin, PinId, PullUp},
        rtc::{DateTime, DayOfWeek},
        I2C,
    },
    pac::I2C0,
};
use arrayvec::ArrayString;
use callback::Callback;
use cortex_m::delay::Delay;

pub struct CallbackWriteText <'a, DP: PinId + BankPinId, CP: PinId + BankPinId>{
    text: ArrayString<16>,
    lcd: &'a mut Lcd<I2C<I2C0, (Pin<DP, Function<gpio::I2C>>, Pin<CP, Function<gpio::I2C>>)>>,
    delay: &'a mut Delay
}

impl<'a, DP: PinId + BankPinId, CP: PinId + BankPinId> CallbackWriteText<'a, DP, CP> {
    pub fn new(text: ArrayString<16>, lcd: &'a mut Lcd<I2C<I2C0, (Pin<DP, Function<gpio::I2C>>, Pin<CP, Function<gpio::I2C>>)>>, delay: &'a mut Delay ) -> CallbackWriteText<'a, DP, CP> {
        Self{ text, lcd, delay }
    }
}

impl <DP: PinId + BankPinId, CP: PinId + BankPinId> CallbackWriteText <'_,DP,CP>{
    pub fn text(&self) -> ArrayString<16> {
        self.text
    }

    pub fn set_text(&mut self, text: ArrayString<16>) {
        self.text = text;
    }
}

impl <DP: PinId + BankPinId, CP: PinId + BankPinId> Callback for CallbackWriteText <'_ ,DP,CP>{
    fn call(&mut self) {
        self.lcd.clear(self.delay).unwrap();
        self.lcd.set_cursor_position(0,0).unwrap();
        self.lcd.write_str(self.text.as_str()).unwrap();
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