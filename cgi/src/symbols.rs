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
        pub(crate) fn render(
            &self,
            placement: crate::layout::ComputedWidgetPlacement,
            output: &mut Vec<(u16, u16, char)>,
        ) {
            dbg!(placement);
            for x in 1..(placement.width - 1) {
                let x = x + placement.x;

                output.push((x as u16, placement.y as u16, self.horizontal));
                output.push((
                    x as u16,
                    (placement.y + placement.height - 1) as u16,
                    self.horizontal,
                ));
            }

            for y in 1..(placement.height - 1) {
                let y = y + placement.y;

                output.push((placement.x as u16, y as u16, self.vertical));
                output.push((
                    (placement.x + placement.width - 1) as u16,
                    y as u16,
                    self.vertical,
                ));
            }

            output.push((placement.x as u16, placement.y as u16, self.top_left));
            output.push((
                (placement.x + placement.width - 1) as u16,
                placement.y as u16,
                self.top_right,
            ));
            output.push((
                placement.x as u16,
                (placement.y + placement.height - 1) as u16,
                self.bottom_left,
            ));
            output.push((
                (placement.x + placement.width - 1) as u16,
                (placement.y + placement.height - 1) as u16,
                self.bottom_right,
            ));
        }
    }
}

#[cfg(test)]
mod outlines {
    use crate::{widget::WidgetBuilder, *};
    use crate::test::FillWidget;
    use super::OutlineStyle;

    #[test]
    fn normal_variable_size() {
        let widget = &WidgetBuilder::new(FillWidget::new('#')).with_outline(OutlineStyle::Normal).build();

        for size in [3, 5, 10, 21] {
            let rendered_text = crate::test::get_single_widget_rendered_text(widget, (size, size / 3));
            println!("{}", rendered_text);
        }
    }
}
