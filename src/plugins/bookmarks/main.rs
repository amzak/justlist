use std::io::BufReader;

use atty::Stream;
use serde::Deserialize;
use shared::serialization::Groups;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    url: String,
    command_template: String,
}

fn main() -> std::io::Result<()> {
    let options = Options::from_args();

    let stdin = std::io::stdin();
    let handle = stdin.lock();
    let reader = BufReader::new(handle);

    let mut groups: Groups = Groups { groups: vec![] };
    if atty::isnt(Stream::Stdin) {
        let mut de = serde_json::Deserializer::from_reader(reader);
        groups = Groups::deserialize(&mut de).unwrap();
    }

    let result = attohttpc::get(options.url).send();

    match result {
        Ok(response) => {
            let mut received_groups = process_response(response, &options.command_template);
            groups.groups.append(&mut received_groups.groups);
        }
        Err(err) => println!("{}", err),
    }

    let stdout = std::io::stdout();
    let stdout_handle = stdout.lock();
    let writer = std::io::BufWriter::new(stdout_handle);
    serde_json::to_writer(writer, &groups)?;

    Ok(())
}

fn process_response(response: attohttpc::Response, command: &str) -> Groups {
    let reader = response.text_reader();
    let mut de = serde_json::Deserializer::from_reader(reader);
    let mut groups = Groups::deserialize(&mut de).unwrap();

    for mut group in groups.groups.iter_mut() {
        group.command_template = Some(String::from(command));
    }

    groups
}
