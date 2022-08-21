use shared::plugin::{JustListAction, JustListPlugin};
use shared::serialization::*;
use std::env;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(Debug, StructOpt)]
#[structopt(about = "This plugin searches for git repos")]
struct Options {
    command_template: String,
    #[structopt(long, short, default_value = "1")]
    depth: u8,
    #[structopt(long, short, parse(from_os_str))]
    working_dir: Option<PathBuf>,
    #[structopt(long, short)]
    verbose: bool,
    #[structopt(
        long = "terminal",
        short = "t",
        help = "This flag indicates, that the current terminal should be reused"
    )]
    is_terminal: bool,
}

struct SearchGitRepos {}

impl JustListAction<Options> for SearchGitRepos {
    fn execute(&self, groups: &mut Groups, options: &Options) {
        let cwd = env::current_dir().unwrap();
        let working_dir = options.working_dir.as_ref().unwrap_or(&cwd);
        let depth = options.depth;

        let mut group = ListGroup {
            label: "git repos".to_string(),
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
    }
}

fn is_git_repo(path: &Path) -> bool {
    let is_dir = path.is_dir();
    let file_name = path.file_name().unwrap().to_str().unwrap();

    return is_dir && file_name.ends_with(".git");
}

fn main() -> std::io::Result<()> {
    let options = Options::from_args();

    let plugin = JustListPlugin::new(options);

    let action = SearchGitRepos {};
    plugin.main(&action)
}
