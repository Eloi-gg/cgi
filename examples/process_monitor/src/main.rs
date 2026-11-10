use cgi::factory_widgets::text::TextAlign;
use cgi::*;

fn table() -> cgi::factory_widgets::table::StandardTable {
    use crate::factory_widgets::table::*;

    let mut table = TableBuilder::new_standard()
        .with_column(
            StandardColumnType::new_text(TextAlign::Center),
            ColumnWidth::Fixed(5),
            "PID",
        )
        .with_column(
            StandardColumnType::new_number(TextAlign::Center),
            ColumnWidth::Adaptative {
                min: 1,
                max: 10,
                offset: 2,
            },
            "",
        )
        .with_column(
            StandardColumnType::new_text(TextAlign::Center),
            ColumnWidth::Fixed(12),
            "Status",
        )
        .with_column(
            StandardColumnType::new_progress_bar(
                factory_widgets::progression::ProgressBarType::HorizontalNineLevels,
            ),
            ColumnWidth::Growable(5, 50),
            "Progress",
        )
        .with_n_rows(4)
        .build();

    set_table_row!(table, 0, ["105", -4, "Running", 0.5]);
    set_table_row!(table, 1, ["95", 0, "Paused", 0.15]);
    set_table_row!(table, 2, ["1", 20, "Waiting", 1.0]);
    set_table_row!(table, 3, ["104", -1, "Running", 0.0]);

    table.set_cell(1, 0, -20);

    table
}

fn main() {
    use cgi::factory_widgets::{Listener, progression::*, text::*};

    let (mut app, _) = cgi::Application::new();
    let table = table();
    let mut layout = cgi::Layout::new().with_widget(
        &table.widget_builder().build(),
        WidgetPlacement::fullscreen(),
    );

    app.set_layout_behaviour(|(..)| 0);
    app.add_layout(0, layout);
    app.spawn_debug_window();
    app.run();
}
