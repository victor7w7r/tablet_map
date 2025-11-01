mod pad;
mod pen;
mod process;
mod service;
mod utils;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

#[cfg(target_os = "linux")]
fn main() {
  if std::env::args().any(|a| a == "-i") {
    service::generate().unwrap();
    std::process::exit(0);
  }

  let running = Arc::new(AtomicBool::new(true));
  let mut wacom_connected = false;
  let mut stop_flag: Option<Arc<AtomicBool>> = None;

  process::setup_cancel(Arc::clone(&running));
  println!("Initializing, waiting for tablet and ydotool...");

  while running.load(Ordering::SeqCst) {
    let is_connected = utils::scan_dev("Wacom Intuos5").is_some();

    if is_connected && !wacom_connected {
      println!("Tablet detected, starting threads...");

      if let Some(ref flag) = stop_flag {
        flag.store(true, Ordering::Relaxed);
      }

      sleep(Duration::from_millis(500));

      let new_stop_flag = Arc::new(AtomicBool::new(false));
      process::start(Arc::clone(&new_stop_flag));
      stop_flag = Some(new_stop_flag);
      wacom_connected = true;
    } else if !is_connected && wacom_connected {
      println!("Tablet disconnected, stopping threads...");

      if let Some(ref flag) = stop_flag {
        flag.store(true, Ordering::Relaxed);
      }

      wacom_connected = false;
    }
    process::check_service();
    sleep(Duration::from_secs(2));
  }

  if let Some(ref flag) = stop_flag {
    flag.store(true, Ordering::Relaxed);
  }
}
