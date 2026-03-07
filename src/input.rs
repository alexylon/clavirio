use crossterm::event::{Event, EventStream, KeyEvent, KeyEventKind};
use futures::StreamExt;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy)]
pub enum InputEvent {
    Press(KeyEvent),
    Tick,
}

pub async fn run_input_loop(tx: mpsc::UnboundedSender<InputEvent>) {
    let mut stream = EventStream::new();

    loop {
        let event = tokio::select! {
            maybe = stream.next() => maybe,
            _ = tokio::time::sleep(std::time::Duration::from_millis(200)) => {
                let _ = tx.send(InputEvent::Tick);
                continue;
            }
        };

        let key = match event {
            Some(Ok(Event::Key(k))) if k.kind == KeyEventKind::Press => k,
            Some(Ok(_)) => {
                let _ = tx.send(InputEvent::Tick);
                continue;
            }
            Some(Err(_)) | None => break,
        };

        if tx.send(InputEvent::Press(key)).is_err() {
            break;
        }
    }
}
