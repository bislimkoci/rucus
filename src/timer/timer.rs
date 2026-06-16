use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Timer {
    duration: Duration,
    remaining: Duration,
}

impl Timer {
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            remaining: duration,
        }
    }

    pub fn step_one_sec(&mut self) {
        self.remaining = self.remaining.saturating_sub(Duration::from_secs(1));
    }

    pub fn is_finished(&self) -> bool {
        self.remaining.is_zero()
    }

    pub fn remaining_secs(&self) -> u64 {
        self.remaining.as_secs()
    }

    pub fn elapsed_secs(&self) -> u64 {
        self.duration.saturating_sub(self.remaining).as_secs()
    }

    pub fn progress(&self) -> f64 {
        let total = self.duration.as_secs_f64();

        if total <= 0.0 {
            return 1.0;
        }

        let elapsed = self.duration.saturating_sub(self.remaining).as_secs_f64();
        elapsed / total
    }
}
