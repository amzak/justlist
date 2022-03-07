#[macro_use]
extern crate lazy_static;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::io::BufReader;
use structopt::StructOpt;

use std::{
    io,
    time::{Duration, Instant},
};

use tui::{
    backend::{Backend, CrosstermBackend},
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

    enable_raw_mode()?;

    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(250);
    let app = read_app_model(options);
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
                    KeyCode::Backspace => state.handle_backspace(),
                    KeyCode::Esc => state.handle_escape(),
                    KeyCode::Enter => app.handle_enter(&state),
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
    let input = state.get_input();
    let paragraph = Paragraph::new(input).style(Style::default().fg(Color::Yellow));
    f.render_widget(paragraph, chunk);
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
