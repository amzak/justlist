use crate::{GroupModel, StatefulList};
use std::collections::HashMap;

pub struct State {
    pub lists: Vec<StatefulList>,
    pub groups: StatefulList,
    index_map: HashMap<usize, usize>,
    input: String,
    input_changed: bool,
}

impl State {
    pub fn new(items: &Vec<GroupModel>) -> State {
        State {
            lists: items.iter().map(|x| StatefulList::from(&x.items)).collect(),
            groups: StatefulList::from(&items),
            index_map: HashMap::new(),
            input: String::new(),
            input_changed: false,
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
        self.input_changed = true;
    }

    pub fn dump_input<'b, 'a: 'b>(&'a self) -> &'b str {
        self.input.as_str()
    }

    pub fn handle_backspace(&mut self) {
        self.input.pop();
        self.input_changed = true;
    }

    pub fn handle_escape(&mut self) {
        self.input.clear();
        self.input_changed = true;
    }

    pub fn was_input_changed(&self) -> bool {
        self.input_changed
    }

    pub fn reset(&mut self) {
        self.input_changed = false;
        self.index_map.clear();
    }

    pub fn map_index(&mut self, index_global: usize, index_local: usize) {
        self.index_map.insert(index_local, index_global);
    }

    pub fn get_by_local_index(&self, index_local: usize) -> usize {
        self.index_map[&index_local]
    }

    pub fn is_empty(&self) -> bool {
        self.input.len() == 0
    }
}
