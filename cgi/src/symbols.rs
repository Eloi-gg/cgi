#[derive(Debug, Copy, Clone)]
pub enum OutlineStyle {
    Normal,
    Rounded,
    Double,
    Thick,
}

impl OutlineStyle {
    pub fn set(&self) -> &line::Set {
        match self {
            OutlineStyle::Normal => &line::NORMAL,
            OutlineStyle::Rounded => &line::ROUNDED,
            OutlineStyle::Double => &line::DOUBLE,
            OutlineStyle::Thick => &line::THICK,
        }
    }
}

// Basically stolen from tui-rs. MIT license.

pub mod block {
    pub const FULL: char = '█';
    pub const SEVEN_EIGHTHS: char = '▉';
    pub const THREE_QUARTERS: char = '▊';
    pub const FIVE_EIGHTHS: char = '▋';
    pub const HALF: char = '▌';
    pub const THREE_EIGHTHS: char = '▍';
    pub const ONE_QUARTER: char = '▎';
    pub const ONE_EIGHTH: char = '▏';

    #[derive(Debug, Clone)]
    pub struct Set {
        pub full: char,
        pub seven_eighths: char,
        pub three_quarters: char,
        pub five_eighths: char,
        pub half: char,
        pub three_eighths: char,
        pub one_quarter: char,
        pub one_eighth: char,
        pub empty: char,
    }

    pub const THREE_LEVELS: Set = Set {
        full: FULL,
        seven_eighths: FULL,
        three_quarters: HALF,
        five_eighths: HALF,
        half: HALF,
        three_eighths: HALF,
        one_quarter: HALF,
        one_eighth: ' ',
        empty: ' ',
    };

    pub const NINE_LEVELS: Set = Set {
        full: FULL,
        seven_eighths: SEVEN_EIGHTHS,
        three_quarters: THREE_QUARTERS,
        five_eighths: FIVE_EIGHTHS,
        half: HALF,
        three_eighths: THREE_EIGHTHS,
        one_quarter: ONE_QUARTER,
        one_eighth: ONE_EIGHTH,
        empty: ' ',
    };

    impl Set {
        pub fn get_level(&self, amt: f32) -> char {
            assert!(amt >= 0.0 && amt <= 1.0, "amt must be between 0.0 and 1.0");
            let amt = amt.max(0.0).min(1.0);
            let level = (amt * 8.0).round() as usize;
            [
                self.full,
                self.seven_eighths,
                self.three_quarters,
                self.five_eighths,
                self.half,
                self.three_eighths,
                self.one_quarter,
                self.one_eighth,
                ' ',
            ][8-level]
        }
    }
}

pub mod bar {
    pub const FULL: char = '█';
    pub const SEVEN_EIGHTHS: char = '▇';
    pub const THREE_QUARTERS: char = '▆';
    pub const FIVE_EIGHTHS: char = '▅';
    pub const HALF: char = '▄';
    pub const THREE_EIGHTHS: char = '▃';
    pub const ONE_QUARTER: char = '▂';
    pub const ONE_EIGHTH: char = '▁';

    #[derive(Debug, Clone)]
    pub struct Set {
        pub full: char,
        pub seven_eighths: char,
        pub three_quarters: char,
        pub five_eighths: char,
        pub half: char,
        pub three_eighths: char,
        pub one_quarter: char,
        pub one_eighth: char,
        pub empty: char,
    }

    pub const THREE_LEVELS: Set = Set {
        full: FULL,
        seven_eighths: FULL,
        three_quarters: HALF,
        five_eighths: HALF,
        half: HALF,
        three_eighths: HALF,
        one_quarter: HALF,
        one_eighth: ' ',
        empty: ' ',
    };

    pub const NINE_LEVELS: Set = Set {
        full: FULL,
        seven_eighths: SEVEN_EIGHTHS,
        three_quarters: THREE_QUARTERS,
        five_eighths: FIVE_EIGHTHS,
        half: HALF,
        three_eighths: THREE_EIGHTHS,
        one_quarter: ONE_QUARTER,
        one_eighth: ONE_EIGHTH,
        empty: ' ',
    };
}

pub mod line {
    pub const VERTICAL: char = '│';
    pub const DOUBLE_VERTICAL: char = '║';
    pub const THICK_VERTICAL: char = '┃';

