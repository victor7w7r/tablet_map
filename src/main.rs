mod gradual;
mod pad;
mod utils;

use pad::PadManager;

#[cfg(target_os = "linux")]
fn main() -> std::io::Result<()> {
  let mut pad_manager = PadManager::new()?;
  pad_manager.run()?;

  Ok(())

  /*
  loop {
    std::thread::sleep(std::time::Duration::from_secs(60));
  }*/
}
