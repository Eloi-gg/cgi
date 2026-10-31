use core::panic;
use std::{borrow::Cow, collections::HashMap};

use crate::{
    layout::ComputedWidgetPlacement,
    widget::{WidgetData, WidgetHdl},
};

pub(super) enum OS {
    Windows,
    Linux,
    MacOS,
}

impl OS {
    pub fn get() -> Self {
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

/// Struct used to return the characters that will be printed to the screen
/// Can own the data or borrow it
pub struct CharContainer<'a> {
    chars: Vec<Cow<'a, [(u16, u16, char)]>>,
    formats: Vec<(crate::text_formatting::CombinedFormat, usize)>,
    mapping: Vec<usize>, // Stores the total size at each `chars` element
    current_size: usize,
}

impl<'a> CharContainer<'a> {
    pub fn new_empty() -> Self {
        Self {
            chars: Vec::new(),
            formats: vec![(crate::text_formatting::CombinedFormat::default(), 0)],
            mapping: vec![0],
            current_size: 0,
        }
    }

    pub fn new_owned(chars: Vec<(u16, u16, char)>) -> Self {
        Self {
            current_size: chars.len(),
            mapping: vec![0, chars.len()],
            chars: vec![Cow::Owned(chars)],
            formats: vec![(crate::text_formatting::CombinedFormat::default(), 0)],
        }
    }

    pub fn new_borrowed(chars: &'a [(u16, u16, char)]) -> Self {
        Self {
            current_size: chars.len(),
            mapping: vec![0, chars.len()],
            chars: vec![Cow::Borrowed(chars)],
            formats: vec![(crate::text_formatting::CombinedFormat::default(), 0)],
        }
    }

    pub fn add_owned(&mut self, other: Vec<(u16, u16, char)>) {
        if self.current_size > 0 {
            self.mapping.push(self.current_size);
        }
        self.current_size += other.len();
        self.chars.push(Cow::Owned(other));
    }

    pub fn add_borrowed(&mut self, other: &'a [(u16, u16, char)]) {
        if self.current_size > 0 {
            self.mapping.push(self.current_size);
        }
        self.current_size += other.len();
        self.chars.push(Cow::Borrowed(other));
    }

    pub fn set_format(&mut self, format: crate::text_formatting::CombinedFormat) {
        let last = self.formats.last_mut().unwrap();
        if last.1 == self.current_size {
            last.0 = format;
        } else {
            self.formats.push((format, self.current_size));
        }
    }

    pub(crate) fn chars(&self) -> Vec<(u16, u16, char)> {
        self.chars
            .iter()
            .flat_map(|it| it.iter())
            .cloned()
            .collect()
    }

    fn len(&self) -> usize {
        self.current_size
    }

    fn at(
        &'a self,
        index: usize,
    ) -> Option<(
        &'a (u16, u16, char),
        &'a crate::text_formatting::CombinedFormat,
    )> {
        let (data_index, &char_mapping) = self
            .mapping
            .iter()
            .enumerate()
            .take_while(|(i, e)| **e <= index)
            .last()
            .unwrap();

        let format_idx = self
            .formats
            .iter()
            .enumerate()
            .take_while(|(j, (i, e))| *e <= index)
            .last()
            .unwrap()
            .0;

        let remaining = index - char_mapping;

        let a = self.chars[data_index].get(remaining);
        let b = self.formats.get(format_idx).map(|(f, _)| f);

        a.zip(b)
    }
}

pub(crate) trait Output {
    fn flush(&mut self);
    fn place_char(&mut self, x: u16, y: u16, ch: char);
}

impl crate::layout::RenderedLayout {
    pub(crate) fn full_render_to_output(&self, output: &mut dyn Output) {
        for (layer_idx, layer) in self.layers.iter().enumerate() {
            for (widget, placement) in layer.iter() {
                self.render_widget_to_output(widget, placement, true, layer_idx, output);
            }
        }
    }

    /// Renders a single widget to the output, NOT including its outline.
    pub(crate) fn render_single_widget_to_output(
        &self,
        widget: &crate::widget::WidgetHdl,
        placement: &ComputedWidgetPlacement,
        output: &mut dyn Output,
    ) {
        let layer = self.widget_layer[widget];
        self.render_widget_to_output(widget, placement, false, layer, output);
        // TODO: single render means extra computation for transparent widgets above?
    }

