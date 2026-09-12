from machine import Pin
import time
led = Pin(2, Pin.OUT)
while True:
    led.toggle()
    time.sleep(0.5)
