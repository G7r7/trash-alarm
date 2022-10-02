use lcd_1602_i2c::{Blink, Lcd};
use rp_pico::{
    hal::{
        gpio::{self, bank0::BankPinId, Function, Input, Pin, PinId, PullUp},
        rtc::{DateTime, DayOfWeek},
        I2C,
    },
    pac::I2C0,
};
use arrayvec::ArrayString;

pub trait Callback{
    fn call();
}

pub struct CallbackWriteText <DP: PinId + BankPinId, CP: PinId + BankPinId>{
    text: ArrayString<16>,
    lcd: Lcd<I2C<I2C0, (Pin<DP, Function<gpio::I2C>>, Pin<CP, Function<gpio::I2C>>)>>
}

impl <DP: PinId + BankPinId, CP: PinId + BankPinId> CallbackWriteText <DP,CP>{
    pub fn text(&self) -> ArrayString<16> {
        self.text
    }

    pub fn set_text(&mut self, text: ArrayString<16>) {
        self.text = text;
    }
}

impl <DP: PinId + BankPinId, CP: PinId + BankPinId> Callback for CallbackWriteText <DP,CP>{
    fn call() {
        todo!()
    }
}