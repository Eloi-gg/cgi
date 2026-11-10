
pub mod progression;
pub mod text;
pub mod table;

use crate::EventType;

#[derive(Clone)]
pub struct Listener<T: ?Sized> {
    events: std::collections::HashSet<EventType>,
    on_event: fn(crate::Event, &mut crate::ActionList, &mut T),
}

impl<T: ?Sized> Listener<T> {
    pub fn empty() -> Self {
        Self::new(|_, _, _| {})
    }

    pub fn new(on_event: fn(crate::Event, &mut crate::ActionList, &mut T)) -> Self {
        Self {
            events: std::collections::HashSet::new(),
            on_event,
        }
    }

    pub fn listen_for(&mut self, event: crate::EventType) {
        self.events.insert(event.into());
    }

    pub fn listening_for(self, event: crate::EventType) -> Self {
        let mut listener = self;
        listener.listen_for(event);
        listener
    }

    pub fn is_listening_for(&self, event: crate::EventType) -> bool {
        self.events.contains(&event.into())
    }
}

pub mod utils {
    use crate::{CharContainer, Displayable};

    pub struct Empty;

    impl Displayable for Empty {
        fn get_chars(&mut self) -> crate::CharContainer<'_> {
            CharContainer::new_empty()
        }
    }
}

#[cfg(test)]
mod factory_widgets_tests {
    use super::text::*;
    use crate::{factory_widgets::Listener, test::*, *};

    #[test]
    fn adding_and_removing_text() {
        let mut text_box = TextBox::new("123", Listener::empty(), TextAlign::Left)
            .with_wrapping_mode(Wrapping::PerLetter);

        text_box.on_event(Event::Resize(16, 1), &mut ActionList::new());

        text_box.append_char('4');
        text_box.append_text("567");
        let chars = text_box.get_chars().chars();
        assert_eq!(
            chars,
            (0..7)
                .into_iter()
                .map(|i| (i as u16, 0, char::from_digit(i + 1, 10).unwrap()))
                .collect::<Vec<_>>()
        );

        text_box.set_text("1234");
        let chars = text_box.get_chars().chars();
        assert_eq!(
            chars,
            vec![
                (0, 0, '1'),
                (1, 0, '2'),
                (2, 0, '3'),
                (3, 0, '4'),
                (6, 0, ' '),
                (5, 0, ' '),
                (4, 0, ' '),
            ]
        );

        text_box.append_text("56");
        let chars = text_box.get_chars().chars();
        assert_eq!(
            chars,
            vec![
                (0, 0, '1'),
                (1, 0, '2'),
                (2, 0, '3'),
                (3, 0, '4'),
                (4, 0, '5'),
                (5, 0, '6'),
            ]
        );
    }

