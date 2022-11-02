from machine import Pin
import time
# from callbacks import blink_led

led = Pin(25, Pin.OUT)

def blink_led(led):
    led(1)
    time.sleep(1)
    led(0)
    time.sleep(1)


if __name__ == '__main__':
    while True:
        blink_led(led)
        print("prout")
