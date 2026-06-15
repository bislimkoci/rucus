use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::widgets::Paragraph;

/// Starts the terminal UI and restores the terminal when the UI closes.
pub fn start() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run_app(&mut terminal);
    ratatui::restore();
    result
}

fn run_app(terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
    loop {
        terminal.draw(render)?;

        if should_quit()? {
            break Ok(());
        }
    }
}

fn render(frame: &mut ratatui::Frame) {
    let greeting = Paragraph::new("Hello, world!");
    frame.render_widget(greeting, frame.area());
}

fn should_quit() -> std::io::Result<bool> {
    let event = event::read()?;

    Ok(matches!(
        event,
        Event::Key(key)
            if key.kind == KeyEventKind::Press
                && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
    ))
}
