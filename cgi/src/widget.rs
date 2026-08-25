use crate::Displayable;
use std::{
    hash::Hash,
    sync::{Arc, Mutex, RwLock},
};

pub(crate) mod connections {
    pub(crate) const TL_CORNER_OFFSET: u8 = 0;
    pub(crate) const TR_CORNER_OFFSET: u8 = 2;
    pub(crate) const BL_CORNER_OFFSET: u8 = 4;
    pub(crate) const BR_CORNER_OFFSET: u8 = 6;

    pub(crate) const TL_CORNER: u8 = 1 << TL_CORNER_OFFSET;
    pub(crate) const TR_CORNER: u8 = 1 << TR_CORNER_OFFSET;
    pub(crate) const BL_CORNER: u8 = 1 << BL_CORNER_OFFSET;
    pub(crate) const BR_CORNER: u8 = 1 << BR_CORNER_OFFSET;

    pub(crate) const CONNECTED_LATERAL: u8 = 1 << 0;
    pub(crate) const CONNECTED_VERTICAL: u8 = 1 << 1;
}

#[derive(Debug)]
pub(crate) struct WidgetData {
    pub dirty: bool,
    pub outline: Option<crate::symbols::line::Set>,
    pub title: Option<String>,
    pub connected: u8,
}

#[derive(Debug)]
pub struct Widget<T: Displayable + ?Sized> {
    pub(crate) displayable: Arc<RwLock<T>>,
    pub(crate) data: Arc<Mutex<WidgetData>>,
}

#[derive(Debug, Hash)]
pub struct WidgetHdl { //TODO: needs pub?
    pub widget: Widget<dyn Displayable>,
}

impl WidgetHdl {
    pub fn from_widget<T: Displayable + 'static>(widget: Widget<T>) -> Self {
        widget.as_hdl()
    }

    pub(crate) fn get_data(&self) -> Option<std::sync::MutexGuard<'_, WidgetData>> {
        self.widget.data.lock().ok()
    }

    pub(crate) fn get_displayable(&self) -> Option<std::sync::RwLockReadGuard<'_, dyn Displayable>> {
        self.widget.displayable.read().ok()
    }

    pub(crate) fn write_displayable(&self) -> Option<std::sync::RwLockWriteGuard<'_, dyn Displayable + 'static>> {
        self.widget.displayable.write().ok()
    }
}

impl Clone for WidgetHdl {
    fn clone(&self) -> Self {
        WidgetHdl {
            widget: Widget {
                displayable: self.widget.displayable.clone(),
                data: self.widget.data.clone(),
            },
        }
    }
}

impl PartialEq for WidgetHdl {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.widget.displayable, &other.widget.displayable)
    }
}

impl Eq for WidgetHdl {}

impl<T: Displayable + ?Sized + 'static> Hash for Widget<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.displayable).hash(state);
    }
}

impl<T: Displayable + 'static> Widget<T> {
    pub fn new(displayable: T) -> Self {
        Widget {
            data: Arc::new(Mutex::new(WidgetData {
                dirty: true,
                outline: None,
                title: None,
                connected: 0,
            })),
            displayable: Arc::new(RwLock::new(displayable)),
        }
    }

    /// Create a builder for constructing this widget with optional configuration
    pub fn builder(displayable: T) -> WidgetBuilder<T> {
        WidgetBuilder::new(displayable)
    }

    pub fn edit(&self) -> std::sync::RwLockWriteGuard<'_, T> {
        self.displayable.write().unwrap()
    }

    pub fn repaint(&mut self) {
        self.data.lock().unwrap().dirty = true;
    }

    pub fn set_outline(&mut self, outline: crate::symbols::OutlineStyle) {
        self.data.lock().unwrap().outline = Some(outline.set().clone());
    }

    pub fn as_hdl(&self) -> WidgetHdl {
        WidgetHdl {
            widget: Widget {
                data: self.data.clone(),
                displayable: self.displayable.clone() as Arc<RwLock<dyn Displayable>>,
            },
        }
    }
}

impl std::fmt::Debug for Widget<dyn Displayable> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Widget at {:p}", Arc::as_ptr(&self.displayable))
    }
}

pub struct WidgetBuilder<T: Displayable + 'static> {
    displayable: T,
    dirty: bool,
    outline: Option<crate::symbols::line::Set>,
    title: Option<String>,
}

impl<T: Displayable + 'static> WidgetBuilder<T> {
    /// Create a new builder with the given displayable
    ///
    /// # Arguments
    /// * `displayable` - The displayable object to wrap in the widget
    pub fn new(displayable: T) -> Self {
        WidgetBuilder {
            displayable,
            dirty: true,
            outline: None,
            title: None,
        }
    }

    /// Set the outline style for the widget (default: None)
    ///
    /// The outline defines the border style for rendering.
    pub fn with_outline(mut self, outline: crate::symbols::OutlineStyle) -> Self {
        self.outline = Some(outline.set().clone());
        self
    }

    /// Build the final `Widget` instance
    pub fn build(self) -> Widget<T> {
        Widget {
            data: Arc::new(Mutex::new(WidgetData {
                dirty: self.dirty,
                outline: self.outline,
                title: self.title,
                connected: 0,
            })),
            displayable: Arc::new(RwLock::new(self.displayable)),
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }
}
