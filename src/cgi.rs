pub mod application;
pub mod layout;
pub mod widget;
pub mod coordinate;

pub use application::Application;
pub use layout::Layout;
pub use widget::Widget;
pub use coordinate::Coordinate;

pub trait Displayable {
    fn display(&self);
    fn name(&self) -> String;
}
