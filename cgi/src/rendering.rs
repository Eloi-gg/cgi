use core::panic;
use std::collections::HashMap;

use crate::{layout::ComputedWidgetPlacement, widget::WidgetHdl};

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

pub(crate) trait Output {
    fn flush(&mut self);
    fn place_char(&mut self, x: u16, y: u16, ch: char);
}

// fn render_widget(
//     widget: &dyn super::Displayable, //
//     size: crate::layout::ComputedWidgetPlacement,
//     output: &mut dyn Output,
// ) {
//     let mut changed_chars = Vec::new();
//     widget.get_changed_chars((size.width as u16, size.height as u16), &mut changed_chars);
//     for (x, y, ch) in changed_chars {
//         output.place_char(size.x as u16 + x, size.y as u16 + y, ch);
//     }
// }

impl crate::layout::RenderedLayout {
    pub(crate) fn new(mut layout: HashMap<WidgetHdl, ComputedWidgetPlacement>) -> crate::layout::RenderedLayout {
        let mut actions = crate::ActionList::new();
        for (widget, placement) in layout.iter_mut() {
            let inside_placement = if let Ok(data) = widget.widget.data.lock() {
                if let Some(_) = (*data).outline {
                    ComputedWidgetPlacement {
                        x: placement.x + 1,
                        y: placement.y + 1,
                        width: placement.width - 2,
                        height: placement.height - 2,
                    }
                } else {
                    *placement
                }
            } else {
                *placement
            };
            widget.widget.displayable.write().unwrap().on_event(
                crate::Event::Resize(inside_placement.width as u16, inside_placement.height as u16),
                &mut actions,
            );
        }

        Self(layout)
    }

    pub(crate) fn render_to_output(&self, output: &mut dyn Output) {
        let mut global_changes = Vec::new();

        for (widget, placement) in self.0.iter() {
            self.render_widget(widget, placement, &mut global_changes);
        }

        for (x, y, c) in global_changes {
            output.place_char(x, y, c);
        }
    }

    pub(crate) fn render_widget_to_output(
        &self,
        widget: &crate::widget::WidgetHdl,
        output: &mut dyn Output,
    ) {
        let placement = self
            .0
            .get(widget)
            .expect("Widget not found in rendered layout");
        let mut global_changes = Vec::new();
        self.render_widget(widget, placement, &mut global_changes);
        for (x, y, c) in global_changes {
            output.place_char(x, y, c);
        }
    }

    fn render_widget(
        &self,
        widget: &crate::widget::WidgetHdl,
        placement: &ComputedWidgetPlacement,
        global_changes: &mut Vec<(u16, u16, char)>,
    ) {
        let mut local_changes = Vec::new();

        if let Ok(data) = widget.widget.data.lock() {
            //TODO: do we really need `dirty`?
            if (*data).dirty {
                let inside_placement = if let Some(ref outline) = (*data).outline {
                    outline.render(
                        (placement.width as u16, placement.height as u16),
                        data.connected,
                        &mut local_changes,
                        (*data).title.as_ref(),
                    );

                    for (x, y, c) in local_changes.drain(..) {
                        global_changes.push((x + placement.x as u16, y + placement.y as u16, c));
                    }
                    ComputedWidgetPlacement {
                        x: placement.x + 1,
                        y: placement.y + 1,
                        width: placement.width - 2,
                        height: placement.height - 2,
                    }
                } else {
                    *placement
                };
                if inside_placement.width > 0 && inside_placement.height > 0 {
                    widget
                        .widget
                        .displayable
                        .write()
                        .unwrap()
                        .get_changed_chars(
                            (
                                inside_placement.width as u16,
                                inside_placement.height as u16,
                            ),
                            &mut local_changes,
                        );
                    // (*data).dirty = false;
                    for (x, y, c) in local_changes.drain(..) {
                        global_changes.push((
                            x + inside_placement.x as u16,
                            y + inside_placement.y as u16,
                            c,
                        ));
                    }
                }
            }
        }
    }
}

pub(crate) struct TestOutput<const W: usize, const H: usize> {
    buffer: [[char; W]; H],
    current_size: (usize, usize),
}

impl<const W: usize, const H: usize> Output for TestOutput<W, H> {
    fn flush(&mut self) {}

    fn place_char(&mut self, x: u16, y: u16, ch: char) {
        if x < W as u16 && y < H as u16 {
            self.buffer[y as usize][x as usize] = ch;
        } else {
            panic!("Attempted to place char out of bounds: ({}, {})", x, y);
        }
    }
}

