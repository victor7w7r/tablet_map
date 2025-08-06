use crate::utils::PAD;

use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, EventType, InputEvent, KeyCode, KeyEvent, RelativeAxisCode};
use std::io::Result;
use std::thread::{sleep, spawn};
use std::time::Duration;

const SCROLL_THRESHOLD: i32 = 3;

static SCROLL_WHEEL: fn(&InputEvent) -> bool =
  |ev| ev.event_type() == EventType::ABSOLUTE && ev.code() == 8;

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
  let mut pad_dev = crate::utils::scan_dev(PAD).expect("Failed to find pad device");

  let mut virt_dev = VirtualDevice::builder()?
    .name("virtual_pad")
    .with_relative_axes(&AttributeSet::from_iter([
      RelativeAxisCode::REL_Y,
      RelativeAxisCode::REL_X,
      RelativeAxisCode::REL_WHEEL,
    ]))?
    .with_keys(&AttributeSet::<KeyCode>::from_iter([
      KeyCode::BTN_LEFT,
      KeyCode::BTN_MIDDLE,
      KeyCode::BTN_RIGHT,
    ]))?
    .build()?;

  pad_dev.grab().expect("Error grabbing pad device");
  println!("Linked: {}", PAD);

  spawn(move || -> Result<()> {
    let mut last_value = 0;

    loop {
      for ev in pad_dev.fetch_events()? {
        key_map(&ev, &mut virt_dev)?;

        if SCROLL_WHEEL(&ev) {
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

            crate::gradual::smooth_scroll(0, limited_scroll, &mut virt_dev)?;
          }

          last_value = current_value;
        }
      }
    }
  });

  Ok(())
}
