use crate::{GroupModel, StatefulList};

pub struct State {
    pub lists: Vec<StatefulList>,
    pub groups: StatefulList,
    input: String,
}

impl State {
    pub fn new(items: &Vec<GroupModel>) -> State {
        State {
            lists: items.iter().map(|x| StatefulList::from(&x.items)).collect(),
            groups: StatefulList::from(&items),
            input: String::new(),
        }
    }

    pub fn select_item_next(&mut self) {
        let selected_group = self.groups.get_selected();
        self.lists[selected_group].next();
    }

    pub fn select_item_prev(&mut self) {
        let selected_group = self.groups.get_selected();
        self.lists[selected_group].previous();
    }

    pub fn get_selected_group(&self) -> usize {
        self.groups.get_selected()
    }

    pub fn handle_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn get_input(&self) -> &str {
        self.input.as_str()
    }

    pub fn handle_backspace(&mut self) {
        self.input.pop();
    }

    pub fn handle_escape(&mut self) {
        self.input.clear()
    }
}
