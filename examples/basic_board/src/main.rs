use cgi::Coordinate::*;
use cgi::widget::WidgetBuilder;
use cgi::*;

fn main() {
    use cgi::factory_widgets::{Listener, progression::*, text::*};

    let (mut app, _) = cgi::Application::new();

    let title = WidgetBuilder::new(TextBox::new(
        "Title",
        Listener::empty(),
        factory_widgets::text::TextAlign::Center,
    ))
    .with_outline(symbols::OutlineStyle::Double)
    .build();

    let panel_left = WidgetBuilder::new(TextBox::new(
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
        Listener::empty(),
        TextAlign::Left,
    ))
    .with_outline(symbols::OutlineStyle::Rounded)
    .with_title("Left Panel")
    .build();
    let panel_right = WidgetBuilder::new(TextBox::new(
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
        Listener::empty(),
        TextAlign::Left,
    ))
    .with_outline(symbols::OutlineStyle::Rounded)
    .with_title("Right Panel")
    .build();

    let progress_bar = WidgetBuilder::new(ProgressBar::new(
        ProgressBarType::HorizontalNineLevels,
        0.565,
        Listener::empty(),
    ))
    .with_outline(symbols::OutlineStyle::Normal)
    .with_title("Progress")
    .build();

    let title_placement = WidgetPlacement::fullscreen().with_height(0.3);
    let mut panels_below_placement = [WidgetPlacement::fullscreen(); 2];

    title_placement
        .shift(0.0, 0.3)
        .with_height(Hybrid(-3, 0.7))
        .split(2, 1, false, &mut panels_below_placement);
    let progress_bar_placement =
        WidgetPlacement::new(Absolute(0), Hybrid(-3, 1.0), 1.0.into(), 1.0.into());

    let mut layout = cgi::Layout::new()
        .with_widget(&title, title_placement.expand_or_shrink(-1, -1))
        .with_widget(
            &panel_left,
            panels_below_placement[0].expand_or_shrink(-1, -1),
        )
        .with_widget(
            &panel_right,
            panels_below_placement[1].expand_or_shrink(-1, -1),
        )
        .with_widget(
            &progress_bar,
            progress_bar_placement.expand_or_shrink(-1, 0),
        );

    app.set_layout_behaviour(|(..)| "MainLayout".to_string());
    app.add_layout("MainLayout", layout);

    app.run();
}
