use super::*;
use ncurses::*;

enum State {
    Idle,
    WaitingForKeyStroke,
}

pub struct KeyStrokeSelector {
    key_stroke: Option<KeyStroke>,
    state: State
}

impl KeyStrokeSelector {
    pub fn new() -> Self {
        Self {
            key_stroke: None,
            state: State::Idle,
        }
    }

    pub fn from(key_stroke: KeyStroke) -> Self {
        Self {
            key_stroke: Some(key_stroke),
            state: State::Idle,
        }
    }

    pub fn render(&self, Row {x, y, w}: Row) {
        mv(y as i32, x as i32);
        let s = match self.state {
            State::Idle => match &self.key_stroke {
                Some(key_stroke) => key_stroke.to_string(),
                None => String::from("<None>"),
            },
            State::WaitingForKeyStroke => String::from("<Input...>"),
        };
        addstr(s.as_str().get(..w).unwrap_or(s.as_str()));
    }

    pub fn handle_key(&mut self, key_stroke: &KeyStroke, key_map: &KeyMap) {
        match self.state {
            State::Idle => {
                if key_map.is_bound(key_stroke, &Action::Accept) {
                    self.state = State::WaitingForKeyStroke;
                }
            },
            State::WaitingForKeyStroke => {
                self.key_stroke = Some(key_stroke.clone());
                self.state = State::Idle
            }
        }
    }
}
