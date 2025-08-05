use crate::gradual::GradualScroll;
use crate::utils::{PAD, find_device_by_name, setup_virtual_device};
use evdev::uinput::VirtualDevice;
use evdev::{Device, EventType, InputEvent, KeyCode, RelativeAxisCode, SynchronizationCode};
use std::io::Result;
use std::time::{Duration, Instant};

const RING_CODE: u16 = 8;
const RING_MAX: i32 = 71;

pub struct PadManager {
  device: Device,
  virtual_device: VirtualDevice,
  last_event_time: Instant,
  ring_position: i32,
  gradual_scroll: GradualScroll,
  last_movements: Vec<(Instant, f64)>,
  acceleration_factor: f64,
}

impl PadManager {
  pub fn new() -> Result<Self> {
    let device_path = match find_device_by_name(PAD)? {
      Some(path) => path,
      None => {
        return Err(std::io::Error::new(
          std::io::ErrorKind::NotFound,
          "Tablet not found",
        ));
      }
    };

    let device = Device::open(&device_path)?;
    let virtual_device = setup_virtual_device(
      PAD,
      &Some(vec![
        KeyCode::BTN_LEFT,
        KeyCode::BTN_MIDDLE,
        KeyCode::BTN_RIGHT,
      ]),
      &Some(vec![
        RelativeAxisCode::REL_X,
        RelativeAxisCode::REL_Y,
        RelativeAxisCode::REL_WHEEL,
      ]),
    )?;

    Ok(PadManager {
      device,
      virtual_device,
      last_event_time: Instant::now(),
      ring_position: 0,
      gradual_scroll: GradualScroll::new(0.45, 1.1),
      last_movements: Vec::with_capacity(2),
      acceleration_factor: 1.0,
    })
  }

  pub fn run_single_iteration(&mut self) -> Result<()> {
    let events: Vec<InputEvent> = self.device.fetch_events()?.collect();
    for ev in &events {
      self.process_event(ev)?;
    }

    if self.gradual_scroll.is_active() {
      let scroll_value = self.gradual_scroll.get_next_value();
      self.emit_scroll(scroll_value as i32)?;
    }

    Ok(())
  }

  pub fn run(&mut self) -> Result<()> {
    println!("Wacom Intuos5 pad manager running...");
    loop {
      self.run_single_iteration()?;
      std::thread::sleep(Duration::from_millis(5));
    }
  }

  fn process_event(&mut self, event: &InputEvent) -> Result<()> {
    if event.event_type() == EventType::ABSOLUTE && event.code() == RING_CODE {
      let now = Instant::now();
      let new_position = event.value();
      let delta = self.calculate_ring_delta(new_position);

      if delta != 0 {
        self.update_movement_history(now, delta as f64);
        self.update_acceleration_factor();
        let scaled_delta = delta as f64 * self.acceleration_factor;
        self.gradual_scroll.add_impulse(scaled_delta);
      }

      self.ring_position = new_position;
      self.last_event_time = now;
    }

    // Por ejemplo, botones específicos de Intuos5
    if event.event_type() == EventType::KEY {
      match event.code() {
        // Códigos de botón para Intuos5
        256..=259 => {
          // Ajustar según los botones reales
          if event.value() == 1 {
            // Presionado
            // Lógica para botones
            println!("Botón pad presionado: {}", event.code());
          }
        }
        _ => {}
      }
    }

    Ok(())
  }

  fn calculate_ring_delta(&self, new_position: i32) -> i32 {
    let mut delta = new_position - self.ring_position;
    if delta.abs() > RING_MAX / 2 {
      if delta > 0 {
        delta -= RING_MAX + 1;
      } else {
        delta += RING_MAX + 1;
      }
    }

    delta
  }

  fn update_movement_history(&mut self, now: Instant, delta: f64) {
    self.last_movements.push((now, delta));
    while self.last_movements.len() > 10 {
      self.last_movements.remove(0);
    }
    let cutoff = now - Duration::from_millis(500);
    self.last_movements.retain(|(t, _)| *t >= cutoff);
  }

  fn update_acceleration_factor(&mut self) {
    if self.last_movements.len() < 2 {
      self.acceleration_factor = 1.0;
      return;
    }

    let mut total_delta = 0.0;
    let mut total_time = 0.0;

    for i in 1..self.last_movements.len() {
      let (t1, _) = self.last_movements[i - 1];
      let (t2, d2) = self.last_movements[i];

      total_delta += d2.abs();
      total_time += t2.duration_since(t1).as_secs_f64();
    }

    if total_time > 0.001 {
      let avg_speed = total_delta / total_time;

      self.acceleration_factor = if avg_speed < 100.0 {
        1.0 + (avg_speed / 100.0)
      } else if avg_speed < 200.0 {
        2.0 + (avg_speed - 100.0) / 50.0
      } else {
        4.0 + (avg_speed - 200.0) / 100.0
      };

      self.acceleration_factor = self.acceleration_factor.min(8.0);
    } else {
      self.acceleration_factor = 1.0;
    }
  }

  fn emit_scroll(&mut self, amount: i32) -> Result<()> {
    if amount != 0 {
      self
        .virtual_device
        .emit(&[InputEvent::new(EventType::RELATIVE.0, RING_CODE, -amount)])?;

      self.virtual_device.emit(&[InputEvent::new(
        EventType::SYNCHRONIZATION.0,
        SynchronizationCode::SYN_REPORT.0,
        0,
      )])?;
    }
    Ok(())
  }
}
