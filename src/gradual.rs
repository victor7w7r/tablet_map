use std::cmp::min;

use evdev::{EventType, InputEvent, RelativeAxisCode, SynchronizationCode, uinput::VirtualDevice};
use std::io::Result;

pub fn smooth_scroll(x: i32, y: i32, dev: &mut VirtualDevice) -> Result<()> {
  if y != 0 {
    dev.emit(&[InputEvent::new(
      EventType::RELATIVE.0,
      RelativeAxisCode::REL_WHEEL.0,
      y,
    )])?;
  }

  if x != 0 {
    dev.emit(&[InputEvent::new(
      EventType::RELATIVE.0,
      RelativeAxisCode::REL_HWHEEL.0,
      x,
    )])?;
  }

  dev.emit(&[InputEvent::new(
    EventType::SYNCHRONIZATION.0,
    SynchronizationCode::SYN_REPORT.0,
    0,
  )])?;

  Ok(())
}
