extern crate alloc;

use alloc::rc::Rc;
use arrayvec::ArrayString;
use callback::{Callback, Stopper};
use core::cell::RefCell;
use core::ops::DerefMut;
use cortex_m::delay::Delay;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use lcd_1602_i2c::Lcd;
use rp_pico::hal::gpio::{Output, PushPull};
use rp_pico::{
    hal::{
        gpio::{self, bank0::BankPinId, Function, Input, Pin, PinId, PullUp},
        I2C,
    },
    pac::I2C0,
};

pub struct CallbackWriteText<DP: PinId + BankPinId, CP: PinId + BankPinId> {
    text: ArrayString<16>,
    lcd: Rc<RefCell<Lcd<I2C<I2C0, (Pin<DP, Function<gpio::I2C>>, Pin<CP, Function<gpio::I2C>>)>>>>,
    delay: Rc<RefCell<Delay>>,
    duration_ms: u32,
}

impl<DP: PinId + BankPinId, CP: PinId + BankPinId> CallbackWriteText<DP, CP> {
    pub fn new(
        text: ArrayString<16>,
        lcd: Rc<
            RefCell<Lcd<I2C<I2C0, (Pin<DP, Function<gpio::I2C>>, Pin<CP, Function<gpio::I2C>>)>>>,
        >,
        delay: Rc<RefCell<Delay>>,
        duration_ms: u32,
    ) -> Self {
        Self {
            text,
            lcd,
            delay,
            duration_ms,
        }
    }
}

impl<DP: PinId + BankPinId, CP: PinId + BankPinId> CallbackWriteText<DP, CP> {
    pub fn text(&self) -> ArrayString<16> {
        self.text
    }
    pub fn set_text(&mut self, text: ArrayString<16>) {
        self.text = text;
    }
}

impl<DP: PinId + BankPinId, CP: PinId + BankPinId> Callback for CallbackWriteText<DP, CP> {
    fn call(&mut self) -> bool {
        (*self.lcd)
            .borrow_mut()
            .clear((*self.delay).borrow_mut().deref_mut())
            .unwrap();
        (*self.lcd).borrow_mut().set_cursor_position(0, 0).unwrap();
        (*self.lcd)
            .borrow_mut()
            .write_str(self.text.as_str())
            .unwrap();
        (*self.delay).borrow_mut().delay_ms(self.duration_ms);
        return true;
    }
}

pub struct CallbackDoNothing {}

impl CallbackDoNothing {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callback for CallbackDoNothing {
    fn call(&mut self) -> bool {
        return true; // ⸸ CI JIT Guillaume ⸸ (Amen)
    }
}

pub struct CallbackBuzzer<T: PinId, S: Stopper> {
    buzzer: Rc<RefCell<Pin<T, Output<PushPull>>>>,
    single_buzz_duration_ms: u32,
    delay: Rc<RefCell<Delay>>,
    stopper: S,
}

impl<T: PinId, S: Stopper> CallbackBuzzer<T, S> {
    pub fn new(
        buzzer: Rc<RefCell<Pin<T, Output<PushPull>>>>,
        single_buzz_duration_ms: u32,
        delay: Rc<RefCell<Delay>>,
        stopper: S,
    ) -> Self {
        Self {
            buzzer,
            single_buzz_duration_ms,
            delay,
            stopper,
        }
    }
}

impl<T: PinId, S: Stopper> Callback for CallbackBuzzer<T, S> {
    fn call(&mut self) -> bool {
        (*self.buzzer).borrow_mut().set_high().unwrap();
        (*self.delay)
            .borrow_mut()
            .delay_ms(self.single_buzz_duration_ms);
        (*self.buzzer).borrow_mut().set_low().unwrap();
        (*self.delay).borrow_mut().delay_ms(500);
        if self.stopper.should_stop() {
            return false;
        }

        return true;
    }
}

pub struct StopperButton<IP: PinId> {
    button: Rc<RefCell<Pin<IP, Input<PullUp>>>>,
}

impl<IP: PinId> StopperButton<IP> {
    pub fn new(button: Rc<RefCell<Pin<IP, Input<PullUp>>>>) -> Self {
        Self { button }
    }
}

impl<T: PinId> Stopper for StopperButton<T> {
    fn should_stop(&mut self) -> bool {
        (*self.button).borrow_mut().is_low().unwrap()
    }
}
