use std::collections::HashSet;

use crate::Displayable;

pub struct Listener<T: ?Sized> {
    events: HashSet<crate::Event>,
    on_event: fn(crate::Event, &mut T),
}

impl<T: ?Sized> Listener<T> {
    pub fn empty() -> Self {
        Self::new(|_, _| {})
    }

    pub fn new(on_event: fn(crate::Event, &mut T)) -> Self {
        Self {
            events: HashSet::new(),
            on_event,
        }
    }

    pub fn listen_for(&mut self, event: crate::Event) {
        self.events.insert(event);
    }

    pub fn listening_for(self, event: crate::Event) -> Self {
        let mut listener = self;
        listener.listen_for(event);
        listener
    }

    pub fn is_listening_for(&self, event: crate::Event) -> bool {
        self.events.contains(&event)
    }
}

pub mod progression {}

pub mod text {
    use super::*;

    pub struct TextBox {
        text: Vec<char>,
        changed_chars: Vec<usize>, // points to chars in the text
        size: (u16, u16), // Remove ?
        listener: Listener<Self>,
    }

    impl TextBox {
        pub fn new(text: &str, listener: Listener<Self>) -> Self {
            Self {
                text: text.chars().collect(),
                changed_chars: (0..text.len()).collect(),
                size: (0, 0),
                listener,
            }
        }

        pub fn set_text(&mut self, text: &str) {
            for (i, c) in text.chars().enumerate() {
                if self.text[i] != c {
                    self.text[i] = c;
                    self.changed_chars.push(i);
                }
            }
        }
    }

    impl Displayable for TextBox {
        fn display(&self) {
            todo!()
        }

        fn name(&self) -> String {
            todo!()
        }

        fn get_changed_chars(&self, size: (u16, u16), out: &mut Vec<(u16, u16, char)>) {
            if size.0 * size.1 == 0 {
                return;
            }
            for i in self.changed_chars.iter() {
                let line = *i as u16 / size.0;
                let column = *i as u16 % size.0;
                if line > size.1 {
                    break;
                }
                out.push((column, line, self.text[*i]));
            }
        }

        fn on_event(&mut self, event: crate::Event) {
            if self.listener.is_listening_for(event) {
                (self.listener.on_event)(event, self);
            }
        }
    }
}
