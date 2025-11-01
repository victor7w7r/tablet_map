use std::io::Result;
use std::process::Command;

pub fn generate() -> Result<()> {
  let systemd_dir = dirs::home_dir().unwrap().join(".config/systemd/user");
  let bin_dir = dirs::home_dir().unwrap().join(".local/bin");
  let bin_path = bin_dir.join("tablet-map").display().to_string();
  std::fs::create_dir_all(&bin_dir)?;
  std::fs::create_dir_all(&systemd_dir)?;

  let unit_path = systemd_dir.join(format!("tablet-map.service"));
  let unit_content = format!(
    "[Unit]
      Description=Tablet Map Service for Wacom Intuos5

      [Service]
      ExecStart={bin_path}
      Restart=no
      StandardOutput=journal
      StandardError=journal

      [Install]
      WantedBy=default.target
    "
  );

  std::fs::write(&unit_path, unit_content)?;

  Command::new("systemctl")
    .args(["--user", "daemon-reload"])
    .status()
    .ok();
  Command::new("systemctl")
    .args(["--user", "enable", "--now", "tablet-map.service"])
    .status()
    .ok();

  println!("✅ Service installed: {}", unit_path.display());
  Ok(())
}
