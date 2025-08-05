use crate::utils::TOUCH;

use evdev::{AbsoluteAxisCode, EventType, InputEvent};
use mouse_keyboard_input::{BTN_LEFT, BTN_TOOL_TRIPLETAP, VirtualDevice};
use std::thread::{JoinHandle, spawn};

fn tripletap(
  ev: &InputEvent,
  tripletap_active: &mut bool,
  last_process_x: &mut Option<i32>,
  last_process_y: &mut Option<i32>,
  last_abs_before_tripletap_x: &Option<i32>,
  last_abs_before_tripletap_y: &Option<i32>,
) {
  if EventType::KEY == ev.event_type() && ev.code() == BTN_TOOL_TRIPLETAP {
    if ev.value() == 1 {
      println!("Tripletap");
      *tripletap_active = true;

      *last_process_x = *last_abs_before_tripletap_x;
      *last_process_y = *last_abs_before_tripletap_y;
      //virtual_dev.press(BTN_LEFT).unwrap();
    } else if ev.value() == 0 {
      println!("Tripletap release");
      *tripletap_active = false;
      //virtual_dev.release(BTN_LEFT).unwrap();
    }
  }
}

fn absolute_moves(
  ev: &InputEvent,
  tripletap_active: bool,
  current_x: &mut Option<i32>,
  current_y: &mut Option<i32>,
  last_abs_before_tripletap_x: &mut Option<i32>,
  last_abs_before_tripletap_y: &mut Option<i32>,
) {
  if EventType::ABSOLUTE == ev.event_type() {
    let axis_code = AbsoluteAxisCode(ev.code());
    let val = ev.value();

    if axis_code == AbsoluteAxisCode::ABS_X {
      if tripletap_active {
        *current_x = Some(val);
      } else {
        *last_abs_before_tripletap_x = Some(val);
      }
    } else if axis_code == AbsoluteAxisCode::ABS_Y {
      if tripletap_active {
        *current_y = Some(val);
      } else {
        *last_abs_before_tripletap_y = Some(val);
      }
    }
  }
}

fn synchronization(
  ev: &InputEvent,
  dev: &mut VirtualDevice,
  current_x: &mut Option<i32>,
  current_y: &mut Option<i32>,
  scale_sensitivity: f64,
  last_process_x: &mut Option<i32>,
  last_process_y: &mut Option<i32>,
) {
  if EventType::SYNCHRONIZATION == ev.event_type() {
    let x = current_x.unwrap_or(0);
    let y = current_y.unwrap_or(0);

    if let (Some(prev_x), Some(prev_y)) = (&*last_process_x, &*last_process_y) {
      let dx = (x - *prev_x) as f64 * scale_sensitivity;
      let dy = (y - *prev_y) as f64 * scale_sensitivity;

      dev.move_mouse(dx as i32, -(dy as i32)).unwrap();
    }

    *last_process_x = Some(x);
    *last_process_y = Some(y);
  }
}

pub fn touch_thread() -> JoinHandle<()> {
  let mut touch_dev = crate::utils::scan_dev(TOUCH).expect("Failed to find pad device");
  let mut virtual_dev: VirtualDevice =
    VirtualDevice::default().expect("Failed to create virtual device");

  println!("Linked: {}", TOUCH);

  spawn(move || {
    let mut tripletap_active = false;

    let scale_sensitivity = 0.8;

    let mut current_x: Option<i32> = None;
    let mut current_y: Option<i32> = None;
    let mut last_abs_before_tripletap_x: Option<i32> = None;
    let mut last_abs_before_tripletap_y: Option<i32> = None;
    let mut last_process_x: Option<i32> = None;
    let mut last_process_y: Option<i32> = None;

    loop {
      for ev in touch_dev.fetch_events().unwrap() {
        tripletap(
          &ev,
          &mut tripletap_active,
          &mut last_process_x,
          &mut last_process_y,
          &last_abs_before_tripletap_x,
          &last_abs_before_tripletap_y,
        );
        absolute_moves(
          &ev,
          tripletap_active,
          &mut current_x,
          &mut current_y,
          &mut last_abs_before_tripletap_x,
          &mut last_abs_before_tripletap_y,
        );
        synchronization(
          &ev,
          &mut virtual_dev,
          &mut current_x,
          &mut current_y,
          scale_sensitivity,
          &mut last_process_x,
          &mut last_process_y,
        );
      }
    }
  })
}
