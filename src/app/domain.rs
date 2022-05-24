pub struct SelectableItemModel {
    pub index: usize,
    pub label: String,
    pub param: String,
}

pub struct GroupModel {
    pub label: String,
    pub items: Vec<SelectableItemModel>,
    pub command_template: String,
    pub is_terminal: bool,
}

pub struct LaunchModel {
    pub executable: Option<String>,
    pub param: Option<String>,
    pub is_terminal: bool,
}

impl LaunchModel {
    pub fn default() -> LaunchModel {
        LaunchModel {
            executable: None,
            param: None,
            is_terminal: false,
        }
    }
}
