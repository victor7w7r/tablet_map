pub struct GradualScroll {
  velocity: f64,
  decay_factor: f64,
  sensitivity: f64,
  active_threshold: f64,
}

impl GradualScroll {
  pub fn new(decay_factor: f64, sensitivity: f64) -> Self {
    Self {
      velocity: 0.0,
      decay_factor,
      sensitivity,
      active_threshold: 0.1,
    }
  }

  pub fn add_impulse(&mut self, impulse: f64) {
    self.velocity += impulse * self.sensitivity
  }

  pub fn get_next_value(&mut self) -> f64 {
    let current_value = self.velocity;

    self.velocity *= self.decay_factor;

    if self.velocity.abs() < self.active_threshold {
      self.velocity = 0.0
    }

    current_value
  }

  pub fn is_active(&self) -> bool {
    self.velocity.abs() >= self.active_threshold
  }
}
