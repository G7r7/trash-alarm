import time


def blink_led(led):
    led(1)
    time.sleep(0.1)
    led(0)
    time.sleep(0.1)
