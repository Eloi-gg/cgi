use std::{hash::Hash, sync::{Arc, Mutex, RwLock}};
use crate::cgi::Displayable;

#[derive(Debug)]
pub struct Widget<T: Displayable + ?Sized> {
    pub displayable: Arc<RwLock<T>>,
    pub dirty: Arc<Mutex<bool>>,
}

#[derive(Debug, Hash)]
pub struct WidgetHdl {
    pub widget: Widget<dyn Displayable>,
}

impl Clone for WidgetHdl {
    fn clone(&self) -> Self {
        WidgetHdl {
            widget: Widget {
                displayable: self.widget.displayable.clone(),
                dirty: self.widget.dirty.clone(),
            },
        }
    }
}

impl PartialEq for WidgetHdl {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.widget.displayable, &other.widget.displayable)
    }
}

impl Eq for WidgetHdl {
}

impl<T: Displayable + ?Sized + 'static> Hash for Widget<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.displayable).hash(state);
    }
}

impl<T: Displayable + 'static> Widget<T> {
    pub fn new(displayable: T) -> Self {
        Widget {
            dirty: Arc::new(Mutex::new(true)),
            displayable: Arc::new(RwLock::new(displayable)),
        }
    }

    pub fn edit(&self) -> std::sync::RwLockWriteGuard<'_, T> {
        self.displayable.write().unwrap()
    }

    pub fn repaint(&mut self) {
        *self.dirty.lock().unwrap() = true;
    }

    pub fn as_dyn(&self) -> Widget<dyn Displayable> {
        Widget {
            dirty: self.dirty.clone(),
            displayable: self.displayable.clone() as Arc<RwLock<dyn Displayable>>,
        }
    }
}

impl std::fmt::Debug for Widget<dyn Displayable> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Widget {{ name: {} }}",
            self.displayable.read().unwrap().name()
        )
    }
}
