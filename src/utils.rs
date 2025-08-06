use evdev::Device;
use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, KeyCode, RelativeAxisCode};
use std::io::Result;

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
