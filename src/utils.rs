use evdev::Device;

pub const PAD: &str = "Wacom Intuos5 touch S Pad";
pub const TOUCH: &str = "Wacom Intuos5 touch S Finger";
pub const STYLUS: &str = "Wacom Intuos5 touch S Pen";

pub fn scan_dev(nombre: &str) -> Option<Device> {
  for entry in std::fs::read_dir("/dev/input").ok()? {
    let path = entry.ok()?.path();

    if path.file_name()?.to_str()?.starts_with("event") {
      if let Ok(dev) = Device::open(&path) {
        if let Some(dev_name) = dev.name() {
          if dev_name.contains(nombre) {
            return Some(dev);
          }
        }
      }
    }
  }

  None
}