    pub const HORIZONTAL: char = '─';
    pub const DOUBLE_HORIZONTAL: char = '═';
    pub const THICK_HORIZONTAL: char = '━';

    pub const TOP_RIGHT: char = '┐';
    pub const ROUNDED_TOP_RIGHT: char = '╮';
    pub const DOUBLE_TOP_RIGHT: char = '╗';
    pub const THICK_TOP_RIGHT: char = '┓';

    pub const TOP_LEFT: char = '┌';
    pub const ROUNDED_TOP_LEFT: char = '╭';
    pub const DOUBLE_TOP_LEFT: char = '╔';
    pub const THICK_TOP_LEFT: char = '┏';

    pub const BOTTOM_RIGHT: char = '┘';
    pub const ROUNDED_BOTTOM_RIGHT: char = '╯';
    pub const DOUBLE_BOTTOM_RIGHT: char = '╝';
    pub const THICK_BOTTOM_RIGHT: char = '┛';

    pub const BOTTOM_LEFT: char = '└';
    pub const ROUNDED_BOTTOM_LEFT: char = '╰';
    pub const DOUBLE_BOTTOM_LEFT: char = '╚';
    pub const THICK_BOTTOM_LEFT: char = '┗';

    pub const VERTICAL_LEFT: char = '┤';
    pub const DOUBLE_VERTICAL_LEFT: char = '╣';
    pub const THICK_VERTICAL_LEFT: char = '┫';

    pub const VERTICAL_RIGHT: char = '├';
    pub const DOUBLE_VERTICAL_RIGHT: char = '╠';
    pub const THICK_VERTICAL_RIGHT: char = '┣';

    pub const HORIZONTAL_DOWN: char = '┬';
    pub const DOUBLE_HORIZONTAL_DOWN: char = '╦';
    pub const THICK_HORIZONTAL_DOWN: char = '┳';

    pub const HORIZONTAL_UP: char = '┴';
    pub const DOUBLE_HORIZONTAL_UP: char = '╩';
    pub const THICK_HORIZONTAL_UP: char = '┻';

    pub const CROSS: char = '┼';
    pub const DOUBLE_CROSS: char = '╬';
    pub const THICK_CROSS: char = '╋';

    #[derive(Debug, Clone)]
    pub struct Set {
        pub vertical: char,
        pub horizontal: char,
        pub top_right: char,
        pub top_left: char,
        pub bottom_right: char,
        pub bottom_left: char,
        pub vertical_left: char,
        pub vertical_right: char,
        pub horizontal_down: char,
        pub horizontal_up: char,
        pub cross: char,
    }

    pub const NORMAL: Set = Set {
        vertical: VERTICAL,
        horizontal: HORIZONTAL,
        top_right: TOP_RIGHT,
        top_left: TOP_LEFT,
        bottom_right: BOTTOM_RIGHT,
        bottom_left: BOTTOM_LEFT,
        vertical_left: VERTICAL_LEFT,
        vertical_right: VERTICAL_RIGHT,
        horizontal_down: HORIZONTAL_DOWN,
        horizontal_up: HORIZONTAL_UP,
        cross: CROSS,
    };

    pub const ROUNDED: Set = Set {
        top_right: ROUNDED_TOP_RIGHT,
        top_left: ROUNDED_TOP_LEFT,
        bottom_right: ROUNDED_BOTTOM_RIGHT,
        bottom_left: ROUNDED_BOTTOM_LEFT,
        ..NORMAL
    };

    pub const DOUBLE: Set = Set {
        vertical: DOUBLE_VERTICAL,
        horizontal: DOUBLE_HORIZONTAL,
        top_right: DOUBLE_TOP_RIGHT,
        top_left: DOUBLE_TOP_LEFT,
        bottom_right: DOUBLE_BOTTOM_RIGHT,
        bottom_left: DOUBLE_BOTTOM_LEFT,
        vertical_left: DOUBLE_VERTICAL_LEFT,
        vertical_right: DOUBLE_VERTICAL_RIGHT,
        horizontal_down: DOUBLE_HORIZONTAL_DOWN,
        horizontal_up: DOUBLE_HORIZONTAL_UP,
        cross: DOUBLE_CROSS,
    };

