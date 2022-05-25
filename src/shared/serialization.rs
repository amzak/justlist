use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SelectableItem {
    pub label: String,
    pub param: String,
}

#[derive(Serialize, Deserialize)]
pub struct ListGroup {
    pub label: String,
    pub items: Vec<SelectableItem>,
    pub command_template: Option<String>,
    pub is_terminal: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct Groups {
    pub groups: Vec<ListGroup>,
}
