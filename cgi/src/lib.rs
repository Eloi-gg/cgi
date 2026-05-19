pub mod application;
pub mod layout;
pub mod widget;
pub mod coordinate;
pub mod factory_widgets;

mod rendering;
pub mod symbols;

#[cfg(test)]
pub mod test;

pub use application::Application;
pub use layout::Layout;
pub use widget::{Widget, WidgetBuilder};
pub use coordinate::Coordinate;
pub use layout::WidgetPlacement;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Event {
    Resize(u16, u16),
    KeyPress(char), // TODO
    Custom(), // TODO
}

impl From<crossterm::event::Event> for Event {
    fn from(event: crossterm::event::Event) -> Self {
        match event {
            crossterm::event::Event::Resize(x, y) => Event::Resize(x, y),
            _ => Self::Custom(),
        }
    }
}

pub trait Displayable {
    fn display(&self); // TODO delete
    fn name(&self) -> String; // TODO delete
    fn on_event(&mut self, event: Event) { let _ = event; }
    fn get_changed_chars(&mut self, size: (u16, u16), out: &mut Vec<(u16, u16, char)>) ;
}

#[cfg(test)]
mod tests {
    use super::*;
}
