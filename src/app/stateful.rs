use tui::widgets::ListState;

pub struct StatefulList {
    pub state: ListState,
    pub len: usize,
}

impl StatefulList {
    pub fn from<T>(items: &Vec<T>) -> StatefulList {
        let mut result = StatefulList {
            state: ListState::default(),
            len: items.len(),
        };

        result.state.select(Some(0));

        result
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == self.len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn get_selected(&self) -> usize {
        self.state.selected().unwrap()
    }
}
