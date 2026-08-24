use std::cell::Cell;
use std::rc::Rc;

use cgi::Coordinate::*;
use cgi::debug::dbg_window;
use cgi::debug::dbg_window::connect::DebugConsole;
use cgi::widget::WidgetBuilder;
use cgi::*;
use once_cell::sync::Lazy;
use std::sync::Mutex;

static CONSOLE: Lazy<Mutex<DebugConsole>> = Lazy::new(|| {
    let dbg_window_path = std::env::current_dir()
        .unwrap()
        .join("target/debug/dbg_window");
    dbg_window::create::spawn_server(dbg_window_path.to_str().unwrap(), "127.0.0.2", 4000);

    let console = Mutex::new(
        dbg_window::connect::connect_to_server(
            "MainApp",
            "127.0.0.2",
            4000,
            Some(std::time::Duration::from_secs(5)),
        )
        .unwrap(),
    );

    console
});

fn console_println(string: &str) {
    let mut console = CONSOLE.lock().unwrap();
    console.send_message(string);
}

fn on_event(
    event: cgi::Event,
    actions: &mut cgi::ActionList,
    text_box: &mut cgi::factory_widgets::text::TextBox,
) {
    if let Event::KeyPress(kc) = event {
        console_println(&format!(
            "Key pressed event {:?} | text: {} | len {}",
            kc,
            text_box.text(),
            text_box.text_len()
        ));
        match kc {
            KeyCode::Char(c) => {
                text_box.append_char(c);
            }
            KeyCode::Enter => {
                text_box.append_char('\n');
            }
            KeyCode::Backspace => {
                text_box.remove_text(text_box.text_len() - 1, text_box.text_len());
            }
            _ => {}
        }
        actions.add(crate::Action::RedrawWidget);
        actions.add(crate::Action::MoveCursor(CursorMove::ToRelativeToWidget(
            text_box.text_len() as u16,
            0,
        )));
    } else {
        console_println(&format!("Event {:?}", event));
    }
}

fn build_app() -> cgi::Application {
    use cgi::factory_widgets::{Listener, progression::*, text::*};

    let (mut app, _) = cgi::Application::new();

    let title = WidgetBuilder::new(TextBox::new(
        "Text editor",
        Listener::empty(),
        factory_widgets::text::TextAlign::Center,
    ))
    .with_outline(symbols::OutlineStyle::Double)
    .build();

    let main_panel = WidgetBuilder::new(TextBox::new(
        "Type your text here...",
        Listener::new(on_event).listening_for(EventType::KeyPress),
        TextAlign::Left,
    ))
    .with_outline(symbols::OutlineStyle::Rounded)
    .build();
    let progress_bar = WidgetBuilder::new(ProgressBar::new(
        ProgressBarType::HorizontalNineLevels,
        0.0,
        Listener::empty(),
    ))
    .with_outline(symbols::OutlineStyle::Normal)
    .with_title("Progress")
    .build();

    let title_placement = WidgetPlacement::fullscreen()
        .with_height(3)
        .expand_or_shrink(-1, 0);
    let main_panel_placement = title_placement.get_below().shift_bottom_right(0, -3);
    let progress_bar_placement =
        WidgetPlacement::new(Absolute(0), Hybrid(-3, 1.0), 1.0.into(), 1.0.into());

    let layout = cgi::Layout::new()
        .with_widget(&title, title_placement)
        .with_widget(&main_panel, main_panel_placement)
        .with_widget(
            &progress_bar,
            progress_bar_placement.expand_or_shrink(-1, 0),
        );

    app.set_layout_behaviour(|(..)| "MainLayout".to_string());
    app.add_layout("MainLayout", layout);
    let _ = app.spawn_debug_window();

    app
}

fn build_app_w_text_input() -> cgi::Application {
    use cgi::factory_widgets::{Listener, progression::*, text::*};

    let (mut app, _) = cgi::Application::new();

    let title = WidgetBuilder::new(TextBox::new(
        "Text editor",
        Listener::empty(),
        factory_widgets::text::TextAlign::Center,
    ))
    .with_outline(symbols::OutlineStyle::Double)
    .build();

    let main_panel = WidgetBuilder::new(TextInput::default())
    .with_outline(symbols::OutlineStyle::Rounded)
    .build();
    let progress_bar = WidgetBuilder::new(ProgressBar::new(
        ProgressBarType::HorizontalNineLevels,
        0.0,
        Listener::empty(),
    ))
    .with_outline(symbols::OutlineStyle::Normal)
    .with_title("Progress")
    .build();

    let title_placement = WidgetPlacement::fullscreen()
        .with_height(3)
        .expand_or_shrink(-1, 0);
    let main_panel_placement = title_placement.get_below().shift_bottom_right(0, -3);
    let progress_bar_placement =
        WidgetPlacement::new(Absolute(0), Hybrid(-3, 1.0), 1.0.into(), 1.0.into());

    let layout = cgi::Layout::new()
        .with_widget(&title, title_placement)
        .with_widget(&main_panel, main_panel_placement)
        .with_widget(
            &progress_bar,
            progress_bar_placement.expand_or_shrink(-1, 0),
        );

    app.set_layout_behaviour(|(..)| "MainLayout".to_string());
    app.add_layout("MainLayout", layout);
    let _ = app.spawn_debug_window();

    app
}

fn main() {
    console_println("Init");
    let app = build_app_w_text_input();
    app.run();
}
