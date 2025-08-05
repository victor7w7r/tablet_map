#!/usr/bin/python3

from math import pow
from subprocess import run, Popen
from time import sleep, time
from evdev import InputDevice, list_devices, ecodes
from threading import Thread

DEVICE_PAD = "Wacom Intuos5 touch S Pad"
DEVICE_TOUCH = "Wacom Intuos5 touch S Finger"
DEVICE_STYLUS = "Wacom Intuos5 touch S Pen"

devices = [InputDevice(path) for path in list_devices()]

pad_dev = next((d for d in devices if DEVICE_PAD in d.name), None)
touch_dev = next((d for d in devices if DEVICE_TOUCH in d.name), None)
stylus_dev = next((d for d in devices if DEVICE_STYLUS in d.name), None)

if not pad_dev or not touch_dev or not stylus_dev: exit(1)

def start_tablet():
  run(['xsetwacom', 'set', 'Wacom Intuos5 touch S Finger touch', 'Area', '0', '0', '20000', '20000'])
  run(['xsetwacom', 'set', 'Wacom Intuos5 touch S Finger touch', 'ScrollDistance', '7'])
  run(['xsetwacom', 'set', 'Wacom Intuos5 touch S Pen stylus', 'Button', '2', 'key f13'])
  run(['xsetwacom', 'set', 'Wacom Intuos5 touch S Pen stylus', 'Button', '3', 'key f14'])
  run(['/bin/ydotoold', '--socket-path=/tmp/.ydotool_socket', '--socket-own=1000:1000'])

def pad_actions():

  WHEEL_MAX = 71
  DELTA_THRESHOLD = 5

  last_value = None
  print(f"Ready {pad_dev.path} ({pad_dev.name})")
  pad_dev.grab()

  try:
    for event in pad_dev.read_loop():
      if event.type == ecodes.EV_KEY and event.value == 1:
        if event.code == 257: Popen(["ydotool", "click", "0xC0"])
        elif event.code == 258: Popen(["ydotool", "click", "0xC2"])
        elif event.code == 259: Popen(["ydotool", "click", "0xC1"])
        elif event.code == 260: Popen(["ydotool", "click", "0xC0"])
        elif event.code == 261: Popen(["ydotool", "click", "0xC2"])
        elif event.code == 262: Popen(["ydotool", "click", "0xC1"])
      if event.type == ecodes.EV_ABS and event.code == 8:
        value = event.value

        if last_value is None:
          last_value = value
          continue

        delta = value - last_value
        if abs(delta) > (WHEEL_MAX // 2):
          if delta > 0: delta -= (WHEEL_MAX + 1)
          else: delta += (WHEEL_MAX + 1)

        if abs(delta) > DELTA_THRESHOLD:
          last_value = value
          continue

        if delta == 0: continue
        direction = 1 if delta > 0 else -1

        for _ in range(min(abs(delta), 5)):
          run(['ydotool','mousemove','-w','--','0', str(direction)])
          sleep(0.01)

        last_value = value

  except KeyboardInterrupt: pad_dev.ungrab()

def touch_actions():
  print(f"Ready {touch_dev.path} ({touch_dev.name})")
  tap_start_time = None
  in_tripletap = False
  ignore_doubletap = False
  tripletap_active = False
  tripletap_frame = False
  tripletap_release_time = 0
  last_x = None
  last_y = None

  MAX_TAP_DURATION = 0.3

  for event in touch_dev.read_loop():
    if event.type == ecodes.EV_KEY and event.code == ecodes.BTN_TOOL_DOUBLETAP:
      if event.value == 1:
        if tripletap_frame or in_tripletap or (tripletap_release_time and time() - tripletap_release_time < 0.3):
          ignore_doubletap = True
          tap_start_time = None
        else:
          ignore_doubletap = False
          tap_start_time = time()
      elif event.value == 0 and tap_start_time is not None:
        if time() - tap_start_time <= MAX_TAP_DURATION and not ignore_doubletap:
          Popen(["ydotool", "click", "0xC1"])
        tap_start_time = None
        ignore_doubletap = False
    if event.type == ecodes.EV_KEY and event.code == ecodes.BTN_TOOL_TRIPLETAP:
      if event.value == 1:
        Popen(["ydotool", "click", "0x40"])
        in_tripletap = True
        tripletap_active = True
        tripletap_release_time = 0
        tripletap_frame = True
      elif event.value == 0:
        Popen(["ydotool", "click", "0x80"])
        tripletap_release_time = time()
        tripletap_active = False
        last_x = None
        last_y = None
    elif tripletap_active and event.type == ecodes.EV_ABS:
      if event.code == ecodes.ABS_X:
        x = event.value
        if last_x is not None:
          dx = x - last_x
          if abs(dx) > 0: Popen(["ydotool", "mousemove", "--" ,str(dx), "0"])
        last_x = x
      elif event.code == ecodes.ABS_Y:
        y = event.value
        if last_y is not None:
          dy = y - last_y
          if abs(dy) > 0: Popen(["ydotool", "mousemove", "--", "0", str(dy)])
        last_y = y
    elif event.type == ecodes.SYN_REPORT: tripletap_frame = False
    if in_tripletap and tripletap_release_time and (time() - tripletap_release_time >= 0.3):
      in_tripletap = False
      tripletap_release_time = 0

def stylus_actions():
  stylus_button_pressed = False
  capture_y = None
  last_y = None

  MIN_SCROLL_SPEED = 1
  MAX_SCROLL_SPEED = 20
  MAX_DISTANCE = 4000
  SENSITIVITY_FACTOR = 0.09

  for event in stylus_dev.read_loop():
    if event.type == ecodes.EV_KEY and event.value == 1 and event.code == 332:
      Popen(["ydotool", "click", "0xC1"])

    if event.type == ecodes.EV_KEY and event.code == 331:
      if event.value == 0:
        stylus_button_pressed = False
        capture_y = None
        last_y = None
      elif event.value == 1:
        stylus_button_pressed = True
        capture_y = None
        last_y = None
    if stylus_button_pressed and event.type == ecodes.EV_ABS and event.code == ecodes.ABS_Y:
      y = event.value
      print(event.value)
      if capture_y is None:
        capture_y = y
        last_y = y
        continue

      delta = y - last_y
      last_y = y

      if delta == 0: continue

      direction = -1 if delta > 0 else 1

      distance_from_capture = abs(y - capture_y)
      if distance_from_capture > MAX_DISTANCE:
        distance_from_capture = MAX_DISTANCE

      normalized_distance = min(1.0, distance_from_capture / MAX_DISTANCE)
      effective_factor = 1 - pow(1 - normalized_distance, 1)
      
      scroll_speed = int(MIN_SCROLL_SPEED + (MAX_SCROLL_SPEED - MIN_SCROLL_SPEED) * effective_factor)
      scroll_speed = int(scroll_speed * SENSITIVITY_FACTOR)
      scroll_speed = max(MIN_SCROLL_SPEED, min(scroll_speed, MAX_SCROLL_SPEED))

      for _ in range(scroll_speed):
        run(['ydotool', 'mousemove', '-w', '--', '0', str(direction)])

t1 = Thread(target=pad_actions)
t2 = Thread(target=touch_actions)
t3 = Thread(target=stylus_actions)
t4 = Thread(target=start_tablet)

t1.start()
t2.start()
t3.start()
t4.start()

t1.join()
t2.join()
t3.join()
t4.join()
