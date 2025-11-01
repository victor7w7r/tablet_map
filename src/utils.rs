use input_linux::EvdevHandle;
use std::fs::File;
use std::process::{Child, Command, Output};
use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread::{JoinHandle, sleep, spawn};
use std::time::{Duration, Instant};

pub fn scan_dev(name: &str) -> Option<EvdevHandle<File>> {
  for entry in std::fs::read_dir("/dev/input").ok()? {
    let path = entry.ok()?.path();

    if path.file_name()?.to_str()?.starts_with("event") {
      if let Ok(file) = File::open(&path) {
        let dev = EvdevHandle::new(file);
        if let Ok(dev_name_bytes) = dev.device_name() {
          if let Ok(dev_name) = String::from_utf8(dev_name_bytes) {
            if dev_name.contains(&name) {
              return Some(dev);
            }
          }
        }
      }
    }
  }

  None
}

pub fn assign_dev(name: &str, name_wl: &str) -> EvdevHandle<File> {
  scan_dev(&name)
    .or_else(|| scan_dev(&name_wl))
    .expect("Failed to find device")
}

pub fn run_scroll(rx: Receiver<i32>) -> JoinHandle<()> {
  spawn(move || {
    let throttle_ms = 12u64;
    loop {
      let mut acc: i32 = match rx.recv() {
        Ok(v) => v,
        Err(_) => break,
      };

      let start = Instant::now();
      while start.elapsed() < Duration::from_millis(throttle_ms) {
        match rx.try_recv() {
          Ok(v) => acc = acc.saturating_add(v),
          Err(TryRecvError::Empty) => sleep(Duration::from_millis(1)),
          Err(TryRecvError::Disconnected) => break,
        }
      }

      if acc == 0 {
        continue;
      }

      send_scroll(&acc.to_string());
    }
  })
}

pub fn send_event(value: &str) -> Option<Output> {
  Command::new("ydotool")
    .args(&["click", value])
    .output()
    .ok()
}

pub fn send_scroll(value: &str) -> Option<Child> {
  Command::new("ydotool")
    .args(&["mousemove", "-w", "--", "0", value])
    .spawn()
    .ok()
}
