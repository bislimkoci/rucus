use std::time::Duration;

use crossbeam::channel::{Receiver, TryRecvError};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
};

use crate::timer::messages::TimerEvent;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TimerStatus {
    Waiting,
    Running,
    Paused,
    Stopped,
    Finished,
}

const ORANGE: Color = Color::Rgb(255, 165, 0);

impl TimerStatus {
    fn label(self) -> &'static str {
        match self {
            TimerStatus::Waiting => "Waiting",
            TimerStatus::Running => "Running",
            TimerStatus::Paused => "Paused",
            TimerStatus::Stopped => "Stopped",
            TimerStatus::Finished => "Finished",
        }
    }
}

pub struct App {
    duration_secs: u64,
    remaining_secs: u64,
    elapsed_secs: u64,
    progress: f64,
    status: TimerStatus,
    should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            duration_secs: 0,
            remaining_secs: 0,
            elapsed_secs: 0,
            progress: 0.0,
            status: TimerStatus::Waiting,
            should_quit: false,
        }
    }

    pub fn start(self, events: Receiver<TimerEvent>) -> std::io::Result<()> {
        let mut terminal = ratatui::init();
        let result = self.run_app(&mut terminal, events);
        ratatui::restore();
        result
    }

    fn run_app(
        mut self,
        terminal: &mut ratatui::DefaultTerminal,
        events: Receiver<TimerEvent>,
    ) -> std::io::Result<()> {
        loop {
            self.update(&events);
            terminal.draw(|frame| self.render(frame))?;
            self.handle_input()?;

            if self.should_quit {
                break Ok(());
            }
        }
    }

    fn render(&self, frame: &mut ratatui::Frame) {
        let orange = Style::default().fg(ORANGE);
        let progress = self.progress.clamp(0.0, 1.0);
        let progress_percent = (progress * 100.0).round();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(11),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(frame.area());

        let rucus_block = Block::default()
            .borders(Borders::ALL)
            .border_style(orange)
            .title("Rucus");
        let timer_area = rucus_block.inner(layout[0]);
        let timer_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(timer_area);

        let total_block = Block::default()
            .borders(Borders::ALL)
            .border_style(orange)
            .title("Total Duration");
        let remaining_block = Block::default()
            .borders(Borders::ALL)
            .border_style(orange)
            .title("Remaining Time");

        let total_duration = Paragraph::new(big_time_lines(&format_time(self.duration_secs)))
            .style(orange)
            .alignment(Alignment::Center)
            .block(total_block);

        let remaining_time = Paragraph::new(big_time_lines(&format_time(self.remaining_secs)))
            .style(orange)
            .alignment(Alignment::Center)
            .block(remaining_block);

        let progress_bar = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(orange)
                    .title("Progress"),
            )
            .style(orange)
            .gauge_style(orange)
            .ratio(progress)
            .label(format!("{progress_percent:.0}%"));

        let status = Paragraph::new(self.status.label())
            .style(orange)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(orange)
                    .title("Status"),
            );

        frame.render_widget(rucus_block, layout[0]);
        frame.render_widget(total_duration, timer_layout[0]);
        frame.render_widget(remaining_time, timer_layout[1]);
        frame.render_widget(progress_bar, layout[1]);
        frame.render_widget(status, layout[2]);
    }

    fn update(&mut self, events: &Receiver<TimerEvent>) {
        loop {
            match events.try_recv() {
                Ok(event) => self.apply_event(event),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.should_quit = true;
                    break;
                }
            }
        }
    }

    fn apply_event(&mut self, event: TimerEvent) {
        match event {
            TimerEvent::Started { duration } => {
                self.duration_secs = duration;
                self.remaining_secs = duration;
                self.elapsed_secs = 0;
                self.progress = 0.0;
                self.status = TimerStatus::Running;
            }
            TimerEvent::Tick {
                remaining_secs,
                elapsed_secs,
                progress,
            } => {
                self.remaining_secs = remaining_secs;
                self.elapsed_secs = elapsed_secs;
                self.progress = progress;
                self.status = TimerStatus::Running;
            }
            TimerEvent::Paused => {
                self.status = TimerStatus::Paused;
            }
            TimerEvent::Resumed => {
                self.status = TimerStatus::Running;
            }
            TimerEvent::Stopped => {
                self.status = TimerStatus::Stopped;
                self.should_quit = true;
            }
            TimerEvent::Finished => {
                self.remaining_secs = 0;
                self.elapsed_secs = self.duration_secs;
                self.progress = 1.0;
                self.status = TimerStatus::Finished;
                self.should_quit = true;
            }
        }
    }

    fn handle_input(&mut self) -> std::io::Result<()> {
        if !event::poll(Duration::from_millis(50))? {
            return Ok(());
        }

        let event = event::read()?;

        if matches!(
            event,
            Event::Key(key)
                if key.kind == KeyEventKind::Press
                    && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
        ) {
            self.should_quit = true;
        }

        Ok(())
    }
}

pub fn start(events: Receiver<TimerEvent>) -> std::io::Result<()> {
    App::new().start(events)
}

fn format_time(total_secs: u64) -> String {
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    format!("{minutes:02}:{seconds:02}")
}

fn big_time_lines(time: &str) -> Vec<Line<'static>> {
    let mut rows = vec![
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
    ];

    for ch in time.chars() {
        let digit = big_digit(ch);

        for (row, piece) in rows.iter_mut().zip(digit) {
            row.push_str(piece);
            row.push(' ');
        }
    }

    rows.into_iter()
        .map(|row| Line::from(Span::styled(row, Style::default().fg(ORANGE))))
        .collect()
}

fn big_digit(ch: char) -> [&'static str; 5] {
    match ch {
        '0' => ["███", "█ █", "█ █", "█ █", "███"],
        '1' => [" ██", "  █", "  █", "  █", "  █"],
        '2' => ["███", "  █", "███", "█  ", "███"],
        '3' => ["███", "  █", "███", "  █", "███"],
        '4' => ["█ █", "█ █", "███", "  █", "  █"],
        '5' => ["███", "█  ", "███", "  █", "███"],
        '6' => ["███", "█  ", "███", "█ █", "███"],
        '7' => ["███", "  █", "  █", "  █", "  █"],
        '8' => ["███", "█ █", "███", "█ █", "███"],
        '9' => ["███", "█ █", "███", "  █", "███"],
        ':' => ["   ", " █ ", "   ", " █ ", "   "],
        _ => ["   ", "   ", "   ", "   ", "   "],
    }
}
