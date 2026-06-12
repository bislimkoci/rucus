use clap::{Parser};

use crate::{cli::Cli, timer::timer::Timer};

mod cli;
mod timer;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        cli::Command::Start {duration} => {
            let duration = humantime::parse_duration(&duration)
                .expect("Duration not fixed, use example is 40min, or 40m");

            let mut timer = Timer::new(duration);

            timer.start();

        },
    }
}