    #[test]
    fn set_text() {
        let size = (8, 4);
        let mut output = TestOutput::<35, 4>::new();
        let text_box = TextBox::new("ABC\nDEF", super::Listener::empty(), TextAlign::Left)
            .widget_builder()
            .build();

        let mut edit = text_box.edit();
        edit.set_wrapping_mode(Wrapping::Off);
        edit.on_event(Event::Resize(size.0, size.1), &mut ActionList::new());
        let placement = WidgetPlacement::fullscreen();
        let layout = Layout::new().with_widget(&text_box, placement);

        let s1 = "AB\nCDEF";
        edit.set_text(s1);
        let chars = edit.get_chars().chars();
        assert_eq!(
            chars,
            vec![
                (0, 0, 'A'),
                (1, 0, 'B'),
                (0, 1, 'C'),
                (1, 1, 'D'),
                (2, 1, 'E'),
                (3, 1, 'F'),
                (2, 0, ' '),
            ]
        );
        edit.set_text("ABCX123\nDEF");
        let chars = edit.get_chars().chars();
        assert_eq!(
            chars,
            vec![
                (0, 0, 'A'),
                (1, 0, 'B'),
                (2, 0, 'C'),
                (3, 0, 'X'),
                (4, 0, '1'),
                (5, 0, '2'),
                (6, 0, '3'),
                (0, 1, 'D'),
                (1, 1, 'E'),
                (2, 1, 'F'),
                (3, 1, ' '),
            ]
        );
        edit.set_text(s1);
        let chars = edit.get_chars().chars();
        assert_eq!(
            chars,
            vec![
                (0, 0, 'A'),
                (1, 0, 'B'),
                (0, 1, 'C'),
                (1, 1, 'D'),
                (2, 1, 'E'),
                (3, 1, 'F'),
                (6, 0, ' '),
                (5, 0, ' '),
                (4, 0, ' '),
                (3, 0, ' '),
                (2, 0, ' '),
            ]
        );
        edit.set_text(&("\n".to_owned() + s1));
        let chars = edit.get_chars().chars();
        assert_eq!(
            chars,
            vec![
                (0, 1, 'A'),
                (1, 1, 'B'),
                (0, 2, 'C'),
                (1, 2, 'D'),
                (2, 2, 'E'),
                (3, 2, 'F'),
                (1, 0, ' '),
                (0, 0, ' '),
                (3, 1, ' '),
                (2, 1, ' '),
            ]
        );
        edit.set_text("");

        let chars = edit.get_chars().chars();
        assert_eq!(
            chars,
            vec![
                (1, 1, ' '),
                (0, 1, ' '),
                (3, 2, ' '),
                (2, 2, ' '),
                (1, 2, ' '),
                (0, 2, ' '),
            ]
        );
        let mut ms = "aaaa".to_owned();
        edit.set_text(&ms);
        ms = "bb\n".to_owned() + &ms;
        edit.set_text(&ms);
        ms = "cccc\n".to_owned() + &ms;
        edit.set_text(&ms);
        ms = "dd\n".to_owned() + &ms;
        edit.set_text(&ms);
        let mut chars = edit.get_chars().chars();
        chars.sort();
        chars.dedup();

        assert_eq!(
            chars,
            vec![
                (0, 0, 'd'),
                (0, 1, 'c'),
                (0, 2, 'b'),
                (0, 3, 'a'),
                (1, 0, 'd'),
                (1, 1, 'c'),
                (1, 2, 'b'),
                (1, 3, 'a'),
                (2, 0, ' '),
                (2, 1, 'c'),
                (2, 2, ' '),
                (2, 3, 'a'),
                (3, 0, ' '),
                (3, 1, 'c'),
                (3, 2, ' '),
                (3, 3, 'a'),
            ]
        );

        // RIGHT ALIGN
        edit.set_text("");
        let _ = edit.get_chars().chars();
        edit.set_align(TextAlign::Right);
        edit.set_text("ABC\nDEF");
        let chars = edit.get_chars().chars();
        assert_eq!(
            chars,
            vec![
                (5, 0, 'A'),
                (6, 0, 'B'),
                (7, 0, 'C'),
                (5, 1, 'D'),
                (6, 1, 'E'),
                (7, 1, 'F'),
            ]
        );

        let s1 = "AB\nCDEF";
        edit.set_text(s1);
        let chars = edit.get_chars().chars();
        assert_eq!(
            chars,
            vec![
                (6, 0, 'A'),
                (7, 0, 'B'),
                (4, 1, 'C'),
                (5, 1, 'D'),
                (6, 1, 'E'),
                (7, 1, 'F'),
                (5, 0, ' '),
            ]
        );

        // CENTER ALIGN
        edit.set_text("");
        let _ = edit.get_chars().chars();
        edit.set_align(TextAlign::Center);
        edit.set_text("ABC\nDEF");
        let chars = edit.get_chars().chars();
        assert_eq!(
            chars,
            vec![
                (2, 0, 'A'),
                (3, 0, 'B'),
                (4, 0, 'C'),
                (2, 1, 'D'),
                (3, 1, 'E'),
                (4, 1, 'F'),
            ]
        );

        let s1 = "AB\nCDEF";
        edit.set_text(s1);
        let chars = edit.get_chars().chars();
        assert_eq!(
            chars,
            vec![
                (3, 0, 'A'),
                (4, 0, 'B'),
                (2, 1, 'C'),
                (3, 1, 'D'),
                (4, 1, 'E'),
                (5, 1, 'F'),
                (2, 0, ' '),
            ]
        );

        drop(edit);
        layout
            .render(size.0 as i32, size.1 as i32)
            .full_render_to_output(&mut output);
        let rendered_text = output.to_string();
        println!("{}", rendered_text);
        // assert_match_with_test_file(&rendered_text, "factory_widgets/centered_text");
    }

    #[test]
    fn centered_text() {
        let mut output = TestOutput::<25, 1>::new();
        let mut text_box =
            TextBox::new("Centered Text", super::Listener::empty(), TextAlign::Center)
                .widget_builder()
                .build();
        text_box
            .edit()
            .on_event(Event::Resize(25, 2), &mut ActionList::new());

        let placement = WidgetPlacement::fullscreen();
        let layout = Layout::new().with_widget(&text_box, placement);

        layout.render(25, 1).full_render_to_output(&mut output);
        let rendered_text = output.to_string();

        assert_match_with_test_file(&rendered_text, "factory_widgets/centered_text");
    }

