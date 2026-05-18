pub mod application;
pub mod layout;
pub mod widget;
pub mod coordinate;
pub mod factory_widgets;

mod rendering;
pub mod symbols;

pub use application::Application;
pub use layout::Layout;
pub use widget::Widget;
pub use coordinate::Coordinate;
pub use layout::WidgetPlacement;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Event {
    Resize(u16, u16),
    KeyPress(char), // TODO
    Custom(), // TODO
}

pub trait Displayable {
    fn display(&self); // TODO delete
    fn name(&self) -> String; // TODO delete
    fn on_event(&mut self, event: Event) { let _ = event; }
    fn get_changed_chars(&self, size: (u16, u16), out: &mut Vec<(u16, u16, char)>) ;
}

#[cfg(test)]
mod tests {
    use std::fmt::format;

    use super::*;

    struct CustomWidget {
        data: String,
    }

    impl Displayable for CustomWidget {
        fn display(&self) {
            println!("Displaying CustomWidget with data: {}", self.data);
        }

        fn name(&self) -> String {
            format!("CustomWidget {}", self.data)
        }

        fn get_changed_chars(&self, size: (u16, u16), out: &mut Vec<(u16, u16, char)>)  {
            todo!()
        }

        fn on_event(&mut self, event: Event) {
            todo!()
        }
    }
}
