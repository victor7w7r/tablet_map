mod pad;
mod stylus;
mod touch;
mod utils;

use std::thread::spawn;

#[cfg(target_os = "linux")]
fn main() {
  spawn(move || pad::Pad::new().unwrap().exec().unwrap());
  spawn(move || touch::Touch::new().unwrap().exec().unwrap());
  spawn(move || stylus::Stylus::new().unwrap().exec().unwrap());

  loop {
    std::thread::sleep(std::time::Duration::from_secs(60));
  }
}
