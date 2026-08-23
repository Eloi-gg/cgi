// TODO: this file is a mess of workarounds.

use crate::Action;
use crate::{ActionList, rendering::Output};
use std::collections::HashMap;
use std::io::Write;
use std::sync::mpsc;
use crossterm as ct;

use crate::{
    Displayable, Widget,
    layout::{ComputedWidgetPlacement, Layout, RenderedLayout},
    widget::WidgetHdl,
};

//TODO: remove pub
pub struct Application {
    pub layouts: HashMap<String, Layout>,
    pub current_layout: String,
    pub behavior: fn((u16, u16)) -> String,
    pub size: (u16, u16),
    pub rendered_layout: RenderedLayout,
    pub output: crate::rendering::LinuxOutput, // TODO : adaptative output (compiles differently depending on OS)
    pub connection_rx: mpsc::Receiver<Action>,
    pub connection_tx: mpsc::Sender<u64>,
}

/// Represents a connection to the application. Can send actions and receive messages
pub struct AppConnection {
    sender: mpsc::Sender<Action>,
    receiver: mpsc::Receiver<u64>,
}

impl AppConnection {
    pub fn send(&self, action: Action) {
        self.sender.send(action).ok();
    }

    pub fn receive(&self) -> Option<u64> {
        self.receiver.try_recv().ok()
    }
}

impl Application {
    pub fn new() -> (Self, AppConnection) {
        let (msg_channel_tx, msg_channel_rx) = mpsc::channel();
        let (action_channel_tx, action_channel_rx) = mpsc::channel();

        return (
            Application {
                layouts: HashMap::new(),
                current_layout: String::new(),
                behavior: |(_w, _h)| "No behavior set!".to_string(),
                size: (0, 0),
                rendered_layout: RenderedLayout(HashMap::new()),
                output: crate::rendering::LinuxOutput,
                connection_tx: msg_channel_tx,
                connection_rx: action_channel_rx,
            },
            AppConnection {
                sender: action_channel_tx,
                receiver: msg_channel_rx,
            },
        );
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

    fn redraw_all(&mut self) {
        for widget in self.rendered_layout.0.keys() {
            self.rendered_layout
                .render_widget_to_output(widget, &mut self.output);
        }
    }
    
    pub fn spawn_debug_window(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use crate::debug::dbg_window;
        use crate::log::*;

        dbg_window::create::spawn_server(
            get_dbg_window_exe_path().to_str().unwrap(),
            CONNECTION_IP,
            CONNECTION_PORT,
        );

        init_logger()?;

        crate::log::log("CONNECTED");
        Ok(())
    }

    pub fn run(mut self) {

        ct::terminal::enable_raw_mode().expect("Failed to enable raw mode");
        let _ = ct::execute!(std::io::stdout(), ct::terminal::EnterAlternateScreen);

        let (cols, rows) = ct::terminal::size().unwrap();

        self.output.flush();
        self.size_changed(cols, rows);
        for widget in self.rendered_layout.0.keys() {
            widget
                .widget
                .displayable
                .write()
                .unwrap()
                .on_event(crate::Event::Resize(cols, rows), &mut ActionList::new());
            self.rendered_layout
                .render_widget_to_output(widget, &mut self.output);
        }

        let mut should_redraw_all = false;

        for _ in 0..500 {
            while let Some(action) = self.connection_rx.try_recv().ok() {
                crate::log::log(&format!("CGI core: received action {:?}", action));

                match action {
                    Action::RedrawAll => {
                        should_redraw_all = true;
                    },
                    Action::ShutDown => {
                        return;
                    },
                    _ => (),
                }
            }
            if should_redraw_all {
                self.redraw_all();
                should_redraw_all = false;
            }

            if ct::event::poll(std::time::Duration::from_millis(100)).unwrap() {
                let event = ct::event::read().unwrap();
                match event {
                    ct::event::Event::Key(key_event) => {
                        if key_event.code == ct::event::KeyCode::Esc {
                            return;
                        }
                    }
                    ct::event::Event::Resize(new_cols, new_rows) => {
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
                            Action::RedrawWidget => {
                                self.rendered_layout
                                    .render_widget_to_output(widget, &mut self.output);
                            }
                            Action::MoveCursor(move_cmd) => {
                                todo!()
                            }
                            Action::RedrawAll => {
                                should_redraw_all = true;
                            },
                            Action::ShutDown => {
                                return;
                            },
                            _ => {
                                println!("Unsupported Action: {:?}", action);
                            }
                        }
                    }
                }
                if should_redraw_all {
                    self.redraw_all();
                    should_redraw_all = false;
                }
            }
            // self.update();
            // print!(".");
        }
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        let _ = ct::terminal::disable_raw_mode();
        let _ = ct::execute!(std::io::stdout(), ct::terminal::LeaveAlternateScreen);
    }
}