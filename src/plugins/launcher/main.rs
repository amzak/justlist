use std::process::Command;
use std::process::Stdio;
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

    Command::new(&command)
        .arg(argument)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
}
