use std::collections::HashMap;
use crate::cgi::layout::LayoutBuilder;

pub struct Application {
    pub layouts: HashMap<String, LayoutBuilder>,
    pub current_layout: String,
    pub behavior: fn((u16, u16)) -> String,
}

impl Application {
    pub fn new() -> Self {
        Application {
            layouts: HashMap::new(),
            current_layout: String::new(),
            behavior: |(_w, _h)| "No behavior set!".to_string(),
        }
    }

    pub fn add_layout(&mut self, name: &str, layout: LayoutBuilder) {
        self.layouts.insert(name.to_string(), layout);
        if self.current_layout.is_empty() {
            self.current_layout = name.to_string();
        }
    }

    pub fn print_state(&self) {
        for (name, layout) in &self.layouts {
            println!("Layout: {}", name);
            for widget in &layout.widgets {
                println!("  Widget: {:?}", widget.widget);
            }
        }
    }

    pub fn update(&self) {
        for widget in self.layouts[&self.current_layout].widgets.iter() {
            if let Ok(mut dirty) = widget.widget.dirty.lock() {
                if *dirty {
                    widget.widget.displayable.read().unwrap().display();
                    *dirty = false;
                }
            }
        }
    }

    pub fn set_layout_behaviour(&mut self, behavior: fn((u16, u16)) -> String) {
        self.behavior = behavior;
    }

    fn size_changed(&mut self, new_x: u16, new_y: u16) {
        self.current_layout = (self.behavior)((new_x, new_y));
    }

    pub fn run(mut self) {
        use crossterm::{
            event::{self, Event, KeyCode},
            terminal::{self, size},
        };

        let (cols, rows) = size().unwrap();
        println!("Terminal size: {} cols, {} rows", cols, rows);

        for _ in 0..50 {
            if event::poll(std::time::Duration::from_millis(100)).unwrap() {
                match event::read().unwrap() {
                    Event::Key(key_event) => {
                        if key_event.code == KeyCode::Esc {
                            break;
                        }
                        println!("Key pressed: {:?}", key_event);
                    }
                    Event::Resize(new_cols, new_rows) => {
                        self.size_changed(new_cols, new_rows);
                        println!("Resized to: {} cols, {} rows", new_cols, new_rows);
                    }
                    _ => {}
                }
            }
            self.update();
        }
    }
}
