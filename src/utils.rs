use evdev::Device;
use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, KeyCode, RelativeAxisCode};
use std::fs::read_dir;
use std::io::Result;
use std::path::Path;

pub const PAD: &str = "Wacom Intuos5 touch S Pad";
pub const TOUCH: &str = "Wacom Intuos5 touch S Finger";
pub const STYLUS: &str = "Wacom Intuos5 touch S Pen";

pub fn find_device_by_name(name_pattern: &str) -> Result<Option<String>> {
  let devices_path = Path::new("/dev/input");
  let mut matching_device = None;

  for entry in read_dir(devices_path)? {
    let entry = entry?;
    let path = entry.path();

    if let Some(file_name) = path.file_name() {
      if let Some(file_name_str) = file_name.to_str() {
        if file_name_str.starts_with("event") {
          if let Ok(device) = Device::open(&path) {
            if let Some(device_name) = device.name() {
              if device_name.contains(name_pattern) {
                matching_device = Some(path.to_string_lossy().to_string());
                break;
              }
            }
          }
        }
      }
    }
  }

  Ok(matching_device)
}

pub fn setup_virtual_device(
  name: &str,
  keys: &Option<Vec<KeyCode>>,
  axes: &Option<Vec<RelativeAxisCode>>,
) -> Result<VirtualDevice> {
  let mut builder = VirtualDevice::builder()?.name(name);

  if let Some(keys) = keys
    && !keys.is_empty()
  {
    builder = builder.with_keys(&AttributeSet::from_iter(keys.iter()))?;
  }

  if let Some(axes) = axes
    && !axes.is_empty()
  {
    builder = builder.with_relative_axes(&AttributeSet::from_iter(axes.iter()))?;
  }

  Ok(builder.build()?)
}
