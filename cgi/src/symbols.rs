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
    use crate::widget;

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
        pub(crate) fn render(&self, size: (u16, u16), connections: u8, output: &mut Vec<(u16, u16, char)>, title: Option<&String>) {
            use widget::connections::*;

            for x in 1..(size.0 - 1) {
                output.push((x, 0, self.horizontal));
                output.push((x, size.1 - 1, self.horizontal));
            }

            for y in 1..(size.1 - 1) {
                output.push((0, y, self.vertical));
                output.push((size.0 - 1, y, self.vertical));
            }
            
            let tl_connections = (connections & (0b11 << TL_CORNER_OFFSET)) >> TL_CORNER_OFFSET;
            let tl_char = match tl_connections & (CONNECTED_LATERAL | CONNECTED_VERTICAL) {
                0b11 => self.cross, // LATERAL | VERTICAL
                CONNECTED_LATERAL => self.horizontal_down,
                CONNECTED_VERTICAL => self.vertical_right,
                0 => self.top_left,
                _ => panic!("how does this happen")
            };

            let tr_connections = (connections & (0b11 << TR_CORNER_OFFSET)) >> TR_CORNER_OFFSET;
            let tr_char = match tr_connections & (CONNECTED_LATERAL | CONNECTED_VERTICAL) {
                0b11 => self.cross, // LATERAL | VERTICAL
                CONNECTED_LATERAL => self.horizontal_down,
                CONNECTED_VERTICAL => self.vertical_left,
                0 => self.top_right,
                _ => panic!("how does this happen")
            };

            let bl_connections = (connections & (0b11 << BL_CORNER_OFFSET)) >> BL_CORNER_OFFSET;
            let bl_char = match bl_connections & (CONNECTED_LATERAL | CONNECTED_VERTICAL) {
                0b11 => self.cross, // LATERAL | VERTICAL
                CONNECTED_LATERAL => self.horizontal_up,
                CONNECTED_VERTICAL => self.vertical_right,
                0 => self.bottom_left,
                _ => panic!("how does this happen")
            };

            let br_connections = (connections & (0b11 << BR_CORNER_OFFSET)) >> BR_CORNER_OFFSET;
            let br_char = match br_connections & (CONNECTED_LATERAL | CONNECTED_VERTICAL) {
                0b11 => self.cross, // LATERAL | VERTICAL
                CONNECTED_LATERAL => self.horizontal_up,
                CONNECTED_VERTICAL => self.vertical_left,
                0 => self.bottom_right,
                _ => panic!("how does this happen")
            };

            output.push((0, 0, tl_char));
            output.push((size.0 - 1, 0, tr_char));
            output.push((0, size.1 - 1, bl_char));
            output.push((size.0 - 1, size.1 - 1, br_char));

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
    use crate::test::*;

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
            &self::test::strings::lorem_ipsum_long(),
            Listener::empty(),
            factory_widgets::text::TextAlign::Left,
        );
        let widget = WidgetBuilder::new(text_box)
            .with_outline(OutlineStyle::Thick)
            .build();
        widget
            .displayable
            .write()
            .unwrap()
            .on_event(crate::Event::Resize(14, 6), &mut ActionList::new());
        
        let rendered_text = crate::test::get_single_widget_rendered_text(&widget, (16, 8));
        println!("{}", rendered_text);
        crate::test::assert_match_with_test_file(&rendered_text, "7_borders.txt");
    }

    #[test]
    fn borders() {
        use crate::symbols::OutlineStyle;
        let mut output = TestOutput::<{ 4 * 4 }, 4>::new();

        let mut widgets = FillGenerator::new().get_n_widgets(4);
        let borders = [
            OutlineStyle::Normal,
            OutlineStyle::Rounded,
            OutlineStyle::Double,
            OutlineStyle::Thick,
        ];
        let mut placements = [WidgetPlacement::new(0, 0, 3, 3); 4];
        for i in 0..4 {
            for j in 0..i {
                placements[j] = placements[j].shift(4, 0);
            }
            widgets[i].set_outline(borders[i]);
        }

        let mut layout = Layout::new();
        for (widget, placement) in widgets.iter().zip(placements.iter()) {
            layout.add_widget(widget, *placement);
        }

        let layout = layout.render(16, 4);
        output.clear();
        layout.render_to_output(&mut output);
        let rendered_text = output.to_string();

        assert_match_with_test_file(&rendered_text, "10_border_types");
    }
    
    #[test]
    fn split_borders() {
        let mut output = TestOutput::<17, 7>::new(); // TODO NOT GOOD DIMENSIONS
        let mut layout = Layout::new();
        let mut dg = FillGenerator::new();
        let mut widgets = dg.get_n_widgets(6);

        let mut placements = [WidgetPlacement::default(); 6];
        
        let fs = WidgetPlacement::fullscreen();
        fs.split(2, 3, true, &mut placements);

        for i in 0..6 {
            widgets[i].set_outline(OutlineStyle::Normal);
            // layout.add_widget(&widgets[i], placements[i as usize]);
        }
        layout.connect_and_add_widgets(&mut widgets, placements.as_mut_slice());
        layout.render(17, 7).render_to_output(&mut output);
        let rendered_text = output.to_string();
        
        assert_match_with_test_file(&rendered_text, "11_split_borders");
    }

    #[test]
    fn test_split_todo_delete() {
        let mut output = TestOutput::<6, 6>::new(); 
        let mut layout = Layout::new();

        let mut widgets = FillGenerator::new().get_n_widgets(2);
        widgets[0].set_outline(OutlineStyle::Normal);
        widgets[1].set_outline(OutlineStyle::Normal);
        let p1 = WidgetPlacement::new(0, 0, 2, 2);
        let p2 = WidgetPlacement::new(2, 2, 2, 2);
        let mut mvec = vec![p1, p2];
        layout.connect_and_add_widgets(&mut widgets, mvec.as_mut_slice());
        
        layout.render(6, 6).render_to_output(&mut output);
        let rendered_text = output.to_string();
        println!("{}", rendered_text);
    }
}
