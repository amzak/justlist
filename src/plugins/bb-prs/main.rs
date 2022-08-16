use model::Response;
use serde::Deserialize;
use shared::plugin::JustListPlugin;
use shared::serialization::{ListGroup, SelectableItem};
use shared::{plugin::JustListAction, serialization::Groups};
use structopt::StructOpt;

pub mod model;

#[derive(Debug, StructOpt)]
struct Options {
    url: String,
    token: String,
    command_template: String,
}

struct ListPullRequests {}

impl ListPullRequests {
    fn process_response(response: attohttpc::Response, command: &str) -> ListGroup {
        let reader = response.text_reader();
        let mut de = serde_json::Deserializer::from_reader(reader);
        let prs = Response::deserialize(&mut de).unwrap();

        let mut group = ListGroup {
            label: "PR".to_string(),
            items: vec![],
            command_template: Some(command.to_owned()),
            is_terminal: Some(false),
        };

        for pull_request in prs.values.iter() {
            let label = format!("[{}] {}", pull_request.state, pull_request.title);
            group.items.push(SelectableItem {
                label: label,
                param: pull_request.link().to_string(),
            });
        }

        group
    }
}

impl JustListAction<Options> for ListPullRequests {
    fn execute(&self, groups: &mut Groups, options: &Options) {
        let result = attohttpc::get(&options.url)
            .bearer_auth(&options.token)
            .send();

        match result {
            Ok(response) => {
                let prs_group =
                    ListPullRequests::process_response(response, &options.command_template);
                groups.groups.push(prs_group);
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
