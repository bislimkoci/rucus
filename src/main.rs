use clap::{Parser};

use crate::cli::{Cli};

mod cli;

fn main() {

    let cli = Cli::parse();

    match cli.command {
        cli::Command::Start {duration} => println!("The duration is {}", duration),
    }

}
