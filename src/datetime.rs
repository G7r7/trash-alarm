use arrayvec::ArrayString;
use core::fmt::{Error, Write};
use cortex_m::delay::Delay;
// use defmt::write;
use embedded_hal::{blocking::serial::write, digital::v2::InputPin};
use lcd_1602_i2c::{Blink, Lcd};
use rp_pico::{
    hal::{
        self,
        gpio::{self, bank0::BankPinId, Function, Input, Pin, PinId, PullUp},
        rtc::{DateTime, DayOfWeek},
        I2C,
    },
    pac::I2C0,
};


pub trait FormatToArrayString {
    fn to_date_arraystring(&self) -> ArrayString<10>;
    fn to_time_arraystring(&self) -> ArrayString<8>;
}

pub trait FromScreenAndButtons {
    fn from_screen_and_buttons<DP: PinId + BankPinId, CP: PinId + BankPinId, IP: PinId, VP: PinId>(
        lcd: &mut Lcd<I2C<I2C0, (Pin<DP, Function<gpio::I2C>>, Pin<CP, Function<gpio::I2C>>)>>,
        delay: &mut Delay,
        increment_button: &mut Pin<IP, Input<PullUp>>,
        validate_button: &mut Pin<VP, Input<PullUp>>,
    ) -> Self;
}
fn get_time_of_day_string(day_of_week: DayOfWeek) -> ArrayString<8> {
    let ret = match day_of_week {
        DayOfWeek::Monday => "Lundi",
        DayOfWeek::Tuesday => "Mardi",
        DayOfWeek::Wednesday => "Mercredi",
        DayOfWeek::Thursday => "Jeudi",
        DayOfWeek::Friday => "Vendredi",
        DayOfWeek::Saturday => "Samedi",
        DayOfWeek::Sunday => "Dimanche",
    };

    let mut ret_arrstr = ArrayString::<8>::new();
    write!(ret_arrstr, "{: <8}", ret).unwrap();
    // write!(ret_arrstr, "{}", ret).unwrap();
    return ret_arrstr;
}

#[derive(PartialEq)]
enum ButtonPhase {
    ButtonPhaseDayOfWeek = 0,
    ButtonPhaseTimeHourTens = 1,
    ButtonPhaseTimeHourUnits = 2,
    ButtonPhaseTimeMinuteTens = 3,
    ButtonPhaseTimeMinuteUnits = 4,
    ButtonPhaseFinished = 5,
}

fn get_button_phase_from_u8(button_phase:&u8) -> ButtonPhase {
    return match button_phase {
        0 => ButtonPhase::ButtonPhaseDayOfWeek,
        1 => ButtonPhase::ButtonPhaseTimeHourTens,
        2 => ButtonPhase::ButtonPhaseTimeHourUnits,
        3 => ButtonPhase::ButtonPhaseTimeMinuteTens,
        4 => ButtonPhase::ButtonPhaseTimeMinuteUnits,
        5 => ButtonPhase::ButtonPhaseFinished,
        _ => ButtonPhase::ButtonPhaseFinished,
    }
}
fn get_button_phase_string(button_phase: &ButtonPhase) -> ArrayString<16>{
    let ret = match button_phase {
        ButtonPhase::ButtonPhaseDayOfWeek => "Jour Semaine ?",
        ButtonPhase:: ButtonPhaseTimeHourTens => "Dizaine Heure ?",
        ButtonPhase:: ButtonPhaseTimeHourUnits => "Unite Heure ?",
        ButtonPhase:: ButtonPhaseTimeMinuteTens => "Dizaine Minute ?",
        ButtonPhase:: ButtonPhaseTimeMinuteUnits => "Unite Minute ?",
        ButtonPhase:: ButtonPhaseFinished => "Finished ?",
        _ => "WTF ?",
    };

    let mut ret_arrstr = ArrayString::<16>::new();
    write!(ret_arrstr, "{: <16}", ret).unwrap();
    // write!(ret_arrstr, "{}", ret).unwrap();
    return ret_arrstr;
}