    fn render_widget_to_output(
        &self,
        widget: &crate::widget::WidgetHdl,
        placement: &ComputedWidgetPlacement,
        render_outline: bool,
        layer: usize,
        output: &mut dyn Output,
    ) {
        // Outline
        let mut outline_buffer = Vec::new();
        let (has_outline, is_transparent) = {
            let widget_data = &*widget.widget.data.lock().unwrap();
            if widget_data.visible == false {
                return;
            }
            (
                self.get_widget_outline_chars(widget_data, placement, &mut outline_buffer),
                widget_data.transparent,
            )
        };

        crate::text_formatting::CombinedFormat::reset_global();

        if render_outline {
            for (x, y, c) in outline_buffer.iter().filter(|(x, y, _)| {
                self.masks[layer].at(placement.x as u16 + x, placement.y as u16 + y)
            }) {
                output.place_char(placement.x as u16 + x, placement.y as u16 + y, *c);
            }
        }

        // Shrink placement for outline
        let placement = if has_outline {
            placement.shrinked()
        } else {
            *placement
        };
        if placement.is_null() {
            return;
        }

        // Content
        let mut lock = widget.widget.displayable.write().unwrap();
        let mut current_format = crate::text_formatting::CombinedFormat::default();
        let chars = lock.get_chars();

        for char_idx in 0..chars.len() {
            if let Some((c, &format)) = chars.at(char_idx) {
                let (x, y, c) = *c;
                if format != current_format {
                    current_format = format;
                    format.apply();
                }

                if self.masks[layer].at(placement.x as u16 + x, placement.y as u16 + y)
                    && !(is_transparent && c == ' ')
                {
                    output.place_char(x + placement.x as u16, y + placement.y as u16, c);
                }
            } else {
                break;
            }
        }
        //         for (char_set, format) in chars.into_iter() {
        //             format.apply();
        //
        //             for (x, y, c) in char_set.iter().filter(|(x, y, c)| {
        //                 self.masks[layer].at(placement.x as u16 + x, placement.y as u16 + y)
        //                     && !(is_transparent && *c == ' ')
        //             }) {
        //                 output.place_char(x + placement.x as u16, y + placement.y as u16, *c);
        //             }
        //         }
    }

