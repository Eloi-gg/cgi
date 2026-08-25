use std::collections::BTreeSet;

use crate::{Displayable, EventType};

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

pub mod progression {
    use super::*;
    use crate::symbols::progress_bar::{bar, block};

    pub enum ProgressBarType {
        HorizontalNineLevels,
        HorizontalThreeLevels,
        VerticalNineLevels,
        VerticalThreeLevels,
    }

    pub struct ProgressBar {
        bar_type: ProgressBarType,
        changed_chars: Vec<(u16, u16, char)>,
        current_num_chars: usize,
        amt: f32,
        size: u16,
        listener: Listener<Self>,
    }

    impl ProgressBar {
        pub fn new(bar_type: ProgressBarType, amt: f32, listener: Listener<Self>) -> Self {
            Self {
                bar_type,
                changed_chars: Vec::new(),
                current_num_chars: 0,
                amt,
                size: 0,
                listener,
            }
        }

        pub fn set_amt(&mut self, amt: f32) {
            self.amt = amt;
            self.compute_changed_chars();
        }

        fn scaled_amt(&self, amt: f32) -> f32 {
            amt.clamp(0.0, 1.0) * self.size as f32
        }

        fn get_set(&self) -> crate::symbols::progress_bar::Set {
            match self.bar_type {
                ProgressBarType::HorizontalNineLevels => block::NINE_LEVELS,
                ProgressBarType::HorizontalThreeLevels => block::THREE_LEVELS,
                ProgressBarType::VerticalNineLevels => bar::NINE_LEVELS,
                ProgressBarType::VerticalThreeLevels => bar::THREE_LEVELS,
            }
        }

        fn compute_changed_chars(&mut self) {
            let scaled = self.scaled_amt(self.amt);

            let new_num_chars = scaled.ceil() as usize;
            let filled = scaled.floor() as usize;
            let partial_fill = scaled.fract();

            let range = if new_num_chars < self.current_num_chars {
                new_num_chars..(self.current_num_chars + 1)
            } else {
                self.current_num_chars..(new_num_chars + 1)
            };

            self.changed_chars.clear();
            self.current_num_chars = new_num_chars;

            let set = self.get_set();

            for i in range {
                let ch = if i < filled {
                    set.full
                } else if i == filled && partial_fill > 0.0 {
                    set.get_level(partial_fill)
                } else {
                    ' '
                };

                // Avoid producing coordinates outside the current sized axis
                if (i as u16) >= self.size {
                    continue;
                }

                if self.is_horizontal() {
                    self.changed_chars.push((i as u16, 0, ch));
                } else {
                    self.changed_chars.push((0, i as u16, ch));
                }
            }
        }

        fn full_recompute(&mut self) {
            self.current_num_chars = 0;
            self.compute_changed_chars();
        }

        fn is_horizontal(&self) -> bool {
            matches!(
                self.bar_type,
                ProgressBarType::HorizontalNineLevels | ProgressBarType::HorizontalThreeLevels
            )
        }
    }

    impl Displayable for ProgressBar {
        fn display(&self) {
            // No-op: rendering is handled via get_changed_chars.
        }

        fn name(&self) -> String {
            "ProgressBar".to_string()
        }

        fn get_changed_chars(&mut self, size: (u16, u16)) -> std::borrow::Cow<'_, [(u16, u16, char)]> {
            if size.0 * size.1 == 0 {
                return std::borrow::Cow::Borrowed(&[]);
            }

            // Update internal size based on requested widget size and recompute the bar characters.
            let axis_len = if self.is_horizontal() { size.0 } else { size.1 };
            if self.size != axis_len {
                self.size = axis_len;
            }
            // Recompute characters from current amt and size.
            self.full_recompute();

            std::borrow::Cow::Borrowed(&self.changed_chars)
        }

        fn on_event(&mut self, event: crate::Event, actions: &mut crate::ActionList) {
            if let crate::Event::Resize(w, h) = event {
                let axis_len = match self.bar_type {
                    ProgressBarType::HorizontalNineLevels => w,
                    ProgressBarType::HorizontalThreeLevels => w,
                    ProgressBarType::VerticalNineLevels => h,
                    ProgressBarType::VerticalThreeLevels => h,
                };
                self.size = axis_len;
            }
            if self.listener.is_listening_for(event.into()) {
                (self.listener.on_event)(event, actions, self);
            }
        }
    }
}

pub mod text {
    use std::collections::{BTreeMap, HashMap};

