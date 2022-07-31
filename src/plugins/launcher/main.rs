use std::process::Command;
use std::process::Stdio;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    command: String,
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

    let mut parts = options.command.split_whitespace();

    let first = parts.next();

    if first.is_none() {
        return;
    }

    let first_part = first.unwrap();

    let mut cmd = Command::new(first_part);

    for part in parts {
        cmd.arg(part);
    }

    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
}
