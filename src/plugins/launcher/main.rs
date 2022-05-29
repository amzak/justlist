use std::env;
use std::process::Command;
use std::process::Stdio;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    command: String,
    argument: String,
}

#[cfg(target_family = "unix")]
fn setsid() {
    nix::unistd::setsid().unwrap();
}

#[cfg(target_family = "windows")]
fn setsid() {}

fn main() {
    setsid();

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