    #[test]
    fn centered_mutliline_text() {
        let mut output = TestOutput::<25, 2>::new();
        let mut text_box = TextBox::new(
            "Centered Text\nother",
            super::Listener::empty(),
            TextAlign::Center,
        )
        .widget_builder()
        .build();
        text_box
            .edit()
            .on_event(Event::Resize(25, 2), &mut ActionList::new());
        let placement = WidgetPlacement::fullscreen();
        let layout = Layout::new().with_widget(&text_box, placement);

        layout.render(25, 2).full_render_to_output(&mut output);
        let rendered_text = output.to_string();

        assert_match_with_test_file(&rendered_text, "factory_widgets/centered_multiline_text");
    }

    #[test]
    fn right_text() {
        let mut output = TestOutput::<25, 1>::new();
        let mut text_box = TextBox::new("Right Text", super::Listener::empty(), TextAlign::Right)
            .widget_builder()
            .build();
        text_box
            .edit()
            .on_event(Event::Resize(25, 2), &mut ActionList::new());
        let placement = WidgetPlacement::fullscreen();
        let layout = Layout::new().with_widget(&text_box, placement);

        layout.render(25, 1).full_render_to_output(&mut output);
        let rendered_text = output.to_string();

        assert_match_with_test_file(&rendered_text, "factory_widgets/right_text");
    }

    #[test]
    fn right_multiline_text() {
        let mut output = TestOutput::<25, 2>::new();
        let mut text_box = TextBox::new(
            "Right Text\nshort",
            super::Listener::empty(),
            TextAlign::Right,
        )
        .widget_builder()
        .build();
        text_box
            .edit()
            .on_event(Event::Resize(25, 2), &mut ActionList::new());
        let placement = WidgetPlacement::fullscreen();
        let layout = Layout::new().with_widget(&text_box, placement);

        layout.render(25, 2).full_render_to_output(&mut output);
        let rendered_text = output.to_string();

        assert_match_with_test_file(&rendered_text, "factory_widgets/right_multiline_text");
    }

    #[test]
    fn loading_bar() {
        let increment = 1.0 / 16.0;
        let test_file = format!(
            "{}/factory_widgets/progress_bar_horizontal.txt",
            crate::test::TESTS_DIR
        );
        let expected_output = std::fs::read_to_string(&test_file)
            .unwrap()
            .lines()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();

        let mut progress_bar = factory_widgets::progression::ProgressBar::new(
            factory_widgets::progression::ProgressBarType::HorizontalNineLevels,
            0.0,
            super::Listener::empty(),
        )
        .widget_builder()
        .build();

        let mut output = TestOutput::<2, 1>::new();
        let placement = WidgetPlacement::fullscreen();
        let mut layout = Layout::new()
            .with_widget(&progress_bar, placement)
            .render(2, 1);

        for step in 0..=16 {
            let amt = 1.0 - increment * (step as f32);
            progress_bar.edit().set_amt(amt);
            layout
                .notify_size_change()
                .full_render_to_output(&mut output);
            let rendered_text = output.to_string();
            eprintln!(
                "Step {}: amt={}, rendered={:?}, expected={:?}",
                step, amt, rendered_text, expected_output[step]
            );
            assert_eq!(rendered_text, expected_output[step]);
            output.clear();
        }
    }

    #[test]
    fn no_wrapping_text() {
        let mut lorem = self::test::strings::lorem_ipsum_short()
            .to_string()
            .repeat(2);
        lorem += "0123456789abcd\nABC\nDEF";
        let mut text_box = crate::factory_widgets::text::TextBox::new(
            &lorem,
            Listener::empty(),
            factory_widgets::text::TextAlign::Left,
        );
        text_box.set_wrapping_mode(Wrapping::Off);
        let widget = text_box
            .widget_builder()
            .with_outline(symbols::OutlineStyle::Thick)
            .build();

        let rendered_text = crate::test::get_single_widget_rendered_text(&widget, (16, 8));
        crate::test::assert_match_with_test_file(
            &rendered_text,
            "factory_widgets/short_text_no_wrap.txt",
        );

        widget
            .displayable
            .write()
            .unwrap()
            .set_align(factory_widgets::text::TextAlign::Right);
        let rendered_text = crate::test::get_single_widget_rendered_text(&widget, (16, 8));
        crate::test::assert_match_with_test_file(
            &rendered_text,
            "factory_widgets/short_text_no_wrap_right_align.txt",
        );

        widget
            .displayable
            .write()
            .unwrap()
            .set_align(factory_widgets::text::TextAlign::Center);
        let rendered_text = crate::test::get_single_widget_rendered_text(&widget, (16, 8));
        crate::test::assert_match_with_test_file(
            &rendered_text,
            "factory_widgets/short_text_no_wrap_centered.txt",
        );
    }

