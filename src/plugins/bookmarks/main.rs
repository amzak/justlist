use serde::Deserialize;
use shared::{plugin::JustListAction, serialization::Groups};
use structopt::StructOpt;

use shared::plugin::JustListPlugin;

#[derive(Debug, StructOpt)]
#[structopt(about = "This plugin gets bookmarks from a JSON file on web or on disk")]
struct Options {
    path: String,
    user: Option<String>,
    password: Option<String>,
    command_template: String,
}

struct Bookmarks {}

impl Bookmarks {
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

impl JustListAction<Options> for Bookmarks {
    fn execute(&self, groups: &mut Groups, options: &Options) {
        let mut request = attohttpc::get(&options.path);

        let cred = options.user.as_ref()
            .zip(options.password.as_ref());

        if let Some((user, password)) = cred {
            request = request.basic_auth(user, Some(password));
        }

        let result = request.send();

        match result {
            Ok(response) => {
                let mut received_groups =
                    Bookmarks::process_response(response, &options.command_template);
                groups.groups.append(&mut received_groups.groups);
            }
            Err(err) => println!("{}", err),
        }
    }
}

fn main() -> std::io::Result<()> {
    let options = Options::from_args();

    let plugin = JustListPlugin::new(options);

    let action = Bookmarks {};
    plugin.main(&action)
}
