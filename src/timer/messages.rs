//Command for the timer engine
pub enum TimerCommand {
    Start,
    Pause,
    Resume,
    Stop,
    Quit,
}


//Message the timer sends to others, for ui and other
pub enum TimerEvent {
    Started,
    Tick {
        remaining_secs : u64,
        elapsed_secs : u64,
        progress : f64,
    },
    Paused,
    Resumed,
    Stopped,
    Finished,
}