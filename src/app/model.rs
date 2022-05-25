use super::domain::{GroupModel, SelectableItemModel};
use crate::app::domain::LaunchModel;
use crate::State;
use serde::Deserialize;
use shared::serialization::Groups;
use std::env;

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
                        is_terminal: group.is_terminal,
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

    pub fn handle_enter(&self, state: &State) -> LaunchModel {
        let selected_group_index = state.get_selected_group();
        let selected_list = &state.lists[selected_group_index];
        let selected_item_index = selected_list.get_selected();

        let global_index = state.get_by_local_index(selected_item_index);

        let group = &self.groups[selected_group_index];
        let selected_item_model = &group.items[global_index];

        let mut cwd = env::current_exe().unwrap();
        cwd.pop();
        cwd.push("launcher");

        let launch = LaunchModel {
            executable: group.command_template.clone(),
            param: Some(selected_item_model.param.clone()),
            is_terminal: group.is_terminal.unwrap_or(false),
        };

        launch
    }
}
