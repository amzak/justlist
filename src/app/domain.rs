pub struct SelectableItemModel {
    pub label: String,
    pub param: String,
}

pub struct GroupModel {
    pub label: String,
    pub items: Vec<SelectableItemModel>,
}
