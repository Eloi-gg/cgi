mod ui;

use cgi::Coordinate::*;
use cgi::widget::WidgetBuilder;
use cgi::*;

const CONNECTION_NAME: &str = "app log";
const CONNECTION_IP: &str = "127.0.0.2";
const CONNECTION_PORT: u16 = 4000;

fn main() {
    let (mut app, app_connection) = cgi::Application::new();
    app.set_layout_behaviour(|s| ui::get_layout_id(s));
    app.add_layout(0, ui::small_layout());
    app.spawn_debug_window();
    app.run();
}
