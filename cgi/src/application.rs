// TODO: this file is a mess of workarounds.

use crate::{Action, AppMessage, Command};
use crate::{ActionList, rendering::Output};
use crossterm as ct;
use crossterm::terminal::ClearType::FromCursorUp;
use std::collections::HashMap;
use std::io::Write;
use std::sync::mpsc;

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
    pub os: crate::rendering::OS,
    pub connection_rx: mpsc::Receiver<AppMessage>,
    pub connection_tx: mpsc::Sender<u64>,
    pub global_action: GlobalAction,
    pub selected_widget: Option<WidgetHdl>,
}

/// Represents a connection to the application. Can send actions and receive messages
pub struct AppConnection {
    sender: mpsc::Sender<AppMessage>,
    receiver: mpsc::Receiver<u64>,
}

impl AppConnection {
    pub fn send_action(&self, action: Action) {
        self.sender.send(AppMessage::Action(action));
    }

    pub fn send_command(&self, command: Command) {
        self.sender.send(AppMessage::Command(command));
    }

    pub fn receive(&self) -> Option<u64> {
        self.receiver.try_recv().ok()
    }
}

struct GlobalAction {
    redraw_all: bool,
    cursor_move: Option<crate::CursorMove>,
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
                os: crate::rendering::OS::get(), // TODO: known at compilation
                connection_tx: msg_channel_tx,
                connection_rx: action_channel_rx,
                global_action: GlobalAction {
                    redraw_all: false,
                    cursor_move: None,
                },
                selected_widget: None,
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

    /// Moves the cursor. Does NOT support [`CursorMove::ToRelativeToWidget`] (should be converted before this is called)
    fn move_cursor(&self, cursor_move: crate::CursorMove) {
        use ct::cursor as ctc;

        use crate::CursorMove as CM;
        let mut output = std::io::stdout();

        let _ = match cursor_move {
            CM::Up(x) => ct::execute!(output, ctc::MoveUp(x)),
            CM::Down(x) => ct::execute!(output, ctc::MoveDown(x)),
            CM::Left(x) => ct::execute!(output, ctc::MoveLeft(x)),
            CM::Right(x) => ct::execute!(output, ctc::MoveRight(x)),
            CM::ToAbsolute(x, y) => ct::execute!(output, ctc::MoveTo(x, y)),
            CM::ToRelativeToWidget(..) => {
                panic!("ToRelativeToWidget should be converted before this is called")
            }
        };
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

    fn handle_widget_actions(&mut self, action: Action) {
        let widget = if let Some(widget) = &self.selected_widget {
            widget
        } else {
            return;
        };
        let placement = self.rendered_layout.get_widget_coords(widget, true);
        match action {
            Action::RedrawWidget => {
                self.rendered_layout
                    .render_widget_to_output(widget, &placement, &mut self.output);
            }
            Action::MoveCursor(cursor_move) => {
                let mv_cmd = if let crate::CursorMove::ToRelativeToWidget(x, y) = cursor_move {
                    let (x, y) = (x + placement.x as u16, y + placement.y as u16);
                    crate::CursorMove::ToAbsolute(x, y)
                } else {
                    cursor_move
                };
                self.global_action.cursor_move = Some(mv_cmd);
            }
            Action::RedrawAll => {
                self.global_action.redraw_all = true;
            }
            Action::ShutDown => {
                return;
            }
            _ => {
                println!("Unsupported Action: {:?}", action);
            }
        }
    }

    fn handle_global_action(&mut self) {
        if self.global_action.redraw_all == true {
            self.rendered_layout.render_to_output(&mut self.output);
            self.global_action.redraw_all = false;
        }
        if let Some(cursor_move) = self.global_action.cursor_move {
            self.move_cursor(cursor_move);
            self.global_action.cursor_move = None;
            crate::log::log(&format!("CURSOR MOVE {:?}", cursor_move));
        }
    }

    fn handle_received_messages(&mut self) {
        let mut should_redraw_all = false;

        while let Some(msg) = self.connection_rx.try_recv().ok() {
            crate::log::log(&format!("CGI core: received message {:?}", msg));
            match msg {
                AppMessage::Command(command) => match command {
                    Command::FocusWidget(widget_hdl) => self.selected_widget = Some(widget_hdl),
                },
                AppMessage::Action(action) => match action {
                    Action::RedrawAll => {
                        should_redraw_all = true;
                    }
                    Action::ShutDown => {
                        return;
                    }
                    _ => (),
                },
            }
        }
        if should_redraw_all {
            self.rendered_layout.render_to_output(&mut self.output);
        }
    }

    pub fn run(mut self) {
        // Initial setup
        ct::terminal::enable_raw_mode().expect("Failed to enable raw mode");
        let _ = ct::execute!(std::io::stdout(), ct::terminal::EnterAlternateScreen);
        let (cols, rows) = ct::terminal::size().unwrap();

        // Initial resize
        self.size_changed(cols, rows);
        for widget in self.rendered_layout.0.keys() {
            widget
                .write_displayable()
                .unwrap()
                .on_event(crate::Event::Resize(cols, rows), &mut ActionList::new());
            self.selected_widget = Some(widget.clone());
        }

        // Event loop
        for _ in 0..500 {
            self.handle_received_messages();

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
                crate::log::log(&format!("CGI core: handling event {:?}", event));
                for widget in self.rendered_layout.0.keys() {
                    widget
                        .write_displayable()
                        .unwrap()
                        .on_event(event.clone().into(), &mut actions_list);
                }
                for action in actions_list.drain() {
                    self.handle_widget_actions(action);
                }

                self.handle_global_action();
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
