use clap::{Parser, Subcommand};


#[derive(Parser, Debug)]
#[command(name = "rucus")]
#[command(about = "A small focus app")]
pub struct Cli {
    #[command(subcommand)]
    pub command : Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Start {
        duration : String,
    },
}
