use std::time::Duration;

use crossbeam::channel::{Receiver, RecvTimeoutError, Sender};

use crate::timer::{
    messages::{TimerCommand, TimerEvent},
    timer::Timer,
};

#[derive(Clone, Copy)]
enum TimerState {
    Idle,
    Running,
    Paused,
}

pub fn run_timer_thread(
    timer: Timer,
    commands: Receiver<TimerCommand>,
    events: Sender<TimerEvent>,
) {
    run_timer_thread_with_interval(timer, commands, events, Duration::from_secs(1));
}

fn run_timer_thread_with_interval(
    mut timer: Timer,
    commands: Receiver<TimerCommand>,
    events: Sender<TimerEvent>,
    tick_interval: Duration,
) {
    let mut state = TimerState::Idle;

    // The worker stays alive until it finishes, is stopped, or loses a channel.
    loop {
        // None means a normal timer tick occurred, so the loop can start again.
        let command = match state {
            TimerState::Running => match commands.recv_timeout(tick_interval) {
                // A command arrived before the tick interval expired.
                Ok(command) => Some(command),
                Err(RecvTimeoutError::Timeout) => {
                    timer.step_one_sec();

                    if timer.is_finished() {
                        let _ = events.send(TimerEvent::Finished);
                        return;
                    }

                    // Report the updated remaining time to the main thread.
                    if send_tick(&timer, &events).is_err() {
                        return;
                    }

                    None
                }
                // Main dropped every command sender, so no future command can arrive.
                Err(RecvTimeoutError::Disconnected) => return,
            },
            // Idle and paused timers do not need a timeout because they must not tick.
            TimerState::Idle | TimerState::Paused => match commands.recv() {
                // recv blocks this worker efficiently until main sends a command.
                Ok(command) => Some(command),
                Err(_) => return,
            },
        };

        // Timeout ticks return None because their work was already done above.
        let Some(command) = command else {
            continue;
        };

        match command {
            TimerCommand::Start => {
                // Ignore repeated Start commands once the timer has begun.
                if matches!(state, TimerState::Idle) {
                    state = TimerState::Running;

                    // Tell main that the state transition succeeded.
                    if events
                        .send(TimerEvent::Started {
                            duration: timer.remaining_secs(),
                        })
                        .is_err()
                    {
                        return;
                    }

                    // A zero-duration timer finishes immediately.
                    if timer.is_finished() {
                        let _ = events.send(TimerEvent::Finished);
                        return;
                    }

                    // Send the initial value immediately instead of waiting one second.
                    if send_tick(&timer, &events).is_err() {
                        return;
                    }
                }
            }
            TimerCommand::Pause => {
                // Pause is meaningful only while actively running.
                if matches!(state, TimerState::Running) {
                    state = TimerState::Paused;
                    if events.send(TimerEvent::Paused).is_err() {
                        return;
                    }
                }
            }
            TimerCommand::Resume => {
                // Resume is meaningful only after a pause.
                if matches!(state, TimerState::Paused) {
                    state = TimerState::Running;
                    if events.send(TimerEvent::Resumed).is_err() {
                        return;
                    }
                }
            }
            TimerCommand::Stop => {
                // Stop reports the state change and then permanently ends this timer.
                let _ = events.send(TimerEvent::Stopped);
                return;
            }
            // Quit silently ends the worker.
            TimerCommand::Quit => return,
        }
    }
}

// Build and send one snapshot of the current timer values.
fn send_tick(timer: &Timer, events: &Sender<TimerEvent>) -> Result<(), ()> {
    events
        .send(TimerEvent::Tick {
            remaining_secs: timer.remaining_secs(),
            elapsed_secs: timer.elapsed_secs(),
            progress: timer.progress(),
        })
        // The caller only needs to know whether an event receiver still exists.
        .map_err(|_| ())
}
