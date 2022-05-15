use atty::Stream;
use serde::de::Deserialize;
use shared::serialization::*;
use std::env;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(Debug, StructOpt)]
struct Options {
    command_template: String,
    #[structopt(long, short, default_value = "1")]
    depth: u8,
    #[structopt(long, short, parse(from_os_str))]
    working_dir: Option<PathBuf>,
    #[structopt(long, short)]
    verbose: bool,
}

fn is_git_repo(path: &Path) -> bool {
    let is_dir = path.is_dir();
    let file_name = path.file_name().unwrap().to_str().unwrap();

    return is_dir && file_name.ends_with(".git");
}

fn main() -> std::io::Result<()> {
    let options = Options::from_args();

    let working_dir = options.working_dir.unwrap_or(env::current_dir().unwrap());
    let depth = options.depth;

    let stdin = std::io::stdin();
    let handle = stdin.lock();
    let reader = BufReader::new(handle);

    let mut groups: Groups = Groups { groups: vec![] };
    if atty::isnt(Stream::Stdin) {
        let mut de = serde_json::Deserializer::from_reader(reader);
        groups = Groups::deserialize(&mut de).unwrap();
    }

    let mut group = ListGroup {
        label: "git repos".to_string(),
        items: vec![],
        command_template: options.command_template,
    };

    for item in WalkDir::new(&working_dir).max_depth(depth as usize) {
        if let Err(_error) = item {
            if options.verbose {
                eprintln!("{}", _error);
            }
            continue;
        }

        let dir_item = item.unwrap();
        let path = dir_item.path();

        if !is_git_repo(path) {
            continue;
        }

        if let Some(parent_dir) = path.parent() {
            let file_path = path.to_str().unwrap();
            let dir_name = parent_dir.file_name().unwrap().to_str().unwrap();

            group.items.push(SelectableItem {
                label: String::from(dir_name),
                param: String::from(file_path),
            })
        }
    }

    groups.groups.push(group);

    let stdout = std::io::stdout();
    let stdout_handle = stdout.lock();
    let writer = std::io::BufWriter::new(stdout_handle);
    serde_json::to_writer(writer, &groups)?;

    Ok(())
}
