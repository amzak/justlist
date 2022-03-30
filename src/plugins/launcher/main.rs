use std::process::Command;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    command: String,
    argument: String,
}

fn main() {
    nix::unistd::setsid().unwrap();
    let options = Options::from_args();

    let command = options.command;
    let argument = options.argument;

    Command::new(&command).arg(argument).spawn().unwrap();
}
