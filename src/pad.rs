use evdev::uinput::VirtualDevice;
use evdev::{EventType, InputEvent, KeyCode, KeyEvent, RelativeAxisCode, SynchronizationCode};
use std::io::Result;
use std::thread::sleep;
use std::time::Duration;

use crate::utils::setup_virtual_device;

const SCROLL_THRESHOLD: i32 = 3;

const DEV_NAME: &str = "Wacom Intuos5 touch S Pad";

fn key_map(ev: &InputEvent, dev: &mut VirtualDevice) -> Result<()> {
  if ev.event_type() == EventType::KEY && ev.value() == 1 {
    if ev.code() == 257 || ev.code() == 260 {
      dev.emit(&[*KeyEvent::new(KeyCode::BTN_LEFT, 1)])?;
      sleep(Duration::from_millis(50));
      dev.emit(&[*KeyEvent::new(KeyCode::BTN_LEFT, 0)])?;
    } else if ev.code() == 258 || ev.code() == 261 {
      dev.emit(&[*KeyEvent::new(KeyCode::BTN_MIDDLE, 1)])?;
      sleep(Duration::from_millis(50));
      dev.emit(&[*KeyEvent::new(KeyCode::BTN_MIDDLE, 0)])?;
    } else if ev.code() == 259 || ev.code() == 262 {
      dev.emit(&[*KeyEvent::new(KeyCode::BTN_RIGHT, 1)])?;
      sleep(Duration::from_millis(50));
      dev.emit(&[*KeyEvent::new(KeyCode::BTN_RIGHT, 0)])?;
    }
  }
  Ok(())
}

pub fn pad_thread() -> Result<()> {
  let mut pad_dev = crate::utils::scan_dev(DEV_NAME).expect("Failed to find pad device");

  let mut virt_dev = setup_virtual_device(
    DEV_NAME,
    &Some(vec![
      KeyCode::BTN_LEFT,
      KeyCode::BTN_MIDDLE,
      KeyCode::BTN_RIGHT,
    ]),
    &Some(vec![
      RelativeAxisCode::REL_Y,
      RelativeAxisCode::REL_X,
      RelativeAxisCode::REL_WHEEL,
    ]),
  )?;

  pad_dev.grab().expect("Error grabbing pad device");
  println!("Linked: {}", DEV_NAME);

  let mut last_value = 0;

  loop {
    for ev in pad_dev.fetch_events()? {
      key_map(&ev, &mut virt_dev)?;

      if ev.event_type() == EventType::ABSOLUTE && ev.code() == 8 {
        let current_value = ev.value();
        let mut delta = current_value - last_value;

        if delta.abs() > 36 {
          if delta > 0 {
            delta -= 72; // 71 + 1
          } else {
            delta += 72;
          }
        }

        if delta != 0 && delta.abs() >= SCROLL_THRESHOLD {
          let scroll_amount = delta / SCROLL_THRESHOLD;
          let limited_scroll = if scroll_amount.abs() > 3 {
            if scroll_amount > 0 { 3 } else { -3 }
          } else {
            scroll_amount
          };

          virt_dev.emit(&[InputEvent::new(
            EventType::RELATIVE.0,
            RelativeAxisCode::REL_WHEEL.0,
            limited_scroll,
          )])?;

          virt_dev.emit(&[InputEvent::new(
            EventType::SYNCHRONIZATION.0,
            SynchronizationCode::SYN_REPORT.0,
            0,
          )])?;
        }

        last_value = current_value;
      }
    }
  }
}
