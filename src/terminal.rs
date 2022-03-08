use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::backend::CrosstermBackend;

use tui::Terminal;

pub struct TerminalState {
    pub terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    errors: Vec<String>,
}

impl TerminalState {
    pub fn new() -> TerminalState {
        enable_raw_mode();
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture);
        let backend: CrosstermBackend<std::io::Stdout> = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend);

        TerminalState {
            terminal: terminal.unwrap(),
            errors: vec![],
        }
    }

    pub fn error(&mut self, err: &str) {
        self.errors.push(String::from(err));
    }
}

impl Drop for TerminalState {
    fn drop(&mut self) {
        disable_raw_mode();

        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );

        self.terminal.show_cursor();
    }
}
