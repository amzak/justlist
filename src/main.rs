#![deny(elided_lifetimes_in_paths)]

#[macro_use]
extern crate lazy_static;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

use serde::{Deserialize, Serialize};
use std::fmt::Display;

use std::fmt;

#[derive(Serialize, Deserialize)]
struct SelectableItem<'a> {
    label: &'a str,
    param: &'a str
}

#[derive(Serialize, Deserialize)]
struct Group<'a> {
    label: &'a str,
    items: Vec<SelectableItem<'a>>
}

struct SelectableItemModel<'a> {
    label: &'a str,
    param: &'a str
}

struct GroupModel<'a> {
    label: &'a str
}

#[derive(Serialize, Deserialize)]
struct ListWithGroups<'a> {
    #[serde(borrow)]
    groups: Vec<Group<'a>>,
    command_template: &'a str,
}

lazy_static! {
}

fn sample<'a>() -> serde_json::Result<ListWithGroups<'a>> {
    let json: &'a str = r#"
    {
        "command_template": "qqq",
        "groups": [
            {
                "label": "group 1",
                "items": [
                {
                    "label": "item 1",
                    "param": "xxx"
                },
                {
                    "label": "item 2",
                    "param": "yyy"
                }]
            },
            {
                "label": "group 2",
                "items": [
                {
                    "label": "item 3",
                    "param": "qqq"
                },
                {
                    "label": "item 4",
                    "param": "www"
                }]
            }
        ]
    }"#;

    let list: ListWithGroups<'a> = serde_json::from_str(json)?;
    Ok(list)
}

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        let mut result = StatefulList {
            state: ListState::default(),
            items,
        };

        result.state.select(Some(0));

        result
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
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
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

struct App<'a> {
    items: StatefulList<SelectableItemModel<'a>>,
    groups: StatefulList<GroupModel<'a>>,
    source: ListWithGroups<'a>
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        let sample = sample().unwrap();

        App {
            groups: StatefulList::with_items(
                sample.groups
                .iter()
                .map(|x| GroupModel { label: x.label })
                .collect()),
            items: StatefulList::with_items(
                sample.groups[0].items
                .iter()
                .map(|x| SelectableItemModel {label: x.label, param: x.param})
                .collect()
            ),
            source: sample,
        }
    }
}

fn main() -> std::io::Result<()> {
    enable_raw_mode()?;

    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(250);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

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
    mut app: App<'_>,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Left => app.groups.next(),
                    KeyCode::Right => app.groups.previous(),
                    KeyCode::Down => app.items.next(),
                    KeyCode::Up => app.items.previous(),
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<'_, B>, app: &mut App<'_>) {
    render_list(f, app);
}

fn render_list<B: Backend>(f: &mut Frame<'_, B>, app: &mut App<'_>) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let items: Vec<_> = app
        .items
        .items
        .iter()
        .map(|group| {
            ListItem::new(group.label)
                .style(Style::default()
                .fg(Color::White)
           )
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

    f.render_stateful_widget(list, chunks[0], &mut app.items.state);
}