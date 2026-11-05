pub mod application;
pub mod coordinate;
pub mod debug;
pub mod factory_widgets;
pub mod layout;
pub mod log;
pub mod text_formatting;
pub mod widget;

mod rendering;
pub mod symbols;

#[cfg(test)]
pub mod test;

pub use application::Application;
pub use coordinate::Coordinate;
pub use crossterm::event::KeyCode;
pub use layout::Layout;
pub use layout::WidgetPlacement;
pub use rendering::CharContainer;
pub use widget::{Widget, WidgetBuilder};

use crate::widget::WidgetHdl;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Event {
    Resize(u16, u16),
    KeyPress(KeyCode),
    WidgetFocused,
    Custom(), // TODO
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum EventType {
    Resize,
    KeyPress,
    WidgetFocused,
    Custom,
}

#[derive(Default)]
pub struct ActionList {
    pub(crate) redraw_list: Vec<WidgetHdl>,
    pub(crate) custom_messages: Vec<u64>,
    pub(crate) visibility_change: Vec<(WidgetHdl, bool)>,
    pub(crate) redraw_all: bool,
    pub(crate) cursor_move: Option<(CursorMove, WidgetHdl)>,
    pub(crate) focused_widget: Option<WidgetHdl>,
    pub(crate) cursor_show: Option<bool>,
    pub(crate) cursor_style: Option<CursorStyle>,
}

impl ActionList {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn set_focused_widget(&mut self, widget: WidgetHdl) {
        self.focused_widget = Some(widget)
    }

    pub fn add(&mut self, action: Action) {
        let Some(focused_widget) = &self.focused_widget else {
            return;
        };
        match action {
            Action::RedrawWidget => self.redraw_list.push(focused_widget.clone()),
            Action::RedrawAll => self.redraw_all = true,
            Action::SetVisible(b) => self.visibility_change.push((focused_widget.clone(), b)),
            Action::MoveCursor(cursor_move) => {
                self.cursor_move = Some((cursor_move, focused_widget.clone()))
            }
            Action::CustomMessage(msg) => self.custom_messages.push(msg),
            Action::SetCursorStyle(set_cursor_style) => self.cursor_style = Some(set_cursor_style),
            Action::ShowCursor => self.cursor_show = Some(true),
            Action::HideCursor => self.cursor_show = Some(false),
        }
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

pub use crossterm::cursor::SetCursorStyle as CursorStyle;

pub enum Command {
    FocusWidget(widget::WidgetHdl),
    ShutDown,
}

#[derive(Debug)]
pub enum AppMessage {
    Command(Command),
    Action(Action),
}

pub enum Action {
    RedrawWidget,
    RedrawAll,
    SetVisible(bool),
    CustomMessage(u64),
    MoveCursor(CursorMove),
    SetCursorStyle(CursorStyle),
    ShowCursor,
    HideCursor,
}

use std::fmt::Display;
impl std::fmt::Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RedrawWidget => write!(f, "RedrawWidget"),
            Self::RedrawAll => write!(f, "RedrawAll"),
            Self::SetVisible(arg0) => f.debug_tuple("SetVisible").field(arg0).finish(),
            Self::CustomMessage(arg0) => f.debug_tuple("CustomMessage").field(arg0).finish(),
            Self::MoveCursor(arg0) => f.debug_tuple("MoveCursor").field(arg0).finish(),
            Self::SetCursorStyle(arg0) => arg0.fmt(f),
            Self::ShowCursor => write!(f, "ShowCursor"),
            Self::HideCursor => write!(f, "HideCursor"),
        }
    }
}

impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FocusWidget(arg0) => f.write_fmt(format_args!(
                "FocusWidget {:p}",
                std::sync::Arc::as_ptr(&arg0.widget.data)
            )),
            Self::ShutDown => f.write_str("ShutDown"),
        }
    }
}

impl From<Event> for EventType {
    fn from(event: Event) -> Self {
        match event {
            Event::Resize(_, _) => EventType::Resize,
            Event::KeyPress(_) => EventType::KeyPress,
            Event::Custom() => EventType::Custom,
            Event::WidgetFocused => todo!(),
        }
    }
}

impl From<crossterm::event::Event> for Event {
    fn from(event: crossterm::event::Event) -> Self {
        match event {
            crossterm::event::Event::Resize(x, y) => Event::Resize(x, y),
            crossterm::event::Event::Key(key_event) => {
                if key_event.kind == crossterm::event::KeyEventKind::Press {
                    Self::KeyPress(key_event.code)
                } else {
                    Self::Custom()
                }
            }
            _ => Event::Custom(),
        }
    }
}

pub trait Displayable: Send + Sync {
    fn on_event(&mut self, event: Event, actions: &mut ActionList) {
        let _ = event;
        let _ = actions;
    }

    /// Returns all visible characters as `(column, line, char)` tuples.
    /// The coordinates are relative to the widget. (0,0) is the top-left corner.
    ///
    /// Implementations may return either a borrowed slice or an owned Vec wrapped in a Cow.
    fn get_chars(&mut self) -> crate::CharContainer<'_>;

    fn widget_builder(self) -> crate::WidgetBuilder<Self>
    where
        Self: Sized,
    {
        WidgetBuilder::new(self)
    }
}
