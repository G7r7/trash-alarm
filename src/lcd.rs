use crate::FormatToArrayString;
use rp_pico::hal::gpio::{bank0::BankPinId, Function, Pin, PinId};
use rp_pico::hal::rtc::DateTime;
use rp_pico::hal::{gpio, Timer, I2C};
use rp_pico::pac::I2C0;

pub trait WriteCurrentDayAndTime {
    fn write_current_day_and_time(&mut self, time: DateTime);
}

pub trait RainbowAnimation {
    fn animate_rainbow(&mut self, loop_duration_ms: u64, timer: &mut Timer);
}

impl<DP: PinId + BankPinId, CP: PinId + BankPinId> WriteCurrentDayAndTime
    for lcd_1602_i2c::Lcd<I2C<I2C0, (Pin<DP, Function<gpio::I2C>>, Pin<CP, Function<gpio::I2C>>)>>
{
    fn write_current_day_and_time(&mut self, time: DateTime) {
        self.set_cursor_position(0, 0).unwrap();
        self.write_str(time.to_day_of_week_arraystring().as_str())
            .unwrap();
        self.set_cursor_position(0, 1).unwrap();
        self.write_str(time.to_time_arraystring(false).as_str())
            .unwrap();
    }
}

impl<DP: PinId + BankPinId, CP: PinId + BankPinId> RainbowAnimation
    for lcd_1602_i2c::Lcd<I2C<I2C0, (Pin<DP, Function<gpio::I2C>>, Pin<CP, Function<gpio::I2C>>)>>
{
    fn animate_rainbow(&mut self, loop_duration_ms: u64, timer: &mut Timer) {
        let (mut r, mut g, mut b) = (0u8, 0u8, 0u8);
        let current_time_ms = timer.get_counter() / 1000;
        let loop_progress = (current_time_ms % loop_duration_ms) as f32 / (loop_duration_ms as f32);
        if loop_progress < 1. / 6. {
            r = 255;
            g = (255. * (loop_progress * 6. - 0.)) as u8;
        } else if loop_progress < 2. / 6. {
            g = 255;
            r = (255. - (255. * (loop_progress * 6. - 1.))) as u8;
        } else if loop_progress < 3. / 6. {
            g = 255;
            b = (255. * (loop_progress * 6. - 2.)) as u8;
        } else if loop_progress < 4. / 6. {
            b = 255;
            g = (255. - (255. * (loop_progress * 6. - 3.))) as u8;
        } else if loop_progress < 5. / 6. {
            b = 255;
            r = (255. * (loop_progress * 6. - 4.)) as u8;
        } else {
            r = 255;
            b = (255. - (255. * (loop_progress * 6. - 5.))) as u8;
        }
        self.set_rgb(r, g, b).unwrap();
    }
}
