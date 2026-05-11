use core::panic;

enum OS {
    Windows,
    Linux,
    MacOS,
}

impl OS {
    fn get() -> Self {
        let os = std::env::consts::OS;
        match os {
            "windows" => OS::Windows,
            "linux" => OS::Linux,
            "macos" => OS::MacOS,
            _ => panic!("Unsupported OS: {}", os),
        }
    }

    fn return_char(&self) -> String {
        match self {
            OS::Windows => "\r\n".to_string(),
            OS::Linux | OS::MacOS => "\n".to_string(),
        }
    }
}
trait Output {
    fn place_char(&mut self, x: u16, y: u16, ch: char);
}

fn render_widget(
    widget: &dyn super::Displayable,
    size: crate::cgi::layout::ComputedWidgetPlacement,
    output: &mut dyn Output,
) {
    let mut changed_chars = Vec::new();
    widget.get_changed_chars((size.width as u16, size.height as u16), &mut changed_chars);
    for (x, y, ch) in changed_chars {
        output.place_char(size.x as u16 + x, size.y as u16 + y, ch);
    }
}

impl crate::cgi::layout::RenderedLayout {
    fn render_to_output(&self, output: &mut dyn Output) {
        for (widget_hdl, placement) in &self.0 {
            render_widget(
                &*widget_hdl.widget.displayable.read().unwrap(),
                *placement,
                output,
            );
        }
    }
}

struct TestOutput<const W: usize, const H: usize> {
    buffer: [[char; W]; H],
    current_size: (usize, usize),
}

impl<const W: usize, const H: usize> Output for TestOutput<W, H> {
    fn place_char(&mut self, x: u16, y: u16, ch: char) {
        if x < W as u16 && y < H as u16 {
            self.buffer[y as usize][x as usize] = ch;
        } else {
            panic!("Attempted to place char out of bounds: ({}, {})", x, y);
        }
    }
}

impl<const W: usize, const H: usize> TestOutput<W, H> {
    fn new() -> Self {
        Self {
            buffer: [[' '; W]; H],
            current_size: (W, H),
        }
    }

    fn to_string(&self) -> String {
        let mut r = String::new();
        let return_char = OS::get().return_char();
        for row_i in 0..self.current_size.1 {
            let line: String = self.buffer[row_i][0..self.current_size.0].iter().collect();
            r.push_str(&line);
            r.push_str(&return_char);
        }
        for _ in 0..return_char.len() {
            r.pop();
        }
        r
    }

    fn change_size(&mut self, new_size: (usize, usize)) {
        self.current_size = new_size;
    }

    fn clear(&mut self) {
        for y in 0..self.current_size.1 {
            for x in 0..self.current_size.0 {
                self.buffer[y][x] = ' ';
            }
        }
    }
}

#[cfg(test)]
mod rendering_tests {
    use super::*;
    use crate::cgi::coordinate::Coordinate::*;
    use crate::cgi::*;

    static TESTS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/");

    struct FillWidget {
        ch: char,
    }

    impl Displayable for FillWidget {
        fn display(&self) {
            // No-op for testing
        }

        fn name(&self) -> String {
            format!("FillWidget '{}'", self.ch)
        }

        fn get_changed_chars(&self, size: (u16, u16), out: &mut Vec<(u16, u16, char)>) {
            for y in 0..size.1 {
                for x in 0..size.0 {
                    out.push((x, y, self.ch));
                }
            }
        }
    }

    fn assert_match_with_test_file(text: &str, file_name: &str) {
        let extension = if file_name.ends_with(".txt") {
            ""
        } else {
            ".txt"
        };
        let expected_result =
            std::fs::File::open(format!("{}/{}{}", TESTS_DIR, file_name, extension)).unwrap();
        let expected_result = std::io::read_to_string(expected_result).unwrap();
        assert_eq!(text, expected_result);
    }

