use shared::serialization::*;
use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(Debug, StructOpt)]
struct Options {
    extension: String,
    command_template: String,
    #[structopt(long, short)]
    verbose: bool,
    #[structopt(long, short, default_value = "1")]
    depth: u8,
    #[structopt(long, short, parse(from_os_str))]
    working_dir: Option<PathBuf>,
}

fn main() -> std::io::Result<()> {
    let options = Options::from_args();

    let working_dir = options.working_dir.unwrap_or(env::current_dir().unwrap());
    let depth = options.depth;

    let mut list = ListWithGroups {
        groups: vec![],
        command_template: options.command_template,
    };

    let mut group = Group {
        label: "files".to_string(),
        items: vec![],
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
        let extension = path.extension();

        if path.is_dir() {
            continue;
        }

        if extension.is_none() {
            continue;
        }

        let ext = extension.unwrap();
        let filter_ext = OsStr::new(&options.extension);

        if filter_ext == ext {
            let file_name = path.file_name().unwrap();
            let file_path = path.to_str().unwrap();

            group.items.push(SelectableItem {
                label: String::from(file_name.to_str().unwrap()),
                param: String::from(file_path),
            })
        }
    }

    list.groups.push(group);

    let stdout = std::io::stdout();
    let stdout_handle = stdout.lock();
    let writer = std::io::BufWriter::new(stdout_handle);
    serde_json::to_writer(writer, &list)?;

    Ok(())
}