    use crate::CursorMove;

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

    pub struct TextInput {
        text_box: TextBox,
        cursor: usize,
        max_length: Option<usize>,
        cursor_visible: bool,
        listener: Listener<Self>,
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
            let new_text: Vec<char> = text.chars().collect();
            if new_text.len() > self.text.len() {
                self.text.resize(new_text.len(), ' ');
            }
            for (i, c) in new_text.iter().enumerate() {
                if self.text[i] != *c {
                    self.text[i] = *c;
                    self.changed_chars.insert(i);
                }
            }

            self.current_length = new_text.len();
            for i in new_text.len()..self.text.len() {
                self.text[i] = ' ';
                self.changed_chars.insert(i);
            }
            self.recompute_layout();
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
                    if self.size.0 == 0 {
                        self.layout.clear();
                        return;
                    }

                    let mut layout = Vec::new();
                    let mut start = 0usize;

                    for &break_idx in &self.line_breaks {
                        let line_len = break_idx.saturating_sub(start);
                        layout.push(line_len as u16);
                        start = break_idx + 1;
                    }

                    let remaining = self.current_length.saturating_sub(start);
                    if !layout.is_empty() || remaining > 0 {
                        layout.push(remaining as u16);
                    }

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
                    let expected_line = self
                        .line_breaks
                        .iter()
                        .take_while(|&&break_point| break_point < index)
                        .count();
                    let last_break = self
                        .line_breaks
                        .iter()
                        .take_while(|&&break_point| break_point < index)
                        .last()
                        .copied()
                        .unwrap_or(0usize);
                    let line_start = if last_break == 0 {
                        0usize
                    } else {
                        last_break + 1
                    };
                    let index_on_line = index - line_start;

                    if index_on_line > self.size.0 as usize {
                        return None;
                    }

