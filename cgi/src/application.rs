// TODO: this file is a mess of workarounds.

use crate::{ActionList, rendering::Output};
use std::collections::HashMap;
use std::io::Write;

use crate::{
    layout::{ComputedWidgetPlacement, Layout, RenderedLayout},
    widget::WidgetHdl,
    Displayable, Widget,
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
        // println!("update");
        self.rendered_layout.render_to_output(&mut self.output);
    }

    pub fn set_layout_behaviour(&mut self, behavior: fn((u16, u16)) -> String) {
        self.behavior = behavior;
    }

    fn size_changed(&mut self, new_x: u16, new_y: u16) {
        self.current_layout = (self.behavior)((new_x, new_y));
        self.size = (new_x, new_y);
        self.rendered_layout =
            self.layouts[&self.current_layout].render(self.size.0 as i32, self.size.1 as i32);
        self.output.flush();
        self.update();
    }

    pub fn run(mut self) {
        use crossterm::{
            event::{poll, read, Event, KeyCode},
            terminal::{enable_raw_mode, size},
        };

        enable_raw_mode().expect("Failed to enable raw mode");

        let (cols, rows) = size().unwrap();
        println!("Terminal size: {} cols, {} rows", cols, rows);

        self.output.flush();
        self.size_changed(cols, rows);

        for _ in 0..500 {
            if poll(std::time::Duration::from_millis(100)).unwrap() {
                let event = read().unwrap();
                match event {
                    Event::Key(key_event) => {
                        if key_event.code == KeyCode::Esc {
                            break;
                        }
                    }
                    Event::Resize(new_cols, new_rows) => {
                        self.size_changed(new_cols, new_rows);
                        // println!("Resized to: {} cols, {} rows", new_cols, new_rows);
                    }
                    _ => {}
                }
                let mut actions_list = ActionList::new();
                for widget in self.rendered_layout.0.keys() {
                    widget
                        .widget
                        .displayable
                        .write()
                        .unwrap()
                        .on_event(event.clone().into(), &mut actions_list);

                    // TODO: apply actions

                    for action in actions_list.drain() {
                        match action {
                            crate::Action::UpdateWidget => {
                                self.rendered_layout
                                    .render_widget_to_output(widget, &mut self.output);
                            }
                            _ => {
                                println!("Action: {:?}", action);
                            }
                        }
                    }
                }
            }
            // self.update();
            // print!(".");
            std::io::stdout().flush().unwrap();
        }
    }
}
