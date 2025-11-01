use crate::utils::{assign_dev, run_scroll, send_event};
use input_linux::{EventKind, InputEvent};
use std::io::Result;
use std::sync::mpsc::{Sender, channel};
use std::thread::sleep;
use std::time::{Duration, Instant};

pub struct Pad {
  dev_name: &'static str,
  dev_wl_name: &'static str,
  tx: Sender<i32>,
  prev: Option<i32>,
  last_ts: Option<Instant>,
  range: i32,
  jump_threshold: i32,
  scale: f64,
}

impl Pad {
  pub fn new() -> Result<Self> {
    let (tx, rx) = channel::<i32>();

    run_scroll(rx);

    Ok(Pad {
      dev_name: "Wacom Intuos5 touch S Pad",
      dev_wl_name: "Wacom Intuos5 touch S (WL) Pad",
      tx,
      prev: None,
      last_ts: None,
      range: 72,
      jump_threshold: 14,
      scale: 0.00001,
    })
  }

  fn key_map(&mut self, ev: &InputEvent) -> Result<()> {
    if ev.kind == EventKind::Key && ev.value == 1 {
      if ev.code == 257 || ev.code == 260 {
        send_event("0xC0");
      } else if ev.code == 258 || ev.code == 261 {
        send_event("0xC2");
      } else if ev.code == 259 || ev.code == 262 {
        send_event("0xC1");
      }
      sleep(Duration::from_millis(100));
    }
    Ok(())
  }

  fn scroll_map(&mut self, ev: &InputEvent) -> Result<()> {
    if ev.kind != EventKind::Absolute {
      return Ok(());
    }

    if ev.value == 0 {
      self.prev = None;
      self.last_ts = None;
      return Ok(());
    }

    let now = Instant::now();

    let prev = match self.prev {
      None => {
        self.prev = Some(ev.value);
        self.last_ts = Some(now);
        return Ok(());
      }
      Some(v) => v,
    };

    let raw = (ev.value - prev).rem_euclid(self.range);
    let mut signed = if raw > self.range / 2 {
      raw - self.range
    } else {
      raw
    };

    signed = -signed;

    if signed.abs() > self.jump_threshold {
      self.prev = Some(ev.value);
      self.last_ts = Some(now);
      return Ok(());
    }

    if signed == 0 {
      self.prev = Some(ev.value);
      self.last_ts = Some(now);
      return Ok(());
    }

    let dt = self
      .last_ts
      .map(|t| now.duration_since(t).as_secs_f64())
      .unwrap_or(0.01)
      .max(0.001);
    let speed = (signed.abs() as f64) / dt;
    let mut magnitude = (speed * self.scale).powf(0.9).round() as i32;
    if magnitude < 1 {
      magnitude = 1;
    }

    let amt = if signed > 0 { magnitude } else { -magnitude };
    let _ = self.tx.send(amt);
    self.prev = Some(ev.value);
    self.last_ts = Some(now);
    Ok(())
  }

  pub fn exec(&mut self) -> Result<()> {
    let pad_dev = assign_dev(self.dev_name, self.dev_wl_name);
    pad_dev.grab(true)?;
    println!("Linked and Grabbed: {}", self.dev_name);

    loop {
      match pad_dev.read_input_event() {
        Ok(ev) => {
          self.key_map(&ev)?;
          self.scroll_map(&ev)?;
        }
        Err(_) => sleep(Duration::from_millis(5)),
      }
    }
  }
}
