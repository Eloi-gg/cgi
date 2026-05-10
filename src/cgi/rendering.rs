use core::panic;

trait Output {
    fn place_char(&mut self, x: u16, y: u16, ch: char);
}

fn render_widget(widget: &dyn super::Displayable, size: crate::cgi::layout::ComputedWidgetPlacement, output: &mut dyn Output) {
    let mut changed_chars = Vec::new();
    widget.get_changed_chars((size.width as u16, size.height as u16), &mut changed_chars);
    for (x, y, ch) in changed_chars {
        output.place_char(size.x as u16 + x, size.y as u16 + y, ch);
    }
}

impl crate::cgi::layout::RenderedLayout {
    fn render_to_output(&self, output: &mut dyn Output) {
        for (widget_hdl, placement) in &self.0 {
            render_widget(&*widget_hdl.widget.displayable.read().unwrap(), *placement, output);
        }
    }
}

struct TestOutput<const W: usize, const H: usize> {
    buffer: [[char; W]; H],
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
        }
    }

    fn to_string(&self) -> String {
        let mut r = String::new();
        for row in &self.buffer {
            let line: String = row.iter().collect();
            r.push_str(&line);
            r.push('\r');
            r.push('\n');
        }
        r.pop();
        r.pop();
        r
    }
}

#[cfg(test)]
mod rendering_tests {
    use super::*;
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
    #[test]
    fn offsets_10_x_10() {
        let mut output = TestOutput::<10, 10>::new();

        let expected_result = std::fs::File::open(format!("{}/1_offsets_10x10.txt", TESTS_DIR)).unwrap();
        let expected_result = std::io::read_to_string(expected_result).unwrap();

        let widget = Widget::new(FillWidget {ch: '#'});
        let placement = WidgetPlacement::fullscreen().expand_or_shrink(-1, -1);
        let layout = Layout::new().with_widget(&widget, placement).render(10, 10);
        layout.render_to_output(&mut output);
        let rendered_text = output.to_string();

        assert_eq!(rendered_text, expected_result);
    }
}