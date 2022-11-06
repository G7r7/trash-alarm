import time
from machine import Pin
from RGB1602 import RGB1602

# Days of the week
MONDAY = 0
TUESDAY = 1
WEDNESDAY = 2
THURSDAY = 3
FRIDAY = 4
SATURDAY = 5
SUNDAY = 6

# Button phases
YEAR = 0
MONTH = 1
DAY = 2
HOUR_TENS = 3
HOUR_UNITS = 4
MINUTE_TENS = 5
MINUTE_UNITS = 6
FINISHED = 7


def get_day_of_week_string(day_of_week: int) -> str:
    switcher = {
        MONDAY: "Lundi",
        TUESDAY: "Mardi",
        WEDNESDAY: "Mercredi",
        THURSDAY: "Jeudi",
        FRIDAY: "Vendredi",
        SATURDAY: "Samedi",
        SUNDAY: "Dimanche",
    }
    return switcher.get(day_of_week)


def get_button_phase_string(button_phase: int) -> str:
    switcher = {
        YEAR: "Annee ?",
        MONTH: "Mois ?",
        DAY: "Jour ?",
        HOUR_TENS: "Dizaine Heure ?",
        HOUR_UNITS: "Unite Heure ?",
        MINUTE_TENS: "Dizaine Minute ?",
        MINUTE_UNITS: "Unite Minute ?",
        FINISHED: "Finished ?",
    }
    return switcher.get(button_phase)


def epoch_from_screen_and_button(lcd: RGB1602, increment_button: Pin, validate_button: Pin) -> int:
    time_list = [2022, 11, 6, 0, 0, 0, 0, 0]
    button_phase = YEAR
    while True:
        if increment_button() == 0:
            if button_phase == YEAR:
                time_list[0] += 1

            elif button_phase == MONTH:
                time_list[1] += 1

            elif button_phase == DAY:
                time_list[2] += 1

            elif button_phase == HOUR_TENS:
                time_list[3] = (time_list[3] + 10) % 30

            elif button_phase == HOUR_UNITS:
                time_list[3] = ((time_list[3] + 1) % (4 if (time_list[3] // 10) == 2 else 10)) + time_list[3] // 10 * 10

            elif button_phase == MINUTE_TENS:
                time_list[4] = (time_list[4] + 10) % 60

            elif button_phase == MINUTE_UNITS:
                time_list[4] = (time_list[4] + 1) % 10 + time_list[4] // 10 * 10

            else:
                pass

            while increment_button() == 0:
                pass

        if validate_button() == 0:
            button_phase += 1
            while validate_button() == 0:
                pass

        if button_phase == FINISHED:
            lcd.clear()
            return time.mktime(tuple(time_list))

        render(time.mktime(tuple(time_list)), lcd, button_phase)

        # We wait for the next user input.
        while increment_button() == 1 and validate_button() == 1:
            pass


def render(epoch: int, lcd: RGB1602, button_phase: int):
    str_lcd_phase = get_button_phase_string(button_phase)
    lcd.printout(str_lcd_phase)
    switcher = {
        YEAR: to_date_string(epoch),
        MONTH: to_date_string(epoch),
        DAY: to_date_string(epoch),
        HOUR_TENS: to_time_string(epoch),
        HOUR_UNITS: to_time_string(epoch),
        MINUTE_TENS: to_time_string(epoch),
        MINUTE_UNITS: to_time_string(epoch),
    }

    str_lcd_value = switcher.get(button_phase)
    lcd.clear()

    lcd.setCursor(0, 0)
    lcd.printout(str_lcd_phase)

    lcd.setCursor(0, 1)
    lcd.printout(str_lcd_value)

    if button_phase == YEAR:
        lcd.setCursor(0, 1)
    elif button_phase == MONTH:
        lcd.setCursor(5, 1)
    elif button_phase == DAY:
        lcd.setCursor(8, 1)
    elif button_phase == HOUR_TENS:
        lcd.setCursor(0, 1)
    elif button_phase == HOUR_UNITS:
        lcd.setCursor(1, 1)
    elif button_phase == MINUTE_TENS:
        lcd.setCursor(3, 1)
    elif button_phase == MINUTE_UNITS:
        lcd.setCursor(4, 1)


def zfl(s, width):
    # Pads the provided string with leading 0's to suit the specified 'chrs' length
    # Force # characters, fill with leading 0's
    return '{:0>{w}}'.format(s, w=width)


def to_date_string(epoch: int) -> str:
    time_tuple = time.gmtime(epoch)
    return zfl(str(time_tuple[0]), 4) + "/" + zfl(str(time_tuple[1]), 2) + "/" + zfl(str(time_tuple[2]), 2)


def to_time_string(epoch: int) -> str:
    time_tuple = time.gmtime(epoch)
    return zfl(str(time_tuple[3]), 2) + ":" + zfl(str(time_tuple[4]), 2) + ":" + zfl(str(time_tuple[5]), 2)


def to_day_of_week_string(epoch: int) -> str:
    time_tuple = time.gmtime(epoch)
    return get_day_of_week_string(time_tuple[6])


def write_current_day_and_time(lcd: RGB1602, epoch: int):
    lcd.setCursor(0, 0)
    lcd.printout(to_date_string(epoch))
    lcd.setCursor(0, 1)
    lcd.printout(to_time_string(epoch))


def rtc_tuple_to_epoch(rtc_tuple: tuple, lcd: RGB1602) -> int:
    time_tuple = (rtc_tuple[0], rtc_tuple[1], rtc_tuple[2], rtc_tuple[4], rtc_tuple[5], rtc_tuple[6], 0, 0)
    return time.mktime(time_tuple)
