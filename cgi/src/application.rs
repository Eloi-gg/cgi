// TODO: this file is a mess of workarounds.

use std::io::Write;
use std::collections::HashMap;
use crate::rendering::Output;

use crate::{
    Displayable, Widget,
    layout::{ComputedWidgetPlacement, Layout, RenderedLayout},
    widget::WidgetHdl,
};

pub struct Application {
    pub layouts: HashMap<String, Layout>,
    pub current_layout: String,
    pub behavior: fn((u16, u16)) -> String,
    pub size: (u16, u16),
    pub rendered_layout: RenderedLayout,
    pub output: crate::rendering::LinuxOutput, // TODO : adaptative output (compiles differently depending on OS)
}

impl Application {
    pub fn new() -> Self {
        Application {
            layouts: HashMap::new(),
            current_layout: String::new(),
            behavior: |(_w, _h)| "No behavior set!".to_string(),
            size: (0, 0),
            rendered_layout: RenderedLayout(HashMap::new()),
            output: crate::rendering::LinuxOutput,
        }
    }

    pub fn add_layout(&mut self, name: &str, layout: Layout) {
        self.layouts.insert(name.to_string(), layout);
        if self.current_layout.is_empty() {
            self.current_layout = name.to_string();
        }
    }

    pub fn print_state(&self) {
        for (name, layout) in &self.layouts {
            println!("Layout: {}", name);
            for widget in &layout.layout {
                println!("  Widget: {:?}", widget.0.widget);
            }
        }
    }

    pub fn update(&mut self) {
        let mut global_changes = Vec::new();
        let mut local_changes = Vec::new();
        for widget in self.layouts[&self.current_layout].layout.keys() {
            if let Ok(mut data) = widget.widget.data.lock() {
                if (*data).dirty {
                    let placement = self.get_widget_placement(widget);
                    widget
                        .widget
                        .displayable
                        .read()
                        .expect(&format!("{:?} != {:?}", &widget.widget, self.rendered_layout.0.keys().next()))
                        .get_changed_chars((placement.width as u16, placement.height as u16), &mut local_changes);
                    (*data).dirty = false;
                    for (x, y, c) in local_changes.drain(..) {
                        global_changes.push((x + placement.x as u16, y + placement.y as u16, c));
                    }
                }
            }
        }

        for (x, y, c) in global_changes {
            self.output.place_char(x, y, c);
        }
    }

    pub fn set_layout_behaviour(&mut self, behavior: fn((u16, u16)) -> String) {
        self.behavior = behavior;
    }

    fn size_changed(&mut self, new_x: u16, new_y: u16) {
        self.current_layout = (self.behavior)((new_x, new_y));
        self.size = (new_x, new_y);
        self.rendered_layout = self.layouts[&self.current_layout].render(self.size.0 as i32, self.size.1 as i32);
    }

    fn get_widget_placement(
        &self,
        widget_hdl: &WidgetHdl,
    ) -> ComputedWidgetPlacement {
        self.rendered_layout
            .0
            .get(&widget_hdl)
            .expect(&format!("{:?} != {:?}", &widget_hdl, self.rendered_layout.0.keys().next()))
            .clone()
    }

    pub fn run(mut self) {
        use crossterm::{
            event::{self, Event, KeyCode},
            terminal::size,
        };

        let (cols, rows) = size().unwrap();
        println!("Terminal size: {} cols, {} rows", cols, rows);

        self.output.flush();
        self.size_changed(cols, rows);

        for _ in 0..500 {
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
            // print!(".");
            std::io::stdout().flush().unwrap();
        }
    }
}
