import time
from machine import Pin

from RGB1602 import RGB1602
from callbacks import blink_led

led = Pin(25, Pin.OUT)
SDA = Pin(0)
SDC = Pin(1)

if __name__ == '__main__':

    lcd = RGB1602(SDA, SDC)

    lcd.printout("Hello!")
    time.sleep(1)

    while True:
        lcd.clear()
        lcd.printout("Let's")
        blink_led(led)
        time.sleep(1)
        lcd.clear()
        lcd.printout("Go!")
        time.sleep(1)