                    let line_width = self
                        .layout
                        .get(expected_line)
                        .copied()
                        .unwrap_or(0)
                        .min(self.size.0);
                    let offset = match self.align {
                        TextAlign::Left => 0,
                        TextAlign::Center => self.size.0.saturating_sub(line_width) / 2,
                        TextAlign::Right => self.size.0.saturating_sub(line_width),
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

    impl TextInput {
        pub fn new(text: &str, listener: Listener<Self>, align: TextAlign) -> Self {
            let mut inner = TextBox::new(text, Listener::empty(), align);
            inner.set_wrapping_mode(Wrapping::Off);
            Self {
                cursor: inner.text_len(),
                max_length: None,
                cursor_visible: true,
                listener,
                text_box: inner,
            }
        }

        pub fn text(&self) -> String {
            self.text_box.text()
        }

        pub fn text_len(&self) -> usize {
            self.text_box.text_len()
        }

        pub fn set_text(&mut self, text: &str) {
            self.text_box.set_text(text);
            self.cursor = self.cursor.min(self.text_box.text_len());
        }

        pub fn append_text(&mut self, text: &str) {
            for ch in text.chars() {
                self.insert_char(ch);
            }
        }

        pub fn append_char(&mut self, ch: char) {
            self.insert_char(ch);
        }

        pub fn insert_char(&mut self, ch: char) {
            if ch.is_control() && ch != '\n' {
                return;
            }

            if let Some(max_length) = self.max_length {
                if self.text_box.text_len() >= max_length {
                    return;
                }
            }

            self.text_box.text.insert(self.cursor, ch);
            self.text_box.current_length += 1;
            self.text_box.changed_chars = (0..self.text_box.current_length).collect();
            self.text_box.recompute_layout();
            self.cursor += 1;
        }

        pub fn remove_text(&mut self, start: usize, end: usize) {
            if start >= end || end > self.text_box.text_len() {
                return;
            }

            self.text_box.text.drain(start..end);
            self.text_box.current_length = self.text_box.text.len();
            self.text_box.changed_chars = (0..self.text_box.current_length).collect();
            self.text_box.recompute_layout();

            if self.cursor > end {
                self.cursor -= end - start;
            } else if self.cursor > start {
                self.cursor = start;
            }
        }

        pub fn remove_char_before_cursor(&mut self) {
            if self.cursor == 0 {
                return;
            }
            let previous_cursor = self.cursor;
            self.remove_text(self.cursor - 1, self.cursor);
            self.cursor = previous_cursor.saturating_sub(1);
        }

        pub fn remove_char_after_cursor(&mut self) {
            if self.cursor >= self.text_box.text_len() {
                return;
            }
            self.remove_text(self.cursor, self.cursor + 1);
        }

        pub fn move_cursor_left(&mut self, amount: usize) {
            self.cursor = self.cursor.saturating_sub(amount);
        }

        pub fn move_cursor_right(&mut self, amount: usize) {
            self.cursor = (self.cursor + amount).min(self.text_box.text_len());
        }

        pub fn set_cursor(&mut self, index: usize) {
            self.cursor = index.min(self.text_box.text_len());
        }

        pub fn cursor(&self) -> usize {
            self.cursor
        }

        pub fn set_max_length(&mut self, max_length: Option<usize>) {
            self.max_length = max_length;
            if let Some(max_length) = max_length {
                if self.text_box.text_len() > max_length {
                    let new_len = self.text_box.text_len().min(max_length);
                    self.text_box.text.truncate(new_len);
                    self.text_box.current_length = new_len;
                    self.cursor = self.cursor.min(new_len);
                    self.text_box.changed_chars = (0..new_len).collect();
                    self.text_box.recompute_layout();
                }
            }
        }

        pub fn set_align(&mut self, align: TextAlign) {
            self.text_box.set_align(align);
        }

        pub fn set_wrapping_mode(&mut self, wrapping: Wrapping) {
            self.text_box.set_wrapping_mode(wrapping);
        }

        pub fn set_cursor_visible(&mut self, visible: bool) {
            self.cursor_visible = visible;
        }

        fn cursor_position(&self, size: (u16, u16)) -> Option<(u16, u16)> {
            let total_len = self.text_box.text_len();
            if total_len == 0 {
                return Some((0, 0));
            }

            let cursor_index = self.cursor.min(total_len);
            if cursor_index == total_len {
                let last_visible = self
                    .text_box
                    .text
                    .iter()
                    .enumerate()
                    .filter(|(_, ch)| **ch != '\n')
                    .last();
                match last_visible {
                    Some((idx, _)) => {
                        let pos = self.text_box.get_char_placement(idx)?;
                        let (x, y, _) = pos;
                        Some((
                            x.saturating_add(1).min(size.0.saturating_sub(1)),
                            y.min(size.1.saturating_sub(1)),
                        ))
                    }
                    None => Some((0, 0)),
                }
            } else {
                let (x, y, _) = self.text_box.get_char_placement(cursor_index)?;
                Some((
                    x.min(size.0.saturating_sub(1)),
                    y.min(size.1.saturating_sub(1)),
                ))
            }
        }
    }

    impl Default for TextBox {
        fn default() -> Self {
            Self::new("", Listener::empty(), TextAlign::Left)
        }
    }

    impl Default for TextInput {
        fn default() -> Self {
            Self::new("", Listener::empty(), TextAlign::Left)
        }
    }

    impl Displayable for TextBox {
        fn display(&self) {
            // Rendering is handled by `get_changed_chars` during layout pass.
        }

        fn name(&self) -> String {
            "TextBox".to_string()
        }

        fn get_changed_chars(&mut self, size: (u16, u16)) -> std::borrow::Cow<'_, [(u16, u16, char)]> {
            if size.0 * size.1 == 0 {
                return std::borrow::Cow::Borrowed(&[]);
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

            // Build an owned Vec for the output and return it via Cow::Owned so callers can take ownership if needed.
            let out_vec = changes
                .into_iter()
                .map(|(pos, c)| (pos.0, pos.1, c))
                .collect::<Vec<_>>();

            std::borrow::Cow::Owned(out_vec)
        }

        fn on_event(&mut self, event: crate::Event, actions: &mut crate::ActionList) {
            crate::log::log(&format!("on event {:?}", event));

            if let crate::Event::Resize(w, h) = event {
                self.size = (w, h);
                self.recompute_layout();
                self.changed_chars = (0..self.text.len()).collect();
            }
            if self.listener.is_listening_for(event.into()) {
                (self.listener.on_event)(event, actions, self);
            }
        }
    }

    impl Displayable for TextInput {
        fn display(&self) {
            // Rendering is handled by `get_changed_chars` during layout pass.
        }

        fn name(&self) -> String {
            "TextInput".to_string()
        }

        fn get_changed_chars(&mut self, size: (u16, u16)) -> std::borrow::Cow<'_, [(u16, u16, char)]> {
            if size.0 * size.1 == 0 {
                return std::borrow::Cow::Borrowed(&[]);
            }

            // Delegate to the inner TextBox which will return a Cow.
            self.text_box.get_changed_chars(size)
        }

        fn on_event(&mut self, event: crate::Event, actions: &mut crate::ActionList) {
            let mut should_update = false;

            if let crate::Event::Resize(w, h) = event {
                self.text_box.size = (w, h);
                self.text_box.recompute_layout();
                should_update = true;
            }

            if let crate::Event::KeyPress(code) = event {
                match code {
                    crate::KeyCode::Backspace => {
                        self.remove_char_before_cursor();
                        should_update = true;
                    }
                    crate::KeyCode::Delete => {
                        self.remove_char_after_cursor();
                        should_update = true;
                    }
                    crate::KeyCode::Left => {
                        self.move_cursor_left(1);
                        should_update = true;
                    }
                    crate::KeyCode::Right => {
                        self.move_cursor_right(1);
                        should_update = true;
                    }
                    crate::KeyCode::Home => {
                        self.set_cursor(0);
                        should_update = true;
                    }
                    crate::KeyCode::End => {
                        self.set_cursor(self.text_box.text_len());
                        should_update = true;
                    }
                    crate::KeyCode::Enter => {
                        self.insert_char('\n');
                        should_update = true;
                    }
                    crate::KeyCode::Char(ch) if !ch.is_control() => {
                        self.insert_char(ch);
                        should_update = true;
                    }
                    _ => {}
                }
            }

            if self.listener.is_listening_for(event.into()) {
                (self.listener.on_event)(event, actions, self);
                should_update = true;
            }

            if should_update {
                if let Some((cx, cy)) = self.cursor_position(self.text_box.size) {
                    actions.add(crate::Action::MoveCursor(CursorMove::ToRelativeToWidget(
                        cx, cy,
                    )));
                }
                actions.add(crate::Action::RedrawWidget);
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
        let mut changed = Vec::<(u16, u16, char)>::new();
        let mut text_box = TextBox::new("123", Listener::empty(), TextAlign::Left);

        text_box.on_event(Event::Resize(16, 1), &mut ActionList::new());

        text_box.append_char('4');
        text_box.append_text("567");
        changed = text_box.get_changed_chars((16, 1)).into_owned();
        assert_eq!(
            changed,
            (0..7)
                .into_iter()
                .map(|i| (i as u16, 0, char::from_digit(i + 1, 10).unwrap()))
                .collect::<Vec<_>>()
        );

        changed.drain(..);
        changed = text_box.get_changed_chars((16, 1)).into_owned();
        assert!(changed.is_empty());

        text_box.remove_text(text_box.text_len() - 3, text_box.text_len());
        changed = text_box.get_changed_chars((16, 1)).into_owned();
        assert_eq!(
            changed,
            (4..7).into_iter().map(|i| (i, 0, ' ')).collect::<Vec<_>>()
        );

        changed.drain(..);
        text_box.append_text("56");
        changed = text_box.get_changed_chars((16, 1)).into_owned();
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

        changed = text_box.get_changed_chars((16, 2)).into_owned();
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
        let widget = WidgetBuilder::new(text_box)
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
        let rendered_text = crate::test::get_single_widget_rendered_text(&widget, (16, 8));
        // println!()
        todo!("Test with wrapping in all three alignment modes")
    }

    #[test]
    fn wrapping_word_text() {
        todo!("Test with wrapping in all three alignment modes")
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
    }

    #[test]
    fn progress_bar_horizontal_and_vertical() {
        let mut bar = crate::factory_widgets::progression::ProgressBar::new(
            crate::factory_widgets::progression::ProgressBarType::HorizontalNineLevels,
            0.625,
            Listener::empty(),
        );
        let mut out = bar.get_changed_chars((4, 1)).into_owned();
        assert_eq!(
            out,
            vec![(0, 0, '█'), (1, 0, '█'), (2, 0, '▌'), (3, 0, ' ')],
            "horizontal bar"
        );

        let mut vertical = crate::factory_widgets::progression::ProgressBar::new(
            crate::factory_widgets::progression::ProgressBarType::VerticalNineLevels,
            0.75,
            Listener::empty(),
        );
        let mut out = vertical.get_changed_chars((1, 4)).into_owned();
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

        layout.connect_and_add_widgets(&mut widgets, placements.as_mut_slice());
        layout.render(17, 7).render_to_output(&mut output);
        let rendered_text = output.to_string();

        println!("{}", rendered_text);
        assert_match_with_test_file(&rendered_text, "factory_widgets/cells_divided_tb");
    }
}
