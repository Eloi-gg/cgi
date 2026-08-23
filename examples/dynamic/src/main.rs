use cgi::Coordinate::*;
use cgi::widget::WidgetBuilder;
use cgi::*;

const CONNECTION_NAME: &str = "app log";
const CONNECTION_IP: &str = "127.0.0.2";
const CONNECTION_PORT: u16 = 4000;

fn main() {
    use cgi::factory_widgets::{Listener, progression::*, text::*};

    cgi::debug::dbg_window::create::spawn_server(
        cgi::log::get_dbg_window_exe_path().to_str().unwrap(),
        CONNECTION_IP,
        CONNECTION_PORT,
    );
    let mut logger = cgi::debug::dbg_window::connect::connect_to_server(
        CONNECTION_NAME,
        CONNECTION_IP,
        CONNECTION_PORT,
        Some(std::time::Duration::from_secs(5)),
    ).unwrap();
    logger.send_message("APP CONNECTED");
    let (mut app, app_connection) = cgi::Application::new();

    let tb = WidgetBuilder::new(TextBox::default())
        .with_outline(symbols::OutlineStyle::Double)
        .build();

    let placement = WidgetPlacement::fullscreen();
    let layout = Layout::new().with_widget(&tb, placement);

    app.set_layout_behaviour(|(..)| "MainLayout".to_string());
    app.add_layout("MainLayout", layout);
    app.spawn_debug_window().unwrap();

    std::thread::spawn(move || {
        for i in 0..10 {
            logger.send_message(&format!("{}", i));
            tb.edit().set_text(&i.to_string()); //&format!("{}", i));
            app_connection.send(cgi::Action::RedrawAll);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        app_connection.send(cgi::Action::ShutDown);
        return;
    });

    app.run();
}
