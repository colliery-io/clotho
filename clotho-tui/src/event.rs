use crossterm::event::{Event, EventStream, KeyEvent, MouseEvent};
use futures::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc;

/// Terminal events the app loop handles.
#[derive(Debug)]
pub enum AppEvent {
    /// A key was pressed.
    Key(KeyEvent),
    /// Mouse event.
    Mouse(MouseEvent),
    /// Terminal was resized.
    Resize(u16, u16),
    /// Periodic tick for polling store updates.
    Tick,
}

/// Spawns a background task that reads terminal events via async EventStream
/// and emits ticks on a fixed interval for store polling.
pub fn spawn_event_reader(tick_rate: Duration) -> mpsc::UnboundedReceiver<AppEvent> {
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        let mut event_stream = EventStream::new();
        let mut tick_interval = tokio::time::interval(tick_rate);

        loop {
            tokio::select! {
                maybe_event = event_stream.next() => {
                    match maybe_event {
                        Some(Ok(Event::Key(key))) => {
                            if tx.send(AppEvent::Key(key)).is_err() { return; }
                        }
                        Some(Ok(Event::Mouse(mouse))) => {
                            if tx.send(AppEvent::Mouse(mouse)).is_err() { return; }
                        }
                        Some(Ok(Event::Resize(w, h))) => {
                            if tx.send(AppEvent::Resize(w, h)).is_err() { return; }
                        }
                        Some(Ok(_)) => {}
                        Some(Err(_)) => return,
                        None => return,
                    }
                }
                _ = tick_interval.tick() => {
                    if tx.send(AppEvent::Tick).is_err() { return; }
                }
            }
        }
    });

    rx
}
