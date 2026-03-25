pub mod application;
pub mod layout;
pub mod widget;

pub use application::Application;
pub use layout::Layout;
pub use widget::Widget;

pub trait Displayable {
    fn display(&self);
    fn name(&self) -> String;
}
