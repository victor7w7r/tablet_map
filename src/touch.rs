use evdev::{AbsoluteAxisCode, EventType, InputEvent, KeyCode, KeyEvent, RelativeAxisCode};
use std::io::Result;
use std::thread::sleep;
use std::time::Duration;

pub struct Touch {
  virt_dev: evdev::uinput::VirtualDevice,
  tripletap_active: bool,
  dev_name: &'static str,
  last_process_x: Option<i32>,
  last_process_y: Option<i32>,
  scale_sensitivity: f64,
  current_x: Option<i32>,
  current_y: Option<i32>,
  last_abs_before_tripletap_x: Option<i32>,
  last_abs_before_tripletap_y: Option<i32>,
}

impl Touch {
  pub fn new() -> Result<Self> {
    Ok(Touch {
      virt_dev: crate::utils::setup_virtual_device(
        "virtual_touch",
        &Some(vec![KeyCode::BTN_LEFT, KeyCode::BTN_TOOL_TRIPLETAP]),
        &Some(vec![RelativeAxisCode::REL_X, RelativeAxisCode::REL_Y]),
      )?,
      dev_name: "Wacom Intuos5 touch S Finger",
      tripletap_active: false,
      scale_sensitivity: 0.8,
      last_process_x: None,
      last_process_y: None,
      current_x: None,
      current_y: None,
      last_abs_before_tripletap_x: None,
      last_abs_before_tripletap_y: None,
    })
  }

  fn setup_tripletap(&mut self, ev: &InputEvent) -> Result<()> {
    if EventType::KEY == ev.event_type() && ev.code() == KeyCode::BTN_TOOL_TRIPLETAP.0 {
      if ev.value() == 1 {
        self.tripletap_active = true;
        self.last_process_x = self.last_abs_before_tripletap_x;
        self.last_process_y = self.last_abs_before_tripletap_y;
        self
          .virt_dev
          .emit(&[*KeyEvent::new(KeyCode::BTN_LEFT, 1)])?;
      } else if ev.value() == 0 {
        self.tripletap_active = false;
        self
          .virt_dev
          .emit(&[*KeyEvent::new(KeyCode::BTN_LEFT, 0)])?;
      }
    }
    Ok(())
  }

  fn process_sync(&mut self, ev: &InputEvent) -> Result<()> {
    if EventType::SYNCHRONIZATION == ev.event_type() {
      if let (Some(prev_x), Some(prev_y)) = (&self.last_process_x, &self.last_process_y) {
        let dx = (self.current_x.unwrap_or(0) - *prev_x) as f64 * self.scale_sensitivity;
        let dy = (self.current_y.unwrap_or(0) - *prev_y) as f64 * self.scale_sensitivity;
        if dx != 0.0 {
          self.virt_dev.emit(&[InputEvent::new_now(
            EventType::RELATIVE.0,
            RelativeAxisCode::REL_X.0,
            dx.round() as i32,
          )])?;
        }

        if dy != 0.0 {
          self.virt_dev.emit(&[InputEvent::new_now(
            EventType::RELATIVE.0,
            RelativeAxisCode::REL_Y.0,
            dy.round() as i32,
          )])?;
        }

        self.virt_dev.emit(&[InputEvent::new(
          EventType::SYNCHRONIZATION.0,
          evdev::SynchronizationCode::SYN_REPORT.0,
          0,
        )])?;
      }

      self.last_process_x = self.current_x;
      self.last_process_y = self.current_y;
    }
    Ok(())
  }

  fn process_absolute(&mut self, ev: &InputEvent) {
    if EventType::ABSOLUTE == ev.event_type() {
      let axis_code = AbsoluteAxisCode(ev.code());
      let val = ev.value();

      if axis_code == AbsoluteAxisCode::ABS_X {
        if self.tripletap_active {
          self.current_x = Some(val);
        } else {
          self.last_abs_before_tripletap_x = Some(val);
        }
      } else if axis_code == AbsoluteAxisCode::ABS_Y {
        if self.tripletap_active {
          self.current_y = Some(val);
        } else {
          self.last_abs_before_tripletap_y = Some(val);
        }
      }
    }
  }

  pub fn exec(&mut self) -> Result<()> {
    let mut touch_dev = crate::utils::scan_dev(self.dev_name).expect("Failed to find touch device");
    println!("Linked: {}", self.dev_name);

    loop {
      for ev in touch_dev.fetch_events().unwrap() {
        self.setup_tripletap(&ev)?;
        self.process_absolute(&ev);
        self.process_sync(&ev)?;
      }
      sleep(Duration::from_millis(5));
    }
  }
}