    #[test]

    fn wrapping_letter_text() {
        let mut lorem = self::test::strings::lorem_ipsum_short()
            .to_string()
            .repeat(2);
        lorem += "0123456789abcd\nABC\nDEF";
        let mut text_box = crate::factory_widgets::text::TextBox::new(
            &lorem,
            Listener::empty(),
            factory_widgets::text::TextAlign::Left,
        );
        text_box.set_wrapping_mode(Wrapping::PerLetter);
        let widget = text_box
            .widget_builder()
            .with_outline(symbols::OutlineStyle::Thick)
            .build();

        let rendered_text = crate::test::get_single_widget_rendered_text(&widget, (18, 12));
        crate::test::assert_match_with_test_file(
            &rendered_text,
            "factory_widgets/short_text_letter_wrap.txt",
        );

        widget
            .displayable
            .write()
            .unwrap()
            .set_align(factory_widgets::text::TextAlign::Right);
        let rendered_text = crate::test::get_single_widget_rendered_text(&widget, (18, 12));
        crate::test::assert_match_with_test_file(
            &rendered_text,
            "factory_widgets/short_text_letter_wrap_right_align.txt",
        );

        widget
            .displayable
            .write()
            .unwrap()
            .set_align(factory_widgets::text::TextAlign::Center);
        let rendered_text = crate::test::get_single_widget_rendered_text(&widget, (18, 12));
        crate::test::assert_match_with_test_file(
            &rendered_text,
            "factory_widgets/short_text_letter_wrap_centered.txt",
        );
    }

    #[test]
    fn wrapping_word_text() {
        let lorem = self::test::strings::lorem_ipsum_short()
            .to_string()
            .repeat(2);
        let mut text_box = crate::factory_widgets::text::TextBox::new(
            &lorem,
            Listener::empty(),
            factory_widgets::text::TextAlign::Left,
        );
        text_box.set_wrapping_mode(Wrapping::PerWord);
        let widget = text_box
            .widget_builder()
            .with_outline(symbols::OutlineStyle::Thick)
            .build();

        let rendered_text = crate::test::get_single_widget_rendered_text(&widget, (16, 8));
        crate::test::assert_match_with_test_file(
            &rendered_text,
            "factory_widgets/short_text_word_wrap.txt",
        );

        widget
            .displayable
            .write()
            .unwrap()
            .set_align(factory_widgets::text::TextAlign::Right);
        let rendered_text = crate::test::get_single_widget_rendered_text(&widget, (16, 8));
        crate::test::assert_match_with_test_file(
            &rendered_text,
            "factory_widgets/short_text_word_wrap_right_align.txt",
        );

        widget
            .displayable
            .write()
            .unwrap()
            .set_align(factory_widgets::text::TextAlign::Center);
        let rendered_text = crate::test::get_single_widget_rendered_text(&widget, (16, 8));
        crate::test::assert_match_with_test_file(
            &rendered_text,
            "factory_widgets/short_text_word_wrap_centered.txt",
        );
    }

    #[test]
    fn text_input_editing() {
        let mut input = TextInput::new("abc", Listener::empty(), TextAlign::Left);
        let mut actions = ActionList::new();

        input.on_event(Event::Resize(16, 1), &mut actions);
        input.on_event(Event::KeyPress(crate::KeyCode::Left), &mut actions);
        input.on_event(Event::KeyPress(crate::KeyCode::Char('x')), &mut actions);
        assert_eq!(input.text(), "abxc");

        input.on_event(Event::KeyPress(crate::KeyCode::Backspace), &mut actions);
        assert_eq!(input.text(), "abc");

        input.on_event(Event::KeyPress(crate::KeyCode::Left), &mut actions);
        input.on_event(Event::KeyPress(crate::KeyCode::Delete), &mut actions);
        assert_eq!(input.text(), "ac");
        assert_eq!(
            input.get_chars().chars(),
            &[(0, 0, 'a'), (1, 0, 'c'), (3, 0, ' '), (2, 0, ' ')]
        );
    }

