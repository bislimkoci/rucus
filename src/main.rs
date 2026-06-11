use std::{thread, time::Duration};

use clap::{Parser};

use crate::cli::{Cli};

mod cli;

fn main() {
    let cli = Cli::parse();
    match cli.command {
        cli::Command::Start {duration} => {
            let duration = humantime::parse_duration(&duration)
                .expect("Duration not fixed, use example is 40min, or 40m");

            basic_timer(duration);
        },
    }
}



fn basic_timer(duration : Duration) {
    let mut remaining = duration.as_secs();

    while remaining > 0 {
        println!("Remaining: {}", remaining_time(remaining));
        thread::sleep(Duration::from_secs(1));
        remaining -= 1;
    }

    println!("Done");

}

fn remaining_time(t : u64) -> String {
    let minutes = t / 60;
    let seconds = t % 60;
    format!("{minutes:02}:{seconds:02}")
}