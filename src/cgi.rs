pub mod application;
pub mod layout;
pub mod widget;
pub mod coordinate;
mod rendering;

pub use application::Application;
pub use layout::Layout;
pub use widget::Widget;
pub use coordinate::Coordinate;
pub use layout::WidgetPlacement;

pub trait Displayable {
    fn display(&self);
    fn name(&self) -> String;
    fn get_changed_chars(&self, size: (u16, u16), out: &mut Vec<(u16, u16, char)>) ;
}
