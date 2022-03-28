use super::domain::{GroupModel, SelectableItemModel};
use crate::State;
use serde::Deserialize;
use shared::serialization::ListWithGroups;
use std::process::Child;
use std::process::Command;

pub struct AppModel {
    pub groups: Vec<GroupModel>,
    pub command_template: String,
}

impl<'a> AppModel {
    pub fn new<R>(reader: R) -> AppModel
    where
        R: std::io::Read,
    {
        let mut de = serde_json::Deserializer::from_reader(reader);
        let data = ListWithGroups::deserialize(&mut de);

        match data {
            Ok(content) => {
                return AppModel {
                    groups: content
                        .groups
                        .iter()
                        .map(|x| GroupModel {
                            label: x.label.clone(),
                            items: x
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
                        .collect(),
                    command_template: content.command_template,
                };
            }
            Err(e) => {
                panic!("can't read json state: {}", e.to_string());
            }
        };
    }

    pub fn handle_enter(&self, state: &State) -> std::io::Result<Child> {
        let selected_group_index = state.get_selected_group();
        let selected_list = &state.lists[selected_group_index];
        let selected_item_index = selected_list.get_selected();

        let global_index = state.get_by_local_index(selected_item_index);

        let selected_item_model = &self.groups[selected_group_index].items[global_index];

        let param = selected_item_model.param.as_str();

        Command::new(&self.command_template).arg(param).spawn()
    }
}
