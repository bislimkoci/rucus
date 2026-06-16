use std::time::Duration;

use crossbeam::channel::{Receiver, TryRecvError};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::timer::messages::TimerEvent;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TimerStatus {
    Waiting,
    Running,
    Paused,
    Stopped,
    Finished,
}

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
        let progress = (self.progress.clamp(0.0, 1.0) * 100.0).round();
        let text = format!(
            "Status: {}\nDuration: {}\nRemaining: {}\nElapsed: {}\nProgress: {:.0}%\n\nPress q or Esc to quit.",
            self.status.label(),
            format_time(self.duration_secs),
            format_time(self.remaining_secs),
            format_time(self.elapsed_secs),
            progress,
        );

        let widget =
            Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Rucus"));
        frame.render_widget(widget, frame.area());
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
