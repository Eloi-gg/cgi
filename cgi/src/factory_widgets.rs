use std::collections::HashSet;

use crate::{Displayable, EventType};

pub struct Listener<T: ?Sized> {
    events: Vec<EventType>,
    on_event: fn(crate::Event, &mut T),
}

impl<T: ?Sized> Listener<T> {
    pub fn empty() -> Self {
        Self::new(|_, _| {})
    }

    pub fn new(on_event: fn(crate::Event, &mut T)) -> Self {
        Self {
            events: Vec::new(),
            on_event,
        }
    }

    pub fn listen_for(&mut self, event: crate::EventType) {
        self.events.push(event.into());
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

pub mod progression {
    use super::*;
    use crate::symbols::{bar, block};

    pub enum ProgressBarType {
        HorizontalNineLevels,
        HorizontalThreeLevels,
        VerticalNineLevels,
        VerticalThreeLevels,
    }

    pub struct ProgressBar {
        bar_type: ProgressBarType,

        amt: f32,
        listener: Listener<Self>,
    }

    impl ProgressBar {
        pub fn new(bar_type: ProgressBarType, amt: f32, listener: Listener<Self>) -> Self {
            Self {
                bar_type,
                amt,
                listener,
            }
        }

        pub fn set_amt(&mut self, amt: f32) {
            self.amt = amt;
        }
    }

    impl Displayable for ProgressBar {
        fn display(&self) {
            todo!()
        }

        fn name(&self) -> String {
            todo!()
        }

        fn get_changed_chars(&mut self, size: (u16, u16), out: &mut Vec<(u16, u16, char)>) {
            if size.0 * size.1 == 0 {
                return;
            }

            match self.bar_type {
                ProgressBarType::HorizontalNineLevels | ProgressBarType::VerticalNineLevels => {
                    let set = match self.bar_type {
                        ProgressBarType::HorizontalNineLevels => block::NINE_LEVELS,
                        ProgressBarType::HorizontalThreeLevels => block::THREE_LEVELS,
                        _ => unreachable!(),
                    };
                    let filled_width = (self.amt * size.0 as f32) as usize;
                    let partial_fill = (self.amt * size.0 as f32).fract();

                    for x in 0..size.0 as usize {
                        let ch = if x < filled_width {
                            block::FULL
                        } else if x == filled_width && partial_fill > 0.0 {
                            set.get_level(partial_fill)
                        } else {
                            ' '
                        };
                        out.push((x as u16, 0, ch));
                    }
                }
                _ => {
                    todo!()
                }
            }
        }

    fn on_event(&mut self, event: crate::Event, actions: &mut crate::ActionList) {
            if self.listener.is_listening_for(event.into()) {
                (self.listener.on_event)(event, self);
            }
        }
    }
}

pub mod text {
    use super::*;

    pub enum TextAlign {
        Left,
        Center,
        Right,
    }

    pub struct TextBox {
        text: Vec<char>,
        formatted_text: Vec<Vec<char>>, // TODO: change structure : store one big chunk of memory
        changed_chars: Vec<usize>, // points to chars in the text
        size: (u16, u16),          // Remove ?
        listener: Listener<Self>,
        align: TextAlign,
    }

    impl TextBox {
        pub fn new(text: &str, listener: Listener<Self>, align: TextAlign) -> Self {
            let text: Vec<char> = text.chars().collect();
            let changed_chars: Vec<usize> = (0..text.len()).collect();
            Self {
                text,
                changed_chars,
                size: (0, 0),
                listener,
                align,
            }
        }

        pub fn set_text(&mut self, text: &str) {
            for (i, c) in text.chars().enumerate() {
                if self.text[i] != c {
                    self.text[i] = c;
                    self.changed_chars.push(i);
                }
            }

            for i in text.len()..self.text.len() {
                self.text[i] = ' ';
                self.changed_chars.push(i);
            }
        }

        pub fn append_text(&mut self, text: &str) {
            let start_index = self.text.len();
            self.text.extend(text.chars());
            for i in start_index..self.text.len() {
                self.changed_chars.push(i);
            }
        }

        pub fn append_char(&mut self, c: char) {
            let index = self.text.len();
            self.text.push(c);
            self.changed_chars.push(index);
        }

        pub fn remove_text(&mut self, start: usize, end: usize) {
            if start >= end || end > self.text.len() {
                return;
            }
            for i in start..end {
                self.text[i] = ' ';
                self.changed_chars.push(i);
            }
        }

        pub fn text_len(&self) -> usize {
            self.text.len()
        }
    }

    impl Displayable for TextBox {
        fn display(&self) {
            todo!()
        }

        fn name(&self) -> String {
            todo!()
        }

        fn get_changed_chars(&mut self, size: (u16, u16), out: &mut Vec<(u16, u16, char)>) {
            if size.0 * size.1 == 0 {
                return;
            }

            let mut line: u16 = 0;
            let mut current_line_text: Vec<char> = Vec::new();
            let align = &self.align;
            let text = &self.text;
            let width = size.0 as usize;

            for i in self.changed_chars.drain(..) {
                if line >= size.1 {
                    break;
                }

                let ch = text[i];

                // Handle newlines
                if ch == '\n' {
                    // Output current line with alignment
                    let line_width = current_line_text.len();
                    let offset = match align {
                        TextAlign::Left => 0,
                        TextAlign::Center => width.saturating_sub(line_width) / 2,
                        TextAlign::Right => width.saturating_sub(line_width),
                    };

                    for (j, &c) in current_line_text.iter().enumerate() {
                        let final_column = j + offset;
                        if final_column < width {
                            out.push((final_column as u16, line, c));
                        }
                    }

                    line += 1;
                    current_line_text.clear();
                    continue;
                }

                // Check if adding this character would exceed the width
                if current_line_text.len() >= width {
                    // Output current line with alignment
                    let line_width = current_line_text.len();
                    let offset = match align {
                        TextAlign::Left => 0,
                        TextAlign::Center => width.saturating_sub(line_width) / 2,
                        TextAlign::Right => width.saturating_sub(line_width),
                    };

                    for (j, &c) in current_line_text.iter().enumerate() {
                        let final_column = j + offset;
                        if final_column < width {
                            out.push((final_column as u16, line, c));
                        }
                    }

                    line += 1;
                    current_line_text.clear();
                }

                current_line_text.push(ch);
            }

            // Output last line if not empty
            if !current_line_text.is_empty() && line < size.1 {
                let line_width = current_line_text.len();
                let offset = match align {
                    TextAlign::Left => 0,
                    TextAlign::Center => width.saturating_sub(line_width) / 2,
                    TextAlign::Right => width.saturating_sub(line_width),
                };

                for (j, &c) in current_line_text.iter().enumerate() {
                    let final_column = j + offset;
                    if final_column < width {
                        out.push((final_column as u16, line, c));
                    }
                }
            }
        }

    fn on_event(&mut self, event: crate::Event, actions: &mut crate::ActionList) {
            if let crate::Event::Resize(w, h) = event {
                self.changed_chars = (0..self.text.len()).collect();
            }
            if self.listener.is_listening_for(event.into()) {
                (self.listener.on_event)(event, self);
                actions.add(crate::Action::UpdateWidget);
            }
        }
    }
}

#[cfg(test)]
mod factory_widgets_tests {
    use super::text::*;
    use crate::{WidgetBuilder, WidgetPlacement, factory_widgets::Listener, test::*, *};

    #[test]
    fn adding_and_removing_text() {
        let mut changed = Vec::new();
        let mut text_box = TextBox::new("123", Listener::empty(), TextAlign::Left);
        text_box.append_char('4');
        text_box.append_text("567");
        text_box.get_changed_chars((16,1), &mut changed);
        assert_eq!(changed, (0..7).into_iter().map(|i| (i as u16, 0, char::from_digit(i + 1, 10).unwrap()) ).collect::<Vec<_>>());

        changed.drain(..);
        text_box.get_changed_chars((16,1), &mut changed);
        assert!(changed.is_empty());

        text_box.remove_text(text_box.text_len() - 3, text_box.text_len());
        text_box.get_changed_chars((16,1), &mut changed);
        assert_eq!(changed, (4..7).into_iter().map(|i| ( i as u16, 0, ' ') ).collect::<Vec<_>>());

        changed.drain(..);
        text_box.append_text("56");
        text_box.get_changed_chars((16,1), &mut changed);
        assert_eq!(changed, (4..6).into_iter().map(|i| (i as u16, 0, char::from_digit(i + 1, 10).unwrap()) ).collect::<Vec<_>>());
    }

    #[test]
    fn centered_text() {
        let mut output = TestOutput::<25, 1>::new();
        let mut text_box = WidgetBuilder::new(TextBox::new(
            "Centered Text",
            super::Listener::empty(),
            TextAlign::Center,
        ))
        .build();
        let placement = WidgetPlacement::fullscreen();
        let layout = Layout::new().with_widget(&text_box, placement);

        layout.render(25, 1).render_to_output(&mut output);
        let rendered_text = output.to_string();

        assert_match_with_test_file(&rendered_text, "factory_widgets/centered_text");
    }

    #[test]
    fn centered_mutliline_text() {
        let mut output = TestOutput::<25, 2>::new();
        let mut text_box = WidgetBuilder::new(TextBox::new(
            "Centered Text\nother",
            super::Listener::empty(),
            TextAlign::Center,
        ))
        .build();
        let placement = WidgetPlacement::fullscreen();
        let layout = Layout::new().with_widget(&text_box, placement);

        layout.render(25, 2).render_to_output(&mut output);
        let rendered_text = output.to_string();

        assert_match_with_test_file(&rendered_text, "factory_widgets/centered_multiline_text");
    }

    #[test]
    fn right_text() {
        let mut output = TestOutput::<25, 1>::new();
        let mut text_box = WidgetBuilder::new(TextBox::new(
            "Right Text",
            super::Listener::empty(),
            TextAlign::Right,
        ))
        .build();
        let placement = WidgetPlacement::fullscreen();
        let layout = Layout::new().with_widget(&text_box, placement);

        layout.render(25, 1).render_to_output(&mut output);
        let rendered_text = output.to_string();

        assert_match_with_test_file(&rendered_text, "factory_widgets/right_text");
    }

    #[test]
    fn right_multiline_text() {
        let mut output = TestOutput::<25, 2>::new();
        let mut text_box = WidgetBuilder::new(TextBox::new(
            "Right Text\nshort",
            super::Listener::empty(),
            TextAlign::Right,
        ))
        .build();
        let placement = WidgetPlacement::fullscreen();
        let layout = Layout::new().with_widget(&text_box, placement);

        layout.render(25, 2).render_to_output(&mut output);
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

        let mut progress_bar = WidgetBuilder::new(factory_widgets::progression::ProgressBar::new(
            factory_widgets::progression::ProgressBarType::HorizontalNineLevels,
            0.0,
            super::Listener::empty(),
        ))
        .build();

        let mut output = TestOutput::<2, 1>::new();
        let placement = WidgetPlacement::fullscreen();
        let layout = Layout::new()
            .with_widget(&progress_bar, placement)
            .render(2, 1);

        for step in 0..=16 {
            let amt = 1.0 - increment * (step as f32);
            progress_bar.edit().set_amt(amt);
            layout.render_to_output(&mut output);
            let rendered_text = output.to_string();
            eprintln!(
                "Step {}: amt={}, rendered={:?}, expected={:?}",
                step, amt, rendered_text, expected_output[step]
            );
            assert_eq!(rendered_text, expected_output[step]);
            output.clear();
        }
    }
}
