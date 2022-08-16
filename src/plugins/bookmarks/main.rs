use serde::Deserialize;
use shared::{plugin::JustListAction, serialization::Groups};
use structopt::StructOpt;

use shared::plugin::JustListPlugin;

#[derive(Debug, StructOpt)]
struct Options {
    url: String,
    command_template: String,
}

struct ListPullRequests {}

impl ListPullRequests {
    fn process_response(response: attohttpc::Response, command: &str) -> Groups {
        let reader = response.text_reader();
        let mut de = serde_json::Deserializer::from_reader(reader);
        let mut groups = Groups::deserialize(&mut de).unwrap();

        for mut group in groups.groups.iter_mut() {
            group.command_template = Some(String::from(command));
        }

        groups
    }
}

impl JustListAction<Options> for ListPullRequests {
    fn execute(&self, groups: &mut Groups, options: &Options) {
        let result = attohttpc::get(&options.url).send();

        match result {
            Ok(response) => {
                let mut received_groups =
                    ListPullRequests::process_response(response, &options.command_template);
                groups.groups.append(&mut received_groups.groups);
            }
            Err(err) => println!("{}", err),
        }
    }
}

fn main() -> std::io::Result<()> {
    let options = Options::from_args();

    let plugin = JustListPlugin::new(options);

    let action = ListPullRequests {};
    plugin.main(&action)
}
