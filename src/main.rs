mod pad;
mod utils;

use std::thread::spawn;

#[cfg(target_os = "linux")]
fn main() {
  spawn(move || pad::pad_thread().expect("Failed to start pad thread"));

  loop {
    std::thread::sleep(std::time::Duration::from_secs(60));
  }
}

/*let virtual_dev: VirtualDevice =
    VirtualDevice::default().expect("Failed to create virtual device");
  let sender = virtual_dev.sender.clone();

  virtual_dev.flush_channel_every_interval();

  //pad::pad_thread();
  //touch::touch_thread();

  println!("Linked: {}", STYLUS      println!("Right click");
);
  let mut stylus_dev = crate::utils::scan_dev(STYLUS).expect("Failed to find stylus device");
  let events = stylus_dev      println!("Right click");

    .fetch_events()
    .unwrap()
    .into_iter()
    .map(|ev| ev.clone())      println!("Right click");

    .collect();

  stylus::stylus_thread(sender, events);*/