    pub const THICK: Set = Set {
        vertical: THICK_VERTICAL,
        horizontal: THICK_HORIZONTAL,
        top_right: THICK_TOP_RIGHT,
        top_left: THICK_TOP_LEFT,
        bottom_right: THICK_BOTTOM_RIGHT,
        bottom_left: THICK_BOTTOM_LEFT,
        vertical_left: THICK_VERTICAL_LEFT,
        vertical_right: THICK_VERTICAL_RIGHT,
        horizontal_down: THICK_HORIZONTAL_DOWN,
        horizontal_up: THICK_HORIZONTAL_UP,
        cross: THICK_CROSS,
    };

    impl Set {
        pub(crate) fn render(&self, size: (u16, u16), output: &mut Vec<(u16, u16, char)>, title: Option<&String>) {
            for x in 1..(size.0 - 1) {
                output.push((x, 0, self.horizontal));
                output.push((x, size.1 - 1, self.horizontal));
            }

            for y in 1..(size.1 - 1) {
                output.push((0, y, self.vertical));
                output.push((size.0 - 1, y, self.vertical));
            }

            output.push((0, 0, self.top_left));
            output.push((size.0 - 1, 0, self.top_right));
            output.push((0, size.1 - 1, self.bottom_left));
            output.push((size.0 - 1, size.1 - 1, self.bottom_right));

            // Render title if provided
            if let Some(title_text) = title {
                let title_bytes = title_text.as_bytes();
                let available_width = (size.0 - 2) as usize;
                let title_len = title_bytes.len().min(available_width);
                
                // Position title at x=1 (right after top_left corner)
                for (i, &byte) in title_bytes[..title_len].iter().enumerate() {
                    output.push((1 + i as u16, 0, byte as char));
                }
            }
        }
    }
}

#[cfg(test)]
mod outlines {
    use super::OutlineStyle;
    use crate::factory_widgets::Listener;
    use crate::rendering::Output;
    use crate::test::FillWidget;
    use crate::{widget::WidgetBuilder, *};

    #[test]
    fn normal_variable_size() {
        let widget = &WidgetBuilder::new(FillWidget::new('#'))
            .with_outline(OutlineStyle::Rounded)
            .build();

        for size in [3, 6, 10, 21] {
            let rendered_text =
                crate::test::get_single_widget_rendered_text(widget, (size, size / 3));
            println!("{}", rendered_text);
        }
    }

    #[test]
    fn long_text() {
        let text_box = crate::factory_widgets::text::TextBox::new(
            self::test::strings::lorem_ipsum_long(),
            Listener::empty(),
            factory_widgets::text::TextAlign::Left,
        );
        let widget = WidgetBuilder::new(text_box)
            .with_outline(OutlineStyle::Thick)
            .build();
        let rendered_text = crate::test::get_single_widget_rendered_text(&widget, (16, 8));
        println!("{}", rendered_text);
        crate::test::assert_match_with_test_file(&rendered_text, "7_borders.txt");
    }

    #[test]
    fn offset_long_text() {
        use crate::*;
        let mut output = crate::rendering::TestOutput::<48, 16>::new();
        let text_box = crate::factory_widgets::text::TextBox::new(
            self::test::strings::lorem_ipsum_long(),
            Listener::empty(),
                factory_widgets::text::TextAlign::Left,
        );
        // let simpler_widget = WidgetBuilder::new(FillWidget::new('#'))
        //     .with_outline(OutlineStyle::Thick)
        //     .build();
        let widget = WidgetBuilder::new(text_box)
            .with_outline(OutlineStyle::Thick)
            .build();

        output.flush();
        // let placement = WidgetPlacement::new(x, 0, 16, 8);
        let placement = WidgetPlacement::fullscreen()
            .expand_or_shrink(-0.25, -0.25)
            .shift_bottom_right(-0.2, -0.2);
        let layout = Layout::new().with_widget(&widget, placement);

        let layout = layout.render(48, 16);
        layout.render_to_output(&mut output);
        dbg!(placement);
        dbg!(layout.0.iter().next().unwrap());
        let rendered_text = output.to_string();
        println!("{}", rendered_text.trim_end());

        // crate::test::assert_match_with_test_file(&rendered_text, "7_borders.txt");
    }
}
