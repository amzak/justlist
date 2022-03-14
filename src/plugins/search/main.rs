use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(parse(from_os_str))]
    working_dir: Option<PathBuf>,
    #[structopt(short, long)]
    max_depth: u8,
    filter: String,
}

fn main() -> std::io::Result<()> {
    let mut options = Options::from_args();

    if options.working_dir.is_none() {
        options.working_dir = Some(env::current_dir().expect("can't get current directory"));
    }

    let working_dir = options.working_dir.unwrap();
    let depth = options.max_depth;

    let mut results: Vec<PathBuf> = Vec::new();

    for item in WalkDir::new(&working_dir).max_depth(depth as usize) {
        let dir_item = item.unwrap();

        if dir_item.path().is_dir() {
            continue;
        }

        let path = dir_item.path();

        let ext = path.extension().unwrap();
        let filter = OsStr::new(&options.filter);

        if filter == ext {
            results.push(path.to_path_buf());
        }
    }

    Ok(())
}
