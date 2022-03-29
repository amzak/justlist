#[macro_use]
extern crate lazy_static;

use crate::terminal::TerminalState;
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

use std::io::BufReader;
use structopt::StructOpt;

use tui::{
    backend::Backend,
    layout::{Constraint, Corner, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{List, ListItem, Paragraph, Tabs},
    Frame, Terminal,
};

use std::fs::File;
use std::path::PathBuf;

mod app;
use app::{
    domain::GroupModel, domain::SelectableItemModel, model::AppModel, state::State,
    stateful::StatefulList,
};

mod terminal;

lazy_static! {}

#[derive(Debug, StructOpt)]
pub struct Options {
    #[structopt(parse(from_os_str))]
    target: Option<PathBuf>,
}

fn read_app_model(options: Options) -> AppModel {
    let app: AppModel;

    if options.target.is_some() {
        let file_path: PathBuf = options.target.unwrap();
        let reader = BufReader::new(File::open(file_path).unwrap());
        app = AppModel::new(reader);
    } else {
        let stdin = std::io::stdin();
        let handle = stdin.lock();
        let reader = BufReader::new(handle);
        app = AppModel::new(reader);
    };

    app
}

fn main() -> std::io::Result<()> {
    let options = Options::from_args();

    let mut terminal_state = TerminalState::new();

    let app = read_app_model(options);
    let state = State::new(&app.groups);
    let res = run_app(&mut terminal_state.terminal, app, state);

    if let Err(err) = res {
        terminal_state.error(&err.to_string());
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: AppModel,
    mut state: State,
) -> std::io::Result<()> {
    loop {
        Terminal::draw(terminal, |f: &mut tui::Frame<B>| ui(f, &app, &mut state))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Left => state.groups.next(),
                KeyCode::Right => state.groups.previous(),
                KeyCode::Down => state.select_item_next(),
                KeyCode::Up => state.select_item_prev(),
                KeyCode::Char(c) => state.handle_char(c),
                KeyCode::Backspace => state.handle_backspace(),
                KeyCode::Esc => state.handle_escape(),
                KeyCode::Enter => {
                    let result = app.handle_enter(&state);
                    match result {
                        Ok(mut child) => {
                            match child.try_wait() {
                                Ok(Some(status)) => {
                                    println!("child process exited with: {}", status)
                                }
                                Ok(None) => {
                                    println!("no status yet");
                                    std::thread::sleep(Duration::from_secs(1));
                                }
                                Err(e) => return Err(e),
                            }
                            return Ok(());
                        }
                        Err(error) => return Err(error),
                    }
                }
                _ => {}
            }
            .clone()
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
    let input = state.dump_input();
    let paragraph = Paragraph::new(input).style(Style::default().fg(Color::Yellow));
    f.render_widget(paragraph, chunk);
}

fn render_list<B: Backend>(f: &mut Frame<B>, app: &AppModel, state: &mut State, chunk: Rect) {
    let group_index = state.get_selected_group();
    let input_was_changed = state.was_input_changed();
    state.reset();

    let list = create_list(app, state);

    let ref mut list_state = state.lists[group_index].state;

    if input_was_changed {
        list_state.select(Some(0));
    }

    f.render_stateful_widget(list, chunk, list_state);
}

fn create_list<'b, 'a: 'b>(app: &'a AppModel, state: &'b mut State) -> List<'a> {
    let selected_group_index = state.get_selected_group();
    let list = &app.groups[selected_group_index].items;
    let mut list_items: Vec<ListItem> = Vec::with_capacity(list.len());

    let mut index = 0;
    for item in list {
        if !item.label.contains(state.dump_input()) {
            continue;
        }

        let list_item = ListItem::new(item.label.as_str()).style(Style::default().fg(Color::White));

        list_items.push(list_item);
        state.map_index(item.index, index);
        index += 1;
    }

    let list = List::new(list_items)
        .start_corner(Corner::TopLeft)
        .highlight_style(
            Style::default()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    list
}
