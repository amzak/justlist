pub struct SelectableItemModel {
    pub index: usize,
    pub label: String,
    pub param: String,
}

pub struct GroupModel {
    pub label: String,
    pub items: Vec<SelectableItemModel>,
    pub command_template: String,
}

pub struct LaunchModel {
    pub executable: Option<String>,
    pub param: Option<String>,
}

impl LaunchModel {
    pub fn default() -> LaunchModel {
        LaunchModel {
            executable: None,
            param: None,
        }
    }
}
