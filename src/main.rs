use std::thread;

use clap::Parser;
use crossbeam::channel::unbounded;

use crate::{
    cli::Cli,
    timer::{
        engine::run_timer_thread,
        messages::TimerCommand,
        timer::Timer,
    },
    ui::app,
};

mod cli;
mod timer;
mod ui;

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


            let ui_thread = thread::spawn(move || app::start(event_receiver));

            command_sender.send(TimerCommand::Start).unwrap();
            

            // Wait for the worker to fully exit and propagate a panic if it had one.
            ui_thread.join().unwrap().unwrap();
            timer_thread.join().unwrap();
        }
    }
}
