use std::sync::{Arc, Mutex, RwLock};
use crate::cgi::Displayable;

pub struct Widget<T: Displayable + ?Sized> {
    pub displayable: Arc<RwLock<T>>,
    pub dirty: Arc<Mutex<bool>>,
}

#[derive(Debug)]
pub struct WidgetHdl {
    pub widget: Widget<dyn Displayable>,
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
