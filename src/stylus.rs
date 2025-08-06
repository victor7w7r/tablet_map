use evdev::{EventType, InputEvent, KeyCode, KeyEvent};
use std::io::Result;
use std::thread::sleep;
use std::time::Duration;

pub struct Stylus {
  virt_dev: evdev::uinput::VirtualDevice,
  dev_name: &'static str,
}

impl Stylus {
  pub fn new() -> Result<Self> {
    Ok(Stylus {
      virt_dev: crate::utils::setup_virtual_device(
        "virtual_stylus",
        &Some(vec![KeyCode::BTN_LEFT, KeyCode::BTN_RIGHT]),
        &None,
      )?,
      dev_name: "Wacom Intuos5 touch S Pen",
    })
  }

  fn key_map(&mut self, ev: &InputEvent) -> Result<()> {
    if ev.event_type() == EventType::KEY {
      match ev.code() {
        332 => {
          if ev.value() == 1 {
            self
              .virt_dev
              .emit(&[*KeyEvent::new(KeyCode::BTN_RIGHT, 1)])?;
          } else if ev.value() == 0 {
            self
              .virt_dev
              .emit(&[*KeyEvent::new(KeyCode::BTN_RIGHT, 0)])?;
          }
        }
        331 => {
          if ev.value() == 1 {
            self
              .virt_dev
              .emit(&[*KeyEvent::new(KeyCode::BTN_LEFT, 1)])?;
          } else if ev.value() == 0 {
            self
              .virt_dev
              .emit(&[*KeyEvent::new(KeyCode::BTN_LEFT, 0)])?;
          }
        }
        _ => {}
      }
    }

    Ok(())
  }

  pub fn exec(&mut self) -> Result<()> {
    let mut stylus_device =
      crate::utils::scan_dev(self.dev_name).expect("Failed to find stylus device");
    println!("Linked: {}", self.dev_name);

    loop {
      for ev in stylus_device.fetch_events()? {
        self.key_map(&ev)?;
      }
      sleep(Duration::from_millis(1));
    }
  }
}
