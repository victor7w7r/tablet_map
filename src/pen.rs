use crate::utils::{assign_dev, run_scroll};
use input_linux::{EventKind, InputEvent};
use std::io::Result;
use std::sync::mpsc::{Sender, channel};
use std::thread::sleep;
use std::time::Duration;

pub struct Pen {
  dev_name: &'static str,
  dev_wl_name: &'static str,
  tx: Sender<i32>,
  active: bool,
  sensitivity: f64,
  origin: Option<i32>,
  deadzone: i32,
  exponent: f64,
  max_steps: i32,
  max_steps_per_event: i32,
  acc: f64,
}

impl Pen {
  pub fn new() -> Result<Self> {
    let (tx, rx) = channel::<i32>();

    run_scroll(rx);

    Ok(Pen {
      dev_name: "Wacom Intuos5 touch S Pen",
      dev_wl_name: "Wacom Intuos5 touch S (WL) Pen",
      tx,
      active: false,
      origin: None,
      deadzone: 15,
      exponent: 2.0,
      sensitivity: 1.0,
      max_steps: 8,
      max_steps_per_event: 3,
      acc: 0.0,
    })
  }

  fn key_map(&mut self, ev: &InputEvent) -> Result<()> {
    if ev.kind != EventKind::Key || ev.code != 331 {
      return Ok(());
    }

    match ev.value {
      1 => {
        self.active = true;
        self.origin = None;
        self.acc = 0.0;
      }
      0 => {
        self.active = false;
        self.origin = None;
        self.acc = 0.0;
      }
      _ => {}
    }

    sleep(Duration::from_millis(100));
    Ok(())
  }

  fn scroll_map(&mut self, ev: &InputEvent) -> Result<()> {
    if ev.kind != EventKind::Absolute || ev.code != 1 {
      return Ok(());
    }

    if !self.active {
      self.origin = None;
      self.acc = 0.0;
      return Ok(());
    }

    let y = ev.value;

    if self.origin.is_none() {
      self.origin = Some(y);
      return Ok(());
    }

    let origin = self.origin.unwrap();

    let delta = y - origin;
    let abs_delta = delta.abs();

    if abs_delta <= self.deadzone {
      return Ok(());
    }

    let max_from_origin = std::cmp::max(origin, 20000 - origin) as f64;
    let max_from_origin = if max_from_origin <= 0.0 {
      20000.0
    } else {
      max_from_origin
    };

    let ratio = (abs_delta as f64) / max_from_origin;
    let ratio_clamped = ratio.min(1.0);

    let base = ratio_clamped.powf(self.exponent);
    let fractional_steps = base * (self.max_steps as f64) * self.sensitivity;
    let signed_frac = if delta > 0 {
      -fractional_steps
    } else {
      fractional_steps
    };
    self.acc += signed_frac;
    let mut send = self.acc.trunc() as i32;
    if send == 0 {
      return Ok(());
    }

    if send > self.max_steps_per_event {
      send = self.max_steps_per_event;
    } else if send < -self.max_steps_per_event {
      send = -self.max_steps_per_event;
    }
    self.acc -= send as f64;
    let _ = self.tx.send(send);

    Ok(())
  }

  pub fn exec(&mut self) -> Result<()> {
    let pen_dev = assign_dev(self.dev_name, self.dev_wl_name);
    println!("Linked: {}", self.dev_name);

    loop {
      match pen_dev.read_input_event() {
        Ok(ev) => {
          self.key_map(&ev)?;
          self.scroll_map(&ev)?;
        }
        Err(_) => sleep(Duration::from_millis(5)),
      }
    }
  }
}
