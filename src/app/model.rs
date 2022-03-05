use crate::app::serialization::ListWithGroups;
use serde::Deserialize;

use super::domain::{GroupModel, SelectableItemModel};

pub struct AppModel {
    pub groups: Vec<GroupModel>,
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
                                .map(|x| SelectableItemModel {
                                    label: x.label.clone(),
                                    param: x.param.clone(),
                                })
                                .collect(),
                        })
                        .collect(),
                };
            }
            Err(e) => {
                panic!("can't read json state: {}", e.to_string());
            }
        };
    }
}