    #[test]
    fn progress_bar_horizontal_and_vertical() {
        let mut bar = crate::factory_widgets::progression::ProgressBar::new(
            crate::factory_widgets::progression::ProgressBarType::HorizontalNineLevels,
            0.625,
            Listener::empty(),
        );
        let mut actions = ActionList::new();
        bar.on_event(Event::Resize(4, 1), &mut actions);
        let out = bar.get_chars().chars();
        assert_eq!(
            out,
            vec![(0, 0, '█'), (1, 0, '█'), (2, 0, '▌'), (3, 0, ' ')],
            "horizontal bar"
        );
        bar.set_amt(0.0);
        assert_eq!(
            bar.get_chars().chars(),
            &[(0, 0, ' '), (1, 0, ' '), (2, 0, ' '), (3, 0, ' ')]
        );

        let mut vertical = crate::factory_widgets::progression::ProgressBar::new(
            crate::factory_widgets::progression::ProgressBarType::VerticalNineLevels,
            0.75,
            Listener::empty(),
        );
        vertical.on_event(Event::Resize(1, 4), &mut actions);
        let out = vertical.get_chars().chars();
        assert_eq!(
            out,
            vec![(0, 0, '█'), (0, 1, '█'), (0, 2, '█'), (0, 3, ' ')],
            "vertical bar"
        );
    }

    #[test]
    fn cells_divided() {
        let mut output = TestOutput::<17, 7>::new(); // TODO NOT GOOD DIMENSIONS
        let mut layout = Layout::new();
        let mut dg = FillGenerator::new();

        let mut text_boxes = Vec::from_iter((0..6).map(|_| TextBox::default()));
        text_boxes[0].set_text("ABC");
        text_boxes[1].set_text("GHI");
        text_boxes[1].set_align(TextAlign::Right);
        text_boxes[2].set_text("nw_abcdefgh");
        text_boxes[2].set_wrapping_mode(Wrapping::Off);
        text_boxes[3].set_text("DEF");
        text_boxes[3].set_align(TextAlign::Center);
        text_boxes[4].set_text("w_abcdefgh");
        text_boxes[4].set_align(TextAlign::Center);

        let mut widgets = text_boxes
            .into_iter()
            .map(|tb| Widget::new(tb))
            .collect::<Vec<_>>();

        let mut placements = [WidgetPlacement::default(); 6];

        let fs = WidgetPlacement::fullscreen();
        fs.split(2, 3, true, &mut placements);

        for i in 0..6 {
            widgets[i].set_outline(symbols::OutlineStyle::Normal);
        }

        layout.connect_and_add_widgets(&widgets, placements.as_mut_slice());
        layout
            .render(17, 7)
            .notify_size_change()
            .full_render_to_output(&mut output);
        let rendered_text = output.to_string();

        println!("{}", rendered_text);
        assert_match_with_test_file(&rendered_text, "factory_widgets/cells_divided_tb");
    }

    #[test]
    fn table() {
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

        // let set_row = |idx: usize, pid: &str, number: i32, status: &str, progress: f32| {
        //     let mut row = table.row(idx);
        //     row[0].set_value(pid.to_string());
        //     row[1].set_value(number);
        //     row[2].set_value(status.to_string());
        //     row[3].set_value(progress);
        // };
        //

        set_table_row!(table, 0, ["105", -4, "Running", 0.5]);
        set_table_row!(table, 1, ["95", 0, "Paused", 0.15]);
        set_table_row!(table, 2, ["1", 20, "Waiting", 1.0]);
        set_table_row!(table, 3, ["104", -1, "Running", 0.0]);

        // table.row_mut(1).set(&cells);
        assert_eq!(table.row(0)[0].get_width(), 3);
        assert_eq!(table.row(0)[1].get_width(), 2);
        assert_eq!(table.row(0)[2].get_width(), 7);
        table.set_cell(1, 0, -20);
        assert_eq!(table.row(0)[1].get_width(), 3);

        table.on_event(Event::Resize(40, 10), &mut ActionList::new());
        let mut output = TestOutput::<40, 10>::new();
        let layout = Layout::new()
            .with_widget(&Widget::new(table), WidgetPlacement::fullscreen())
            .render(40, 10);

        layout.full_render_to_output(&mut output);
        let rendered_text = output.to_string();
        println!("{}", rendered_text);
    }
}
