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
        .with_outline(symbols::OutlineStyle::Normal)
        .build();
    let empty = WidgetBuilder::new(cgi::factory_widgets::utils::Empty)
        .with_outline(symbols::OutlineStyle::Rounded)
        .with_title("Empty")
        .build();

    let fs = WidgetPlacement::fullscreen();
    let mut horizontal_placement = [WidgetPlacement::default(); 2];
    let mut vertical_placement = [WidgetPlacement::default(); 2];

    fs.split(2, 1, true, &mut horizontal_placement);
    fs.split(1, 2, true, &mut vertical_placement);

    let h_layout = Layout::new()
        .with_widget(&tb, horizontal_placement[0])
        .with_widget(&empty, horizontal_placement[1]);
    let v_layout = Layout::new()
        .with_widget(&tb, vertical_placement[0])
        .with_widget(&empty, vertical_placement[1]);

    app.set_layout_behaviour(|(x, y)| (x > 2 * y) as u8);
    app.add_layout(0, v_layout);
    app.add_layout(1, h_layout);
    app.spawn_debug_window().unwrap();

    std::thread::spawn(move || {
        app_connection.send_command(Command::FocusWidget(tb.as_hdl()));
        for i in 0..50 {
            logger.send_message(&format!("{}", i));
            tb.edit().set_text(&i.to_string()); //&format!("{}", i));
            app_connection.send_command(cgi::Command::FocusWidget(tb.as_hdl()));
            app_connection.send_action(cgi::Action::RedrawAll);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        app_connection.send_command(cgi::Command::ShutDown);
        return;
    });

    app.run();
}
