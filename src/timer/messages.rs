// Commands sent to the timer engine.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerCommand {
    Start,
    Pause,
    Resume,
    Stop,
    Quit,
}

// Events emitted by the timer engine for a UI or other consumer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TimerEvent {
    Started {
        duration: u64,
    },
    Tick {
        remaining_secs: u64,
        elapsed_secs: u64,
        progress: f64,
    },
    Paused,
    Resumed,
    Stopped,
    Finished,
}
