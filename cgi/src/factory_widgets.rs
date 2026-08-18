use std::collections::BTreeSet;

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
    use std::collections::{BTreeMap, HashMap};

    use super::*;

    pub enum TextAlign {
        Left,
        Center,
        Right,
    }

    pub enum Wrapping {
        Off,
        PerWord,
        PerLetter,
    }

    pub struct TextBox {
        text: Vec<char>,
        layout: Vec<u16>,
        line_breaks: Vec<usize>,
        changed_chars: BTreeSet<usize>, // points to chars in the text
        size: (u16, u16),               // Remove ?
        current_length: usize,
        listener: Listener<Self>,
        align: TextAlign,
        wrapping: Wrapping,
    }

    impl TextBox {
        pub fn new(text: &str, listener: Listener<Self>, align: TextAlign) -> Self {
            let text: Vec<char> = text.chars().collect();
            let changed_chars: BTreeSet<usize> = (0..text.len()).collect();

            Self {
                current_length: text.len(),
                text,
                changed_chars,
                size: (0, 0),
                listener,
                align,
                layout: vec![0; 1], // Initialize with a single line
                line_breaks: vec![0],
                wrapping: Wrapping::PerLetter,
            }
        }

        pub fn text(&self) -> String {
            self.text.iter().collect()
        }

        pub fn set_text(&mut self, text: &str) {
            for (i, c) in text.chars().enumerate() {
                if self.text[i] != c {
                    self.text[i] = c;
                    self.changed_chars.insert(i);
                }
            }

            for i in text.len()..self.text.len() {
                self.text[i] = ' ';
                self.changed_chars.insert(i);
            }
        }

        pub fn append_text(&mut self, text: &str) {
            for (i, c) in text.chars().enumerate() {
                self.changed_chars.insert(self.current_length + i);
                if self.current_length + i >= self.text.len() {
                    self.text.push(c);
                } else {
                    self.text[self.current_length + i] = c;
                }
            }

            self.recompute_layout_from(self.current_length);
            self.current_length += text.len();
        }

        pub fn append_char(&mut self, c: char) {
            if self.current_length >= self.text.len() {
                self.text.push(c);
            } else {
                self.text[self.current_length] = c;
            }
            self.changed_chars.insert(self.current_length);
            self.recompute_layout_from(self.current_length);
            self.current_length += 1;
        }

        pub fn remove_text(&mut self, start: usize, end: usize) {
            if start >= end || end > self.text.len() {
                return;
            }
            for i in start..end {
                self.text[i] = ' ';
                self.changed_chars.insert(i);
                self.recompute_layout_from(start);
            }
            if end == self.current_length {
                self.current_length = start;
            }
        }

        pub fn text_len(&self) -> usize {
            self.current_length
        }

        fn recompute_layout_from(&mut self, _start: usize) {
            self.recompute_layout();
        }

        fn recompute_layout(&mut self) {
            self.layout.clear();

            let mut line_width = 0;

            self.line_breaks = self
                .text
                .iter()
                .enumerate()
                .filter_map(|(i, c)| if *c == '\n' { Some(i) } else { None })
                .collect();

            match self.wrapping {
                Wrapping::Off => {
                    let mut layout = vec![*self.line_breaks.first().unwrap() as u16];
                    layout.append(&mut self
                        .line_breaks
                        .windows(2)
                        .map(|pair| (pair[1] - pair[0] - 1) as u16)
                        .collect());
                    layout.push((self.current_length - *self.line_breaks.last().unwrap() - 1) as u16);
                    self.layout = layout;
                }
                Wrapping::PerWord => {}
                Wrapping::PerLetter => {
                    if self.size.0 == 0 {
                        return;
                    }
                    // TODO: use the line_breaks to avoid recomputing
                    for &c in &self.text {
                        if c == '\n' {
                            self.layout.push(line_width);
                            line_width = 0;
                        } else {
                            if line_width >= self.size.0 {
                                self.layout.push(self.size.0);
                                line_width = 0;
                            }
                            line_width += 1;
                        }
                    }
                    if line_width > 0 || self.layout.is_empty() {
                        self.layout.push(line_width);
                    }
                }
            }
        }

        // TODO: every call to this function is quite expensive, precompute all text stuff before calling the function many times
        fn get_char_placement(&self, index: usize) -> Option<(u16, u16, char)> {
            if index >= self.text.len() || self.text[index] == '\n' {
                return None;
            }

            let visual_index = self.text[..index].iter().filter(|&&c| c != '\n').count() as u16;
            let mut line_start = 0;
            let character = self
                .text
                .iter()
                .filter(|&&c| c != '\n')
                .nth(visual_index as usize)
                .copied();

            if character.is_none() {
                return None;
            }
            let character = character.unwrap();

            match self.wrapping {
                Wrapping::Off => {
                    let last = self
                        .line_breaks
                        .iter()
                        .enumerate()
                        .take_while(|(_, i)| **i < index)
                        .last();

                    let (expected_line, last_break) = match last {
                        Some((idx, break_point)) => (1 + idx as usize, *break_point as usize),
                        None => (0, 0),
                    };
                    let mut index_on_line = index - last_break;
                    if last_break > 0 {
                        index_on_line -= 1;
                    }
                    // Letting one character overflow so that we know if the text is too wide.
                    // The character will be clipped anyway
                    if index_on_line > 1 + self.size.0 as usize {
                        return None;
                    }
                    let line_width = self.layout[expected_line].clamp(0, self.size.0);
                    let offset = match self.align {
                        TextAlign::Left => 0,
                        TextAlign::Center => self.size.0.saturating_sub(line_width as u16) / 2,
                        TextAlign::Right => self.size.0.saturating_sub(line_width as u16),
                    };

                    return Some((
                        offset + index_on_line as u16,
                        expected_line as u16,
                        character,
                    ));
                }
                Wrapping::PerWord => todo!(),
                Wrapping::PerLetter => {
                    for (line, &line_width) in self.layout.iter().enumerate() {
                        let line_end = line_start + line_width;
                        if visual_index < line_end {
                            let x = visual_index - line_start;
                            let offset = match self.align {
                                TextAlign::Left => 0,
                                TextAlign::Center => self.size.0.saturating_sub(line_width) / 2,
                                TextAlign::Right => self.size.0.saturating_sub(line_width),
                            };
                            return Some((x + offset, line as u16, character));
                        }
                        line_start = line_end;
                    }
                }
            }

            None
        }

        pub fn set_wrapping_mode(&mut self, wrapping: Wrapping) {
            self.wrapping = wrapping;
            self.recompute_layout();
        }

        pub fn set_align(&mut self, align: TextAlign) {
            self.align = align;
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

            const NO_WRAPPING_POINTS: u16 = 3;

            let mut width_overflow_line_idx = Vec::new();
            let mut changes: BTreeMap<(u16, u16), char> = self
                .changed_chars
                .iter()
                .filter_map(|i| self.get_char_placement(*i))
                .filter(|(x, y, _)| {
                    if *x == size.0 {
                        width_overflow_line_idx.push(*y);
                        false
                    } else {
                        *x < size.0 && *y < size.1
                    }
                })
                .map(|(x, y, c)| ((x, y), c))
                .collect();

            if let Wrapping::Off = self.wrapping {
                for y in width_overflow_line_idx {
                    for i in 0..NO_WRAPPING_POINTS {
                        changes.insert((size.0 - 1 - i, y), '.');
                    }
                }
            }

            self.changed_chars.clear();

            // changes.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1))); // sort by position
            // changes.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1); // filter for duplicate positions

            out.append(
                &mut changes
                    .into_iter()
                    .map(|(pos, c)| (pos.0, pos.1, c))
                    .collect::<Vec<_>>(),
            );
        }

        fn on_event(&mut self, event: crate::Event, actions: &mut crate::ActionList) {
            if let crate::Event::Resize(w, h) = event {
                self.size = (w, h);
                self.recompute_layout();
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

        text_box.on_event(Event::Resize(16, 1), &mut ActionList::new());

        text_box.append_char('4');
        text_box.append_text("567");
        text_box.get_changed_chars((16, 1), &mut changed);
        assert_eq!(
            changed,
            (0..7)
                .into_iter()
                .map(|i| (i as u16, 0, char::from_digit(i + 1, 10).unwrap()))
                .collect::<Vec<_>>()
        );

        changed.drain(..);
        text_box.get_changed_chars((16, 1), &mut changed);
        assert!(changed.is_empty());

        text_box.remove_text(text_box.text_len() - 3, text_box.text_len());
        text_box.get_changed_chars((16, 1), &mut changed);
        assert_eq!(
            changed,
            (4..7).into_iter().map(|i| (i, 0, ' ')).collect::<Vec<_>>()
        );

        changed.drain(..);
        text_box.append_text("56");
        text_box.get_changed_chars((16, 1), &mut changed);
        assert_eq!(
            changed,
            (4..6)
                .into_iter()
                .map(|i| (i as u16, 0, char::from_digit(i + 1, 10).unwrap()))
                .collect::<Vec<_>>()
        );

        changed.drain(..);
        text_box.append_text("\n789xx");
        text_box.remove_text(text_box.text_len() - 1, text_box.text_len());
        text_box.remove_text(text_box.text_len() - 1, text_box.text_len());

        text_box.get_changed_chars((16, 2), &mut changed);
        assert_eq!(
            changed,
            vec![
                (0, 1, '7'),
                (1, 1, '8'),
                (2, 1, '9'),
                (3, 1, ' '),
                (4, 1, ' '),
            ]
        );
    }

    #[test]
    fn adding_and_removing_text_2() {
        let mut output = TestOutput::<16, 1>::new();
        let mut text_box = WidgetBuilder::new(TextBox::new(
            "123",
            super::Listener::empty(),
            TextAlign::Left,
        ))
        .build();

        let placement = WidgetPlacement::fullscreen();
        let layout = Layout::new().with_widget(&text_box, placement);

        {
            let mut edit = text_box.edit();
            edit.on_event(Event::Resize(16, 1), &mut ActionList::new());
            edit.append_char('4');
            edit.append_text("567");
        }

        let rendered_layout = layout.render(16, 1);
        for _ in 0..3 {
            {
                let mut edit = text_box.edit();
                let len = edit.text_len();
                edit.remove_text(len - 1, len);
            }
            rendered_layout.render_to_output(&mut output);
        }

        for _ in 0..3 {
            let mut edit = text_box.edit();
            edit.on_event(Event::Resize(16, 1), &mut ActionList::new());
            edit.append_char('x');
        }
        rendered_layout.render_to_output(&mut output);
        let rendered_text = output.to_string();

        assert_eq!(rendered_text, "1234xxx         ");
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
        text_box
            .edit()
            .on_event(Event::Resize(25, 2), &mut ActionList::new());

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
        text_box
            .edit()
            .on_event(Event::Resize(25, 2), &mut ActionList::new());
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
        text_box
            .edit()
            .on_event(Event::Resize(25, 2), &mut ActionList::new());
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
        text_box
            .edit()
            .on_event(Event::Resize(25, 2), &mut ActionList::new());
        let placement = WidgetPlacement::fullscreen();
        let layout = Layout::new().with_widget(&text_box, placement);

        layout.render(25, 2).render_to_output(&mut output);
        let rendered_text = output.to_string();

        assert_match_with_test_file(&rendered_text, "factory_widgets/right_multiline_text");
    }

    #[test]
    fn short_text() {
        let text_box = crate::factory_widgets::text::TextBox::new(
            self::test::strings::lorem_ipsum_short(),
            Listener::empty(),
            factory_widgets::text::TextAlign::Left,
        );
        let widget = WidgetBuilder::new(text_box)
            .with_outline(symbols::OutlineStyle::Thick)
            .build();

        let rendered_text = crate::test::get_single_widget_rendered_text(&widget, (16, 8));
        println!("{}", rendered_text);
        crate::test::assert_match_with_test_file(
            &rendered_text,
            "factory_widgets/short_text_letter_wrap.txt",
        );
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
        let widget = WidgetBuilder::new(text_box)
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
        todo!("Test with wrapping in all three alignment modes")
    }

    #[test]
    fn wrapping_word_text() {
        todo!("Test with wrapping in all three alignment modes")
    }
}
