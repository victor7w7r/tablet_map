use std::cmp::min;
use std::thread::sleep;

use evdev::{EventType, InputEvent, RelativeAxisCode, SynchronizationCode, uinput::VirtualDevice};
use std::io::Result;

#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub struct GradualMove {
  pub x_direction: i32,
  pub y_direction: i32,
  pub both_move: i32,
  pub move_only_x: i32,
  pub move_only_y: i32,
}

impl GradualMove {
  pub fn calculate(x: i32, y: i32) -> Self {
    let x_direction = x.signum();
    let y_direction = y.signum();

    let move_x = x.abs();
    let move_y = y.abs();

    let both_move = min(move_x, move_y);

    let move_only_x = move_x - both_move;
    let move_only_y = move_y - both_move;

    Self {
      x_direction,
      y_direction,
      both_move,
      move_only_x,
      move_only_y,
    }
  }
}

fn scroll_both(x: i32, y: i32, dev: &mut VirtualDevice) -> Result<()> {
  dev.emit(&[InputEvent::new(
    EventType::RELATIVE.0,
    RelativeAxisCode::REL_WHEEL.0,
    y,
  )])?;
  dev.emit(&[InputEvent::new(
    EventType::RELATIVE.0,
    RelativeAxisCode::REL_HWHEEL.0,
    x,
  )])?;
  Ok(())
}

fn scroll_x(x: i32, dev: &mut VirtualDevice) -> Result<()> {
  dev.emit(&[InputEvent::new(
    EventType::RELATIVE.0,
    RelativeAxisCode::REL_HWHEEL.0,
    x,
  )])?;
  Ok(())
}

fn scroll_y(y: i32, dev: &mut VirtualDevice) -> Result<()> {
  dev.emit(&[InputEvent::new(
    EventType::RELATIVE.0,
    RelativeAxisCode::REL_WHEEL.0,
    y,
  )])?;
  Ok(())
}

pub fn smooth_scroll(x: i32, y: i32, dev: &mut VirtualDevice) -> Result<()> {
  let gradual_move = GradualMove::calculate(x, y);

  for _ in 0..gradual_move.both_move {
    scroll_both(gradual_move.x_direction, gradual_move.y_direction, dev)?;
  }
  for _ in 0..gradual_move.move_only_x {
    scroll_x(gradual_move.x_direction, dev)?;
  }
  for _ in 0..gradual_move.move_only_y {
    scroll_y(gradual_move.y_direction, dev)?;
  }

  dev.emit(&[InputEvent::new(
    EventType::SYNCHRONIZATION.0,
    SynchronizationCode::SYN_REPORT.0,
    0,
  )])?;

  Ok(())
}
