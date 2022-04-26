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
