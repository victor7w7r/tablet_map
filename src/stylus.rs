use crate::utils::STYLUS;

use evdev::{EventType, InputEvent};
use mouse_keyboard_input::{BTN_LEFT, BTN_RIGHT, ChannelSender, VirtualDevice};
use std::{
  thread::sleep,
  thread::{JoinHandle, spawn},
  time::Duration,
};

fn key_map(ev: &InputEvent, sender: ChannelSender) {
  if ev.event_type() == EventType::KEY && ev.code() == 332 {
    if ev.value() == 0 {
      VirtualDevice::send_press(BTN_RIGHT, &sender.clone()).unwrap();
      sleep(Duration::from_millis(50));
      VirtualDevice::send_release(BTN_RIGHT, &sender.clone()).unwrap();
    }
  } else if ev.event_type() == EventType::KEY && ev.code() == 331 {
    if ev.value() == 0 {
      VirtualDevice::send_press(BTN_LEFT, &sender.clone()).unwrap();
      sleep(Duration::from_millis(50));
      VirtualDevice::send_release(BTN_LEFT, &sender.clone()).unwrap();
    }
  }
}

pub fn stylus_thread(sender: ChannelSender, test: Vec<InputEvent>) -> JoinHandle<()> {
  spawn(move || {
    loop {
      for ev in test.clone() {
        println!("{:?}", ev);
        key_map(&ev, sender.clone());
      }
    }
  })
}
