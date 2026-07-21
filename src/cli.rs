use clap::{ArgAction, Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, action = ArgAction::SetTrue)]
    /// generate config file.
    pub config: bool,
}

pub struct Cli {
    pub args: Args,
}

impl Cli {
    pub fn new() -> Self {
        Self {
            args: Args::parse(),
        }
    }
}
