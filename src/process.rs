use crate::pad::Pad;
use crate::pen::Pen;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{sleep, spawn};
use std::time::Duration;

pub fn setup_cancel(running: Arc<AtomicBool>) {
  let r = Arc::clone(&running);

  ctrlc::set_handler(move || {
    println!("\nClosing...");
    r.store(false, Ordering::SeqCst);
  })
  .expect("ERROR: Error setting Ctrl-C handler");
}

pub fn check_service() {
  if !Command::new("pidof")
    .arg("ydotoold")
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .expect("ERROR: pidof failed")
    .success()
  {
    eprintln!("ERROR: ydotoold is not running, exiting...");
    std::process::exit(1);
  }
}

pub fn start(stop_flag: Arc<AtomicBool>) {
  let stop_flag_pad = Arc::clone(&stop_flag);
  let stop_flag_stylus = Arc::clone(&stop_flag);

  spawn(move || {
    while !stop_flag_pad.load(Ordering::Relaxed) {
      if let Ok(mut pad) = Pad::new() {
        let _ = pad.exec();
      }
      if !stop_flag_pad.load(Ordering::Relaxed) {
        sleep(Duration::from_secs(2));
      }
    }
  });

  spawn(move || {
    while !stop_flag_stylus.load(Ordering::Relaxed) {
      if let Ok(mut stylus) = Pen::new() {
        let _ = stylus.exec();
      }
      if !stop_flag_stylus.load(Ordering::Relaxed) {
        sleep(Duration::from_secs(2));
      }
    }
  });
}
