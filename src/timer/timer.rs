use std::{thread, time::Duration};

#[derive(Clone, Debug)]
pub struct Timer {
    duration : Duration,
    remaining : Duration,
}

impl Timer {
    pub fn new(duration : Duration) -> Self {
        Self {
            duration : duration,
            remaining : duration,
        }
    }

    //temporary function
    pub fn start(&mut self) {

        while self.remaining.as_secs() > 0 {
            println!("Remaining: {}", Self::remaining_time(self.remaining.as_secs()));
            thread::sleep(Duration::from_secs(1));
            self.remaining -= Duration::new(1, 0);
        }

        println!("Done");
    }

    pub fn step_one_sec(&mut self) {

        if self.remaining.is_zero() {
            println!("Done");
        }
        self.remaining = self.remaining.saturating_sub(Duration::new(1, 0));

    }

    pub fn remaining_time(t : u64) -> String {
        let minutes = t / 60;
        let seconds = t % 60;
        format!("{minutes:02}:{seconds:02}")
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

//For a thread we want:
// read message, that being user command, if state = idle - blocked until a message arrives else:
// check if message arrives, if not: continue in the loop