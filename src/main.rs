use std::thread;

use clap::Parser;
use crossbeam::channel::unbounded;

use crate::{
    cli::Cli,
    timer::{
        engine::run_timer_thread,
        messages::{TimerCommand, TimerEvent},
        timer::Timer,
    },
};

mod cli;
mod timer;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        cli::Command::Start { duration } => {
            let duration = humantime::parse_duration(&duration)
                .expect("Duration not fixed, use example is 40min, or 40m");

            let timer = Timer::new(duration);

            // Channel 1: main sends commands; the timer thread receives them.
            let (command_sender, command_receiver) = unbounded();

            // Channel 2: the timer thread sends events; main receives them.
            let (event_sender, event_receiver) = unbounded();

            // Spawn a second OS thread. `move` transfers timer, command_receiver,
            // and event_sender into the closure, so the worker owns them.
            let timer_thread = thread::spawn(move || {
                run_timer_thread(timer, command_receiver, event_sender);
            });

            command_sender.send(TimerCommand::Start).unwrap();

            // Iterating over Receiver blocks main until the worker sends an event.
            // It ends if every event sender is dropped, unless we break first.
            for event in event_receiver {
                match event {
                    // A Tick carries a snapshot; only remaining_secs is displayed here.
                    TimerEvent::Tick { remaining_secs, .. } => {
                        println!("Remaining: {}", Timer::remaining_time(remaining_secs));
                    }
                    // Finished means the worker has completed the countdown.
                    TimerEvent::Finished => {
                        println!("Done");
                        break;
                    }
                    
                    TimerEvent::Started
                    | TimerEvent::Paused
                    | TimerEvent::Resumed
                    | TimerEvent::Stopped => {}
                }
            }

            // Wait for the worker to fully exit and propagate a panic if it had one.
            timer_thread.join().unwrap();
        }
    }
}
