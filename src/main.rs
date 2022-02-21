#[macro_use]
extern crate lazy_static;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::borrow::Cow;
use std::io::BufReader;
use std::iter::Filter;

use std::{
    io,
    time::{Duration, Instant},
};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Corner, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{List, ListItem, ListState, Paragraph, Tabs},
    Frame, Terminal,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct SelectableItem {
    label: String,
    param: String,
}

#[derive(Serialize, Deserialize)]
struct Group {
    label: String,
    items: Vec<SelectableItem>,
}

#[derive(Serialize, Deserialize)]
struct ListWithGroups {
    groups: Vec<Group>,
    command_template: String,
}

struct SelectableItemModel {
    label: String,
    param: String,
}

struct GroupModel {
    label: String,
    items: Vec<SelectableItemModel>,
}

lazy_static! {}

struct StatefulList {
    state: ListState,
    len: usize,
}

impl StatefulList {
    fn from<T>(items: &Vec<T>) -> StatefulList {
        let mut result = StatefulList {
            state: ListState::default(),
            len: items.len(),
        };

        result.state.select(Some(0));

        result
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
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

    fn get_selected(&self) -> usize {
        self.state.selected().unwrap()
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

struct AppModel {
    groups: Vec<GroupModel>,
}

impl<'a> AppModel {
    fn new<R>(reader: R) -> AppModel
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

struct State {
    lists: Vec<StatefulList>,
    groups: StatefulList,
    input: String,
}

impl State {
    fn new(items: &Vec<GroupModel>) -> State {
        State {
            lists: items.iter().map(|x| StatefulList::from(&x.items)).collect(),
            groups: StatefulList::from(&items),
            input: String::new(),
        }
    }

    fn select_item_next(&mut self) {
        let selected_group = self.groups.get_selected();
        self.lists[selected_group].next();
    }

    fn select_item_prev(&mut self) {
        let selected_group = self.groups.get_selected();
        self.lists[selected_group].previous();
    }

    fn get_selected_group(&self) -> usize {
        self.groups.get_selected()
    }

    fn handle_char(&mut self, c: char) {
        self.input.push(c);
    }

    fn get_input(&self) -> &str {
        self.input.as_str()
    }
}

fn main() -> std::io::Result<()> {
    enable_raw_mode()?;

    let stdin = std::io::stdin();
    let reader = BufReader::new(stdin.lock());

    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(250);
    let app = AppModel::new(reader);
    let state = State::new(&app.groups);
    let res = run_app(&mut terminal, app, state, tick_rate);

    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: AppModel,
    mut state: State,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();

    loop {
        Terminal::draw(terminal, |f: &mut tui::Frame<B>| ui(f, &app, &mut state))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Left => state.groups.next(),
                    KeyCode::Right => state.groups.previous(),
                    KeyCode::Down => state.select_item_next(),
                    KeyCode::Up => state.select_item_prev(),
                    KeyCode::Char(c) => state.handle_char(c),
                    KeyCode::Backspace => {
                        state.input.pop();
                    }
                    KeyCode::Esc => state.input.clear(),
                    _ => {}
                }
                .clone()
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &AppModel, state: &mut State) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    render_tabs(f, app, state, chunks[0]);
    render_input(f, state, chunks[1]);
    render_list(f, app, state, chunks[2]);
}

fn render_tabs<B: Backend>(f: &mut Frame<B>, app: &AppModel, state: &mut State, chunk: Rect) {
    let groups = app
        .groups
        .iter()
        .map(|group| {
            Spans::from(vec![Span::styled(
                group.label.as_str(),
                Style::default().fg(Color::Yellow),
            )])
        })
        .collect();
    let tabs = Tabs::new(groups)
        .select(state.get_selected_group())
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(tabs, chunk);
}

fn render_input<B: Backend>(f: &mut Frame<B>, state: &mut State, chunk: Rect) {
    let input = Paragraph::new(state.input.as_str()).style(Style::default().fg(Color::Yellow));
    f.render_widget(input, chunk);
}

fn render_list<B: Backend>(f: &mut Frame<B>, app: &AppModel, state: &mut State, chunk: Rect) {
    let list = create_list(app, state);

    let group_index = state.get_selected_group();

    let ref mut state = state.lists[group_index].state;

    f.render_stateful_widget(list, chunk, state);
}

fn create_list<'b, 'a: 'b>(app: &'a AppModel, state: &State) -> List<'a> {
    let query = state.get_input();
    let filter = |x: &&SelectableItemModel| x.label.contains(query);

    let selected_group_index = state.get_selected_group();

    let items: Vec<_> = app.groups[selected_group_index]
        .items
        .iter()
        .filter(filter)
        .map(|list_item| {
            ListItem::new(list_item.label.as_str()).style(Style::default().fg(Color::White))
        })
        .collect();

    let list = List::new(items)
        .start_corner(Corner::TopLeft)
        .highlight_style(
            Style::default()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    list
}
