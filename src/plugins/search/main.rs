use shared::serialization::*;
use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;
use walkdir::DirEntry;
use walkdir::WalkDir;

#[derive(Debug)]
enum SearchType {
    FileType,
    FileName,
    Directory,
}

type ParseError = &'static str;

impl FromStr for SearchType {
    type Err = ParseError;
    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "type" => Ok(SearchType::FileType),
            "filetype" => Ok(SearchType::FileType),
            "name" => Ok(SearchType::FileName),
            "filename" => Ok(SearchType::FileName),
            "file" => Ok(SearchType::FileName),
            "directory" => Ok(SearchType::Directory),
            _ => Err("not supported"),
        }
    }
}

#[derive(Debug, StructOpt)]
struct Options {
    search_type: SearchType,
    target: String,
    command_template: String,
    #[structopt(long, short)]
    verbose: bool,
    #[structopt(long, short, default_value = "1")]
    depth: u8,
    #[structopt(long, short, parse(from_os_str))]
    working_dir: Option<PathBuf>,
    #[structopt(long, short)]
    title: Option<String>,
}

fn main() -> std::io::Result<()> {
    let options = Options::from_args();

    let working_dir = options.working_dir.unwrap_or(env::current_dir().unwrap());
    let depth = options.depth;

    let mut list = ListWithGroups {
        groups: vec![],
        command_template: options.command_template,
    };

    let title = if options.title.is_some() {
        options.title.unwrap()
    } else {
        "files".to_string()
    };

    let mut group = Group {
        label: title,
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

        if is_match(&dir_item, &options.target, &options.search_type) {
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

fn is_match(dir_entry: &DirEntry, target: &str, search_type: &SearchType) -> bool {
    if search_type == SearchType::FileType {
        unimplemented!();
    }

    let path = dir_entry.path();
    let extension = path.extension();

    if path.is_dir() {
        return false;
    }

    if extension.is_none() {
        return false;
    }

    let ext = extension.unwrap();
    let filter_ext = OsStr::new(target);

    filter_ext == ext
}