fn day_of_week_from_u8(v: u8) -> DayOfWeek {
    match v {
        0 => DayOfWeek::Sunday,
        1 => DayOfWeek::Monday,
        2 => DayOfWeek::Tuesday,
        3 => DayOfWeek::Wednesday,
        4 => DayOfWeek::Thursday,
        5 => DayOfWeek::Friday,
        6 => DayOfWeek::Saturday,
        _ => DayOfWeek::Sunday,
    }
}

impl FromScreenAndButtons for DateTime {
    fn from_screen_and_buttons<
        DP: PinId + BankPinId,
        CP: PinId + BankPinId,
        IP: PinId,
        VP: PinId,
    >(
        lcd: &mut Lcd<I2C<I2C0, (Pin<DP, Function<gpio::I2C>>, Pin<CP, Function<gpio::I2C>>)>>,
        delay: &mut Delay,
        increment_button: &mut Pin<IP, Input<PullUp>>,
        validate_button: &mut Pin<VP, Input<PullUp>>,
    ) -> Self {
        // Année
        let mut datetime = DateTime {
            year: 0,
            day: 1,
            month: 1,
            day_of_week: DayOfWeek::Monday,
            hour: 0,
            minute: 0,
            second: 0,
        };
        let mut day_of_week = DayOfWeek::Monday;
        lcd.clear(delay).unwrap();

        let mut button_phase = ButtonPhase::ButtonPhaseDayOfWeek;

        loop {
            if validate_button.is_low().unwrap() {
                let mut incr_button_phase= button_phase as u8 + 1;
                button_phase = get_button_phase_from_u8(&incr_button_phase);
                lcd.clear(delay).unwrap();
                while validate_button.is_low().unwrap() {}

            }

            lcd.set_cursor_position(0, 0).unwrap();
            lcd.write_str(get_button_phase_string(&button_phase).as_str()).unwrap();

            if button_phase == ButtonPhase::ButtonPhaseDayOfWeek {
                // lcd.set_cursor_position(0, 1).unwrap();
                // lcd.write_str("        ").unwrap();
                lcd.set_cursor_position(0, 1).unwrap();
                lcd.write_str(get_time_of_day_string(day_of_week).as_str())
                    .unwrap();
                if increment_button.is_low().unwrap() {
                    day_of_week = day_of_week_from_u8((day_of_week as u8 + 1) % 7);
                    while increment_button.is_low().unwrap() {}
                }
            } else {
                lcd.set_cursor_position(0, 1).unwrap();
                lcd.write_str(&datetime.to_time_arraystring()[0..5]).unwrap();
            }
            match button_phase {
                ButtonPhase::ButtonPhaseTimeHourUnits => blink_digit(lcd, 1),
                ButtonPhase::ButtonPhaseTimeHourTens => blink_digit(lcd, 0),
                ButtonPhase::ButtonPhaseTimeMinuteTens => blink_digit(lcd, 3),
                ButtonPhase::ButtonPhaseTimeMinuteUnits => blink_digit(lcd, 4),
                ButtonPhase::ButtonPhaseFinished => continue,
                ButtonPhase::ButtonPhaseDayOfWeek => continue,
            }
            // We wait for the next user input.
            while !increment_button.is_low().unwrap() && !validate_button.is_low().unwrap() {}
        }
        return datetime;
    }
}

fn blink_digit<DP: PinId + BankPinId, CP: PinId + BankPinId>(
    lcd: &mut Lcd<I2C<I2C0, (Pin<DP, Function<gpio::I2C>>, Pin<CP, Function<gpio::I2C>>)>>,
    pos_digit: u8
) {
    lcd.set_cursor_position(pos_digit, 1).unwrap();
    lcd.set_blink(Blink::On).unwrap();
}

impl FormatToArrayString for DateTime {
    fn to_date_arraystring(&self) -> ArrayString<10> {
        let mut date_string = ArrayString::<10>::new();
        write!(
            &mut date_string,
            "{:0>4}/{:0>2}/{:0>2}",
            self.year, self.month, self.day
        )
        .unwrap();
        return date_string;
    }

    fn to_time_arraystring(&self) -> ArrayString<8> {
        let mut time_string = ArrayString::<8>::new();
        write!(
            &mut time_string,
            "{:0>2}:{:0>2}:{:0>2}",
            self.hour, self.minute, self.second
        )
        .unwrap();
        return time_string;
    }
}
