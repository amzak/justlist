use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SelectableItem {
    pub label: String,
    pub param: String,
}

#[derive(Serialize, Deserialize)]
pub struct Group {
    pub label: String,
    pub items: Vec<SelectableItem>,
}

#[derive(Serialize, Deserialize)]
pub struct ListWithGroups {
    pub groups: Vec<Group>,
    pub command_template: String,
}
