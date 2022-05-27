#[macro_use]
extern crate lazy_static;

use crate::app::domain::LaunchModel;
use crate::terminal::TerminalState;
use crossterm::event::{self, Event, KeyCode};

use std::{env::consts::FAMILY, io::BufReader, process::Output};
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

use std::env;
use std::io::{self, Write};
use std::process::Command;

mod app;
use app::{domain::GroupModel, model::AppModel, state::State, stateful::StatefulList};

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
    let app = read_app_model(options);

    let result = _main(app);

    match result {
        Ok(launch) => {
            execute_launch(launch);
        }
        Err(error) => {
            print!("{}", error);
        }
    }
    Ok(())
}

fn execute_launch(launch: LaunchModel) {
    if launch.executable.is_none() {
        return;
    }

    let LaunchModel {
        executable,
        param,
        is_terminal,
    } = launch;

    if FAMILY == "windows" {
        launch_windows(&executable.unwrap(), &param.unwrap());
        return;
    }

    let child_result = if is_terminal {
        launch_inplace(&executable.unwrap(), &param.unwrap())
    } else {
        launch_external(&executable.unwrap(), &param.unwrap())
    };

    match child_result {
        Err(error) => print!("{}", error.to_string()),
        Ok(output) => {
            println!("status: {}", output.status);
            io::stdout().write_all(&output.stdout).unwrap();
            io::stderr().write_all(&output.stderr).unwrap();
        }
    }
}

fn launch_windows(exec: &str, param: &str) {
    Command::new(exec).arg(param).spawn();
}

fn launch_inplace(exec: &str, param: &str) -> io::Result<Output> {
    Command::new(exec).arg(param).output()
}

fn launch_external(exec: &str, param: &str) -> io::Result<Output> {
    let mut launcher_command = env::current_exe().unwrap();
    launcher_command.pop();
    launcher_command.push("launcher");

    Command::new(launcher_command).arg(exec).arg(param).output()
}

fn _main(app: AppModel) -> io::Result<LaunchModel> {
    let mut terminal_state = TerminalState::new();
    let state = State::new(&app.groups);
    return run_app(&mut terminal_state.terminal, app, state);
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: AppModel,
    mut state: State,
) -> std::io::Result<LaunchModel> {
    loop {
        Terminal::draw(terminal, |f: &mut tui::Frame<B>| ui(f, &app, &mut state))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(LaunchModel::default()),
                KeyCode::Left => state.groups.previous(),
                KeyCode::Right => state.groups.next(),
                KeyCode::Down => state.select_item_next(),
                KeyCode::Up => state.select_item_prev(),
                KeyCode::Char(c) => state.handle_char(c),
                KeyCode::Backspace => state.handle_backspace(),
                KeyCode::Esc => state.handle_escape(),
                KeyCode::Enter => {
                    let launch = app.handle_enter(&state);
                    return Ok(launch);
                }
                _ => return Ok(LaunchModel::default()),
            }
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