impl<const W: usize, const H: usize> TestOutput<W, H> {
    pub fn new() -> Self {
        Self {
            buffer: [[' '; W]; H],
            current_size: (W, H),
        }
    }

    pub fn to_string(&self) -> String {
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

    pub fn change_size(&mut self, new_size: (usize, usize)) {
        self.current_size = new_size;
    }

    pub fn clear(&mut self) {
        for y in 0..self.current_size.1 {
            for x in 0..self.current_size.0 {
                self.buffer[y][x] = ' ';
            }
        }
    }
}

pub(crate) struct LinuxOutput;

impl Output for LinuxOutput {
    fn place_char(&mut self, x: u16, y: u16, ch: char) {
        // Move cursor to position (x, y) using ANSI escape code
        // ESC[row;colH moves cursor to row and col (1-indexed)
        print!("\x1b[{};{}H{}", y + 1, x + 1, ch);

        // // Flush immediately to show character right away
        // use std::io::Write;
        // std::io::stdout().flush().unwrap();
    }

    fn flush(&mut self) {
        use std::io::Write;
        std::io::stdout().flush().unwrap();
        // clear the screen
        print!("\x1b[2J\x1b[H");
    }
}

#[cfg(test)]
mod rendering_tests {
    use std::println;

    use super::*;
    use crate::coordinate::Coordinate::*;
    use crate::factory_widgets::{progression::*, text::*, Listener};
    use crate::test::*;
    use crate::*;

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

    #[test]
    fn title() {
        let text_box = crate::factory_widgets::text::TextBox::new(
            &self::test::strings::lorem_ipsum_long(),
            factory_widgets::Listener::empty(),
            factory_widgets::text::TextAlign::Left,
        );
        let widget = WidgetBuilder::new(text_box)
            .with_outline(symbols::OutlineStyle::Thick)
            .with_title("Title")
            .build();

        let rendered_text = crate::test::get_single_widget_rendered_text(&widget, (16, 8));
        println!("{}", rendered_text);
        crate::test::assert_match_with_test_file(&rendered_text, "8_title.txt");
    }

    #[test]
    fn titles_full() {
        let mut output = TestOutput::<132, 16>::new();

        let title = WidgetBuilder::new(TextBox::new(
            "Title",
            Listener::empty(),
            factory_widgets::text::TextAlign::Center,
        ))
        .with_outline(symbols::OutlineStyle::Double)
        .with_title("Title")
        .build();

        let panel_left = WidgetBuilder::new(TextBox::new(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
            Listener::empty(),
            TextAlign::Left,
        ))
        .with_outline(symbols::OutlineStyle::Rounded)
        .with_title("Panel Left")
        .build();
        let panel_right = WidgetBuilder::new(TextBox::new(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
            Listener::empty(),
            TextAlign::Left,
        ))
        .with_outline(symbols::OutlineStyle::Rounded)
        .with_title("Panel Right")
        .build();

        let progress_bar = WidgetBuilder::new(ProgressBar::new(
            ProgressBarType::HorizontalNineLevels,
            0.565,
            Listener::empty(),
        ))
        .with_outline(symbols::OutlineStyle::Normal)
        .with_title("Progress")
        .build();

        // Title section: top (lines 0-2, height 3)
        let title_placement =
            WidgetPlacement::new(Absolute(1), Absolute(0), Absolute(130), Absolute(3));

        // Panels section: middle (lines 5-10, height 6)
        let panels_placement =
            WidgetPlacement::new(Absolute(1), Absolute(5), Absolute(130), Absolute(6));

        let mut panels_below_placement = [WidgetPlacement::fullscreen(); 2];
        panels_placement.split(2, 1, &mut panels_below_placement);

        // Progress bar section: bottom (lines 13-15, height 3)
        let progress_bar_placement =
            WidgetPlacement::new(Absolute(1), Absolute(13), Absolute(130), Absolute(3));

        let mut layout = crate::Layout::new()
            .with_widget(&title, title_placement)
            .with_widget(&panel_left, panels_below_placement[0])
            .with_widget(&panel_right, panels_below_placement[1])
            .with_widget(&progress_bar, progress_bar_placement);

        let layout = layout.render(132, 16);
        output.clear();
        layout.render_to_output(&mut output);
        let rendered_text = output.to_string();
        crate::test::assert_match_with_test_file(&rendered_text, "9_titles_full");
    }

    
}
