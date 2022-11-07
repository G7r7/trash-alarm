import machine  # MicroPython version 1.19.1
import datetime_utils
import time
import RGB1602

led = machine.Pin(25, machine.Pin.OUT)
SDA = machine.Pin(0)
SDC = machine.Pin(1)
increment_button = machine.Pin(16, machine.Pin.IN, machine.Pin.PULL_UP)
validate_button = machine.Pin(17, machine.Pin.IN, machine.Pin.PULL_UP)

if __name__ == '__main__':
    lcd = RGB1602.RGB1602(SDA, SDC)
    current_epoch = datetime_utils.epoch_from_screen_and_button(lcd, increment_button, validate_button)
    time_tuple = time.gmtime(current_epoch)
    clock = machine.RTC()
    clock.datetime(time_tuple)

    while True:
        lcd.clear()
        current_epoch = datetime_utils.rtc_tuple_to_epoch(clock.datetime())
        datetime_utils.write_current_day_and_time(lcd, current_epoch)
        time.sleep(1)