    #[test]
    fn offsets_10_x_10() {
        let mut output = TestOutput::<10, 10>::new();

        let widget = Widget::new(FillWidget { ch: '#' });
        let placement = WidgetPlacement::fullscreen().expand_or_shrink(-1, -1);
        let layout = Layout::new().with_widget(&widget, placement).render(10, 10);
        layout.render_to_output(&mut output);
        let rendered_text = output.to_string();

        assert_match_with_test_file(&rendered_text, "1_offsets_10x10");
    }

    #[test]
    fn side_by_side_15x8() {
        let mut output = TestOutput::<15, 8>::new();

        let widget1 = Widget::new(FillWidget { ch: '1' });
        let widget2 = Widget::new(FillWidget { ch: '2' });
        let placement1 = WidgetPlacement::new(Absolute(1), Absolute(0), Absolute(6), Relative(0.5));
        let placement2 = WidgetPlacement::new(
            placement1.get_bottom_right().0 + 1.into(),
            Absolute(0),
            Absolute(6),
            Relative(0.5),
        );
        let layout = Layout::new()
            .with_widget(&widget1, placement1)
            .with_widget(&widget2, placement2)
            .render(15, 8);
        layout.render_to_output(&mut output);
        let rendered_text = output.to_string();

        assert_match_with_test_file(&rendered_text, "2_side_by_side_15x8");
    }

    #[test]
    fn more_complex_15x8() {
        let mut output = TestOutput::<15, 8>::new();

        let widget1 = Widget::new(FillWidget { ch: '1' });
        let widget2 = Widget::new(FillWidget { ch: '2' });
        let widget3 = Widget::new(FillWidget { ch: '3' });
        let widget4 = Widget::new(FillWidget { ch: '4' });

        let placement1 = WidgetPlacement::new(0.0, 0.0, 1.0 / 3.0, 0.5).shift_top_left(1, 0);
        let placement2 = WidgetPlacement::new(
            placement1.get_bottom_right().0,
            Absolute(0),
            Relative(2.0 / 3.0),
            Relative(0.5),
        )
        .expand_or_shrink(-1, 0);
        let placement3 =
            WidgetPlacement::new(0.0.into(), Hybrid(1, 0.5), (2.0 / 3.0).into(), 0.25.into())
                .expand_or_shrink(-1, 0);
        let placement4 = WidgetPlacement::new(
            placement3.get_bottom_right().0,
            Hybrid(1, 0.5),
            Relative(1.0 / 3.0),
            Relative(0.25),
        )
        .shift_top_left(1, 0);

        let layout = Layout::new()
            .with_widget(&widget1, placement1)
            .with_widget(&widget2, placement2)
            .with_widget(&widget3, placement3)
            .with_widget(&widget4, placement4)
            .render(15, 8);
        layout.render_to_output(&mut output);
        let rendered_text = output.to_string();

        assert_match_with_test_file(&rendered_text, "3_more_complex_15x8");
    }

    #[test]
    fn relative() {
        let mut output = TestOutput::<12, 4>::new();

        let widget = Widget::new(FillWidget { ch: '1' });
        let widget2 = Widget::new(FillWidget { ch: '2' });

        let placement1 = WidgetPlacement::new(0, 1, 3, 2);
        let placement2 = WidgetPlacement::new(0.0, 0.0, 1.0, 1.0)
            .shift_top_left(3, 0)
            .expand_or_shrink(-1, 0);

        for (i, x) in [4, 8, 12].iter().enumerate() {
            let layout = Layout::new()
                .with_widget(&widget, placement1)
                .with_widget(&widget2, placement2)
                .render(*x, 4);
            output.clear();
            output.change_size((*x as usize, 4));
            layout.render_to_output(&mut output);
            let rendered_text = output.to_string();

            assert_match_with_test_file(&rendered_text, &format!("{}_relative_{}x4", i + 4, *x));
        }
    }
}
