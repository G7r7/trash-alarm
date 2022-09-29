use rp_pico::hal::gpio::{Function, Pin, PinId,bank0::BankPinId};
use rp_pico::hal::{gpio, I2C};
use rp_pico::hal::rtc::DateTime;
use rp_pico::pac::I2C0;
use crate::FormatToArrayString;

pub trait WriteCurrentDayAndTime {
    fn write_current_day_and_time(&mut self, time: DateTime);
}

impl<DP: PinId + BankPinId, CP: PinId + BankPinId> WriteCurrentDayAndTime for lcd_1602_i2c::Lcd<I2C<I2C0, (Pin<DP, Function<gpio::I2C>>, Pin<CP, Function<gpio::I2C>>)>>{
    fn write_current_day_and_time(&mut self, time: DateTime) {
        self.set_cursor_position(0, 0).unwrap();
        self.write_str(time.to_day_of_week_arraystring().as_str()).unwrap();
        self.set_cursor_position(0, 1).unwrap();
        self.write_str(time.to_time_arraystring(false).as_str()).unwrap();
    }
}