    fn get_widget_outline_chars(
        &self,
        widget_data: &WidgetData,
        placement: &ComputedWidgetPlacement,
        changes: &mut Vec<(u16, u16, char)>,
    ) -> bool {
        if let Some(ref outline) = widget_data.outline {
            outline.render(
                (placement.width as u16, placement.height as u16),
                widget_data.connected,
                changes,
                widget_data.title.as_ref(),
            );
            true
        } else {
            false
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
    use crate::factory_widgets::{Listener, progression::*, text::*};
    use crate::test::*;
    use crate::*;

    #[test]
    fn offsets_10_x_10() {
        let mut output = TestOutput::<10, 10>::new();

        let widget = Widget::new(FillWidget::new('#'));
        let placement = WidgetPlacement::fullscreen().expand_or_shrink(-1, -1);
        let layout = Layout::new().with_widget(&widget, placement).render(10, 10);
        layout.full_render_to_output(&mut output);
        let rendered_text = output.to_string();

        assert_match_with_test_file(&rendered_text, "1_offsets_10x10");
    }

    #[test]
    fn side_by_side_15x8() {
        let mut output = TestOutput::<15, 8>::new();

        let widget1 = Widget::new(FillWidget::new('1'));
        let widget2 = Widget::new(FillWidget::new('2'));
        let placement1 =
            WidgetPlacement::new_with_size(Absolute(1), Absolute(0), Absolute(6), Relative(0.5));
        let placement2 = WidgetPlacement::new_with_size(
            placement1.get_bottom_right().0 + 1.into(),
            Absolute(0),
            Absolute(6),
            Relative(0.5),
        );
        let layout = Layout::new()
            .with_widget(&widget1, placement1)
            .with_widget(&widget2, placement2)
            .render(15, 8);
        layout.full_render_to_output(&mut output);
        let rendered_text = output.to_string();

        assert_match_with_test_file(&rendered_text, "2_side_by_side_15x8");
    }

    #[test]
    fn more_complex_15x8() {
        let mut output = TestOutput::<15, 8>::new();

        let widget1 = Widget::new(FillWidget::new('1'));
        let widget2 = Widget::new(FillWidget::new('2'));
        let widget3 = Widget::new(FillWidget::new('3'));
        let widget4 = Widget::new(FillWidget::new('4'));

        let placement1 =
            WidgetPlacement::new_with_size(0.0, 0.0, 1.0 / 3.0, 0.5).shift_top_left(1, 0);
        let placement2 = WidgetPlacement::new_with_size(
            placement1.get_bottom_right().0,
            Absolute(0),
            Relative(2.0 / 3.0),
            Relative(0.5),
        )
        .expand_or_shrink(-1, 0);
        let placement3 = WidgetPlacement::new_with_size(
            0.0.into(),
            Hybrid(1, 0.5),
            (2.0 / 3.0).into(),
            0.25.into(),
        )
        .expand_or_shrink(-1, 0);
        let placement4 = WidgetPlacement::new_with_size(
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
        layout.full_render_to_output(&mut output);
        let rendered_text = output.to_string();

        assert_match_with_test_file(&rendered_text, "3_more_complex_15x8");
    }

    #[test]
    fn relative() {
        let mut output = TestOutput::<12, 4>::new();

        let widget = Widget::new(FillWidget::new('1'));
        let widget2 = Widget::new(FillWidget::new('2'));

        let placement1 = WidgetPlacement::new_with_size(0, 1, 3, 2);
        let placement2 = WidgetPlacement::new_with_size(0.0, 0.0, 1.0, 1.0)
            .shift_top_left(3, 0)
            .expand_or_shrink(-1, 0);

        for (i, x) in [4, 8, 12].iter().enumerate() {
            let layout = Layout::new()
                .with_widget(&widget, placement1)
                .with_widget(&widget2, placement2)
                .render(*x, 4);
            output.clear();
            output.change_size((*x as usize, 4));
            layout.full_render_to_output(&mut output);
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
        let widget = text_box
            .widget_builder()
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

        let title = TextBox::new(
            "Title",
            Listener::empty(),
            factory_widgets::text::TextAlign::Center,
        )
        .widget_builder()
        .with_outline(symbols::OutlineStyle::Double)
        .with_title("Title")
        .build();

        let panel_left = TextBox::new(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
            Listener::empty(),
            TextAlign::Left,
        )
        .widget_builder()
        .with_outline(symbols::OutlineStyle::Rounded)
        .with_title("Panel Left")
        .build();
        let panel_right = TextBox::new(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
            Listener::empty(),
            TextAlign::Left,
        )
        .widget_builder()
        .with_outline(symbols::OutlineStyle::Rounded)
        .with_title("Panel Right")
        .build();

        let progress_bar = ProgressBar::new(
            ProgressBarType::HorizontalNineLevels,
            0.565,
            Listener::empty(),
        )
        .widget_builder()
        .with_outline(symbols::OutlineStyle::Normal)
        .with_title("Progress")
        .build();

        // Title section: top (lines 0-2, height 3)
        let title_placement =
            WidgetPlacement::new_with_size(Absolute(1), Absolute(0), Absolute(130), Absolute(3));

        // Panels section: middle (lines 5-10, height 6)
        let panels_placement =
            WidgetPlacement::new_with_size(Absolute(1), Absolute(5), Absolute(130), Absolute(6));

        let mut panels_below_placement = [WidgetPlacement::fullscreen(); 2];
        panels_placement.split(2, 1, false, &mut panels_below_placement);

        // Progress bar section: bottom (lines 13-15, height 3)
        let progress_bar_placement =
            WidgetPlacement::new_with_size(Absolute(1), Absolute(13), Absolute(130), Absolute(3));

        let mut layout = crate::Layout::new()
            .with_widget(&title, title_placement)
            .with_widget(&panel_left, panels_below_placement[0])
            .with_widget(&panel_right, panels_below_placement[1])
            .with_widget(&progress_bar, progress_bar_placement);

        let layout = layout.render(132, 16);
        output.clear();
        layout.full_render_to_output(&mut output);
        let rendered_text = output.to_string();

        println!("{}", rendered_text);
        crate::test::assert_match_with_test_file(&rendered_text, "9_titles_full");
    }
}
