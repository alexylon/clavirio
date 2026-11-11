use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventState, KeyModifiers};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy)]
pub enum InputEvent {
    Key(KeyEvent),
    Tick,
}

pub async fn run_input_loop(tx: mpsc::UnboundedSender<InputEvent>) {
    // crossterm::event::read() blocks, so run in a dedicated thread
    let _ = tokio::task::spawn_blocking(move || {
        loop {
            let ev = if event::poll(std::time::Duration::from_millis(200)).unwrap_or(false) {
                match event::read() {
                    Ok(Event::Key(k)) => InputEvent::Key(with_caps_lock(k)),
                    Ok(_) => InputEvent::Tick,
                    Err(_) => break,
                }
            } else {
                InputEvent::Tick
            };

            if tx.send(ev).is_err() {
                break;
            }
        }
    })
    .await;
}

/// Modifiers that make a key a shortcut, such as Ctrl+C, rather than text
const SHORTCUT_MODIFIERS: KeyModifiers = KeyModifiers::CONTROL
    .union(KeyModifiers::ALT)
    .union(KeyModifiers::SUPER);

/// Under the kitty keyboard protocol, which the shift error setting turns on, a
/// letter typed with Caps Lock arrives as if the lock were off, with the lock as
/// a flag. Caps Lock makes it a capital, and with Shift a lowercase letter,
/// except on macOS, where it stays a capital. Shortcuts are left as they are.
fn with_caps_lock(mut key: KeyEvent) -> KeyEvent {
    let KeyCode::Char(c) = key.code else {
        return key;
    };

    let caps_lock = key.state.contains(KeyEventState::CAPS_LOCK);
    if !caps_lock || key.modifiers.intersects(SHORTCUT_MODIFIERS) {
        return key;
    }

    let cased: Vec<char> = if c.is_uppercase() && !cfg!(target_os = "macos") {
        c.to_lowercase().collect()
    } else {
        c.to_uppercase().collect()
    };

    if let [single] = cased[..] {
        key.code = KeyCode::Char(single);
    }

    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEventKind;

    #[test]
    fn caps_lock_sets_the_case_of_typed_letters_only() {
        let typed = |c, state, modifiers| {
            let key = KeyEvent::new_with_kind_and_state(
                KeyCode::Char(c),
                modifiers,
                KeyEventKind::Press,
                state,
            );
            with_caps_lock(key).code
        };
        let caps = KeyEventState::CAPS_LOCK;
        let plain = KeyModifiers::NONE;
        let with_shift = if cfg!(target_os = "macos") { 'A' } else { 'a' };

        assert_eq!(typed('a', caps, plain), KeyCode::Char('A'));
        assert_eq!(typed('A', caps, plain), KeyCode::Char(with_shift));
        assert_eq!(typed('1', caps, plain), KeyCode::Char('1'));
        assert_eq!(typed('a', KeyEventState::NONE, plain), KeyCode::Char('a'));
        assert_eq!(typed('c', caps, KeyModifiers::CONTROL), KeyCode::Char('c'));
    }
}
