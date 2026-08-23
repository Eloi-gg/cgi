pub mod application;
pub mod coordinate;
pub mod debug;
pub mod factory_widgets;
pub mod layout;
pub mod widget;
pub mod log;

mod rendering;
pub mod symbols;

#[cfg(test)]
pub mod test;

pub use application::Application;
pub use coordinate::Coordinate;
pub use layout::Layout;
pub use layout::WidgetPlacement;
pub use widget::{Widget, WidgetBuilder};

pub type KeyCode = crossterm::event::KeyCode;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Event {
    Resize(u16, u16),
    KeyPress(KeyCode), // TODO
    Custom(),       // TODO
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum EventType {
    Resize,
    KeyPress,
    Custom,
}

#[derive(Debug)]
pub struct ActionList(Vec<Action>);

impl ActionList {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    fn drain(&mut self) -> Vec<Action> {
        self.0.drain(..).collect()
    }

    pub fn add(&mut self, action: Action) {
        self.0.push(action);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CursorMove {
    Up(u16),
    Down(u16),
    Left(u16),
    Right(u16),
    
    ToAbsolute(u16, u16),
    ToRelativeToWidget(u16, u16),
}

#[derive(Debug)]
pub enum Action {
    RedrawWidget,
    RedrawAll,
    MoveCursor(CursorMove),
    SendMessage(u64),
    ShutDown,
}

impl Action {
    pub fn send_messages_from_raw_bytes(bytes: &[u8]) -> ActionList {
        let mut r = ActionList(vec![]);
        
        let it =  bytes.iter().rev(); 
        let mut num: u64 = 0;
        let mut i = 0; 
        for byte in it {
            num |= (*byte as u64) << (i * 8);
            i += 1;
            if i > 8 { 
                r.0.push(Action::SendMessage(num));
                num = 0;
                i = 0;
            }
        }
        if i > 0 {
            r.0.push(Action::SendMessage(num));
        }
        r
    }
}

impl From<Event> for EventType {
    fn from(event: Event) -> Self {
        match event {
            Event::Resize(_, _) => EventType::Resize,
            Event::KeyPress(_) => EventType::KeyPress,
            Event::Custom() => EventType::Custom,
        }
    }
}

impl From<crossterm::event::Event> for Event {
    fn from(event: crossterm::event::Event) -> Self {
        match event {
            crossterm::event::Event::Resize(x, y) => Event::Resize(x, y),
            crossterm::event::Event::Key(key_event) => if key_event.kind == crossterm::event::KeyEventKind::Press {
                Self::KeyPress(key_event.code)
            } else {
                Self::Custom()
            },
            _ => Event::Custom(),
        }
    }
}

pub trait Displayable {
    fn display(&self); // TODO delete
    fn name(&self) -> String; // TODO delete
    fn on_event(&mut self, event: Event, actions: &mut ActionList) {
        let _ = event;
        let _ = actions;
    }

    /// Returns the changed characters as a list of `(column, line, char)` tuples.
    /// The coordinates are relative to the widget. (0,0) is the top-left corner.
    ///
    /// # Arguments
    ///
    /// * `size` - The size of the widget.
    /// * `out` - The output vector to store the changed characters.
    fn get_changed_chars(&mut self, size: (u16, u16), out: &mut Vec<(u16, u16, char)>);
}
