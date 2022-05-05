use super::domain::{GroupModel, SelectableItemModel};
use crate::State;
use serde::Deserialize;
use shared::serialization::Groups;
use std::env;
use std::io::{self, Write};
use std::process::Command;
use std::process::Stdio;

pub struct AppModel {
    pub groups: Vec<GroupModel>,
}

impl<'a> AppModel {
    pub fn new<R>(reader: R) -> AppModel
    where
        R: std::io::Read,
    {
        let mut de = serde_json::Deserializer::from_reader(reader);
        let data = Groups::deserialize(&mut de);

        match data {
            Ok(content) => {
                let model_groups = content
                    .groups
                    .iter()
                    .map(|group| GroupModel {
                        label: group.label.clone(),
                        command_template: group.command_template.clone(),
                        items: group
                            .items
                            .iter()
                            .enumerate()
                            .map(|(index, x)| SelectableItemModel {
                                label: x.label.to_lowercase(),
                                param: x.param.clone(),
                                index: index,
                            })
                            .collect(),
                    })
                    .collect();

                let model = AppModel {
                    groups: model_groups,
                };

                return model;
            }
            Err(e) => {
                panic!("can't read json state: {}", e.to_string());
            }
        };
    }

    pub fn handle_enter(&self, state: &State) -> std::io::Result<()> {
        let selected_group_index = state.get_selected_group();
        let selected_list = &state.lists[selected_group_index];
        let selected_item_index = selected_list.get_selected();

        let global_index = state.get_by_local_index(selected_item_index);

        let group = &self.groups[selected_group_index];
        let selected_item_model = &group.items[global_index];

        let mut cwd = env::current_exe().unwrap();
        cwd.pop();
        cwd.push("launcher");

        let child_result = Command::new(cwd)
            .arg(&group.command_template)
            .arg(&selected_item_model.param)
            .output();

        match child_result {
            Err(error) => return Err(error),
            Ok(output) => {
                println!("status: {}", output.status);
                io::stdout().write_all(&output.stdout).unwrap();
                io::stderr().write_all(&output.stderr).unwrap();
            }
        }

        Ok(())
    }
}
