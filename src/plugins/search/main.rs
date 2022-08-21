use shared::plugin::{JustListAction, JustListPlugin};
use shared::serialization::*;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(Debug)]
struct QueryFlags {
    names: bool,
    directories: bool,
    extensions: bool,
}

fn parse_query_flags(s: &str) -> QueryFlags {
    let mut names = s.len() == 0; // if nothing was specified, search for names
    let mut directories = false;
    let mut extensions = false;

    for c in s.chars() {
        match c {
            'n' => names = true,
            'd' => directories = true,
            'e' => extensions = true,
            _ => {}
        }
    }

    QueryFlags {
        names,
        directories,
        extensions,
    }
}

#[derive(Debug, StructOpt)]
struct Options {
    query: String,
    command_template: String,
    #[structopt(short, parse(from_str = parse_query_flags), help="n - for names, d - for directories, e - for extensions")]
    query_flags: QueryFlags,
    #[structopt(long, short)]
    verbose: bool,
    #[structopt(long, short, default_value = "1")]
    depth: u8,
    #[structopt(long, short, parse(from_os_str))]
    working_dir: Option<PathBuf>,
    #[structopt(long)]
    title: Option<String>,
    #[structopt(
        long = "terminal",
        short = "t",
        help = "This flag should indicate, that the app runs in a terminal"
    )]
    is_terminal: bool,
}

impl Options {
    pub fn get_working_dir_or<'b, 'a: 'b>(&'a self, default: &'b PathBuf) -> &'b PathBuf {
        self.working_dir.as_ref().unwrap_or(default)
    }
}

struct Search {}

impl JustListAction<Options> for Search {
    fn execute(&self, groups: &mut Groups, options: &Options) {
        let curr_dir = env::current_dir().unwrap();
        let working_dir = options.get_working_dir_or(&curr_dir);
        let depth = options.depth;

        let title = if options.title.is_some() {
            options.title.as_ref().unwrap()
        } else {
            "files"
        };

        let mut group = ListGroup {
            label: title.to_string(),
            items: vec![],
            command_template: Some(options.command_template.to_string()),
            is_terminal: Some(options.is_terminal),
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

            if is_match(&options.query, path, &options.query_flags) {
                let file_name = path.file_name().unwrap();
                let file_path = path.to_str().unwrap();

                group.items.push(SelectableItem {
                    label: String::from(file_name.to_str().unwrap()),
                    param: String::from(file_path),
                });
            }
        }

        groups.groups.push(group);
    }
}

fn is_match(query: &str, path: &Path, query_flags: &QueryFlags) -> bool {
    let is_dir = path.is_dir();
    let mut result = false;
    let file_name = path.file_name().unwrap().to_str().unwrap();

    if is_dir && query_flags.directories {
        result |= file_name.contains(query);
    }

    if query_flags.extensions && !is_dir {
        if let Some(extension_wrap) = path.extension() {
            let extension = extension_wrap.to_str().unwrap();
            result |= extension.contains(query);
        }
    }

    if query_flags.names && !is_dir {
        result |= file_name.contains(query);
    }

    result
}

fn main() -> std::io::Result<()> {
    let options = Options::from_args();

    let plugin = JustListPlugin::new(options);

    let action = Search {};
    plugin.main(&action)
}
