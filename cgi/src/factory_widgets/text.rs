
use std::collections::HashSet;

use crate::{CursorMove, Displayable};

use super::*;

#[derive(Clone)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone)]
pub enum Wrapping {
    Off,
    PerWord,
    PerLetter,
}

#[derive(Clone)]
pub struct TextBox {
    text: Vec<char>,
    layout: Vec<u16>,
    line_breaks: Vec<usize>,
    eraser: Vec<(u16, u16)>,
    size: (u16, u16),
    current_length: usize,
    listener: Listener<Self>,
    align: TextAlign,
    wrapping: Wrapping,
    multiple_updates: bool,
    style: Option<crate::text_formatting::CombinedFormat>,
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

        Self {
            eraser: vec![],
            current_length: text.len(),
            text,
            size: (0, 0),
            listener,
            align,
            multiple_updates: false,
            layout: vec![0; 1], // Initialize with a single line
            line_breaks: vec![0],
            wrapping: Wrapping::PerLetter,
            style: None,
        }
    }

    pub fn text(&self) -> String {
        self.text[..self.current_length].iter().collect()
    }

    pub fn set_text(&mut self, text: &str) {
        self.multiple_updates = !self.eraser.is_empty();
        let new_text: Vec<char> = text.chars().collect();
        if new_text.len() > self.text.len() {
            self.text.resize(new_text.len(), ' ');
        }

        for (i, c) in new_text.iter().enumerate() {
            self.text[i] = *c;
        }

        self.current_length = new_text.len();

        self.recompute_layout();
    }

    pub fn append_text(&mut self, text: &str) {
        let added_len = text.chars().count();
        for (i, c) in text.chars().enumerate() {
            if self.current_length + i >= self.text.len() {
                self.text.push(c);
            } else {
                self.text[self.current_length + i] = c;
            }
        }

        self.current_length += added_len;
        self.recompute_layout_from(self.current_length - 1 - added_len);
    }

    pub fn pop_char(&mut self) -> char {
        self.current_length -= 1;
        let r = self.text.pop().unwrap();
        self.recompute_layout_from(self.current_length - 1);
        r
    }

    pub fn pop_char_at(&mut self, index: usize) -> char {
        let r = self.text.remove(index);
        self.current_length -= 1;
        self.recompute_layout_from(index - 1);
        r
    }

    pub fn remove_text(&mut self, start: usize, end: usize) {
        let drain = self.text.drain(..).collect::<Vec<char>>();
        let part_a = drain[..start].as_ref();
        let part_b = drain[end..].as_ref();
        self.text.extend_from_slice(part_a);
        self.text.extend_from_slice(part_b);
        self.current_length = self.text.len();
        self.recompute_layout_from(start - 1);
    }

    pub fn append_char(&mut self, c: char) {
        if self.current_length >= self.text.len() {
            self.text.push(c);
        } else {
            self.text[self.current_length] = c;
        }
        self.current_length += 1;
        self.recompute_layout_from(self.current_length - 2);
    }

    pub fn text_len(&self) -> usize {
        self.current_length
    }

    /// Returns the difference between two layouts as a vector of offsets.
    /// each line is b - a. so a + diff = b. diff > 0 means b is longer than a.
    /// The result is as long as the longer of the two layouts.
    fn layout_diff(&self, layout_a: &[u16], layout_b: &[u16]) -> Vec<i16> {
        let mut diff = Vec::new();

        let max_len = layout_a.len().max(layout_b.len());

        for i in 0..max_len {
            let a = layout_a
                .get(i)
                .map(|x| (*x).min(self.size.0) as i16)
                .unwrap_or(0);
            let b = layout_b
                .get(i)
                .map(|x| (*x).min(self.size.0) as i16)
                .unwrap_or(0);
            diff.push(b - a);
        }

        diff
    }
    //start must be a character that HAS NOT changed or 0 for full recompute
    fn recompute_layout_from(&mut self, start: usize) {
        if self.size.0 == 0 {
            return;
        }

        // If start == 0, full recompute
        let starting_place = if start == 0 {
            (0, 0, ' ')
        } else {
            self.get_char_placement(start).unwrap()
        };
        let mut line_start = 0;

        let old_layout = self
            .layout
            .drain((starting_place.1 as usize)..)
            .collect::<Vec<u16>>();

        self.line_breaks = self
            .text
            .iter()
            .take(self.current_length)
            .enumerate()
            .filter_map(|(i, c)| if *c == '\n' { Some(i) } else { None })
            .collect();

        // Only interested in line breaks after the starting place
        let new_line_breaks = &self.line_breaks[(starting_place.1 as usize)..];

        match self.wrapping {
            Wrapping::Off => {
                for &break_idx in new_line_breaks {
                    let line_len = break_idx.saturating_sub(line_start);
                    self.layout.push(line_len as u16);
                    line_start = break_idx + 1;
                }

                let remaining = self.current_length.saturating_sub(line_start);
                if self.layout.is_empty() || remaining > 0 {
                    self.layout.push(remaining as u16);
                }
            }
            Wrapping::PerWord => {
                let mut start = starting_place.0 as usize;

                for &break_idx in new_line_breaks {
                    let whitespaces = self.text[start..break_idx]
                        .iter()
                        .enumerate()
                        .filter_map(|(i, c)| {
                            if c.is_whitespace() {
                                Some(start + i)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    let mut is_last = false;

                    while start < break_idx {
                        let last_whitespace = Self::get_last_index_before_threshold(
                            &whitespaces,
                            self.size.0 as usize + start,
                        );
                        let last_whitespace = match last_whitespace {
                            Some(idx) => {
                                if idx == whitespaces.len() - 1 {
                                    is_last = true;
                                }
                                whitespaces[idx]
                            }
                            None => start,
                        };

                        let line_len = last_whitespace.saturating_sub(start);
                        self.layout.push(line_len as u16);
                        start = last_whitespace + 1;

                        if is_last {
                            let line_len = break_idx.saturating_sub(start);
                            self.layout.push(line_len as u16);
                            start = break_idx + 1;
                            break;
                        }
                    }
                }

                let remaining = self.current_length.saturating_sub(start);
                if self.layout.is_empty() || remaining > 0 {
                    self.layout.push(remaining as u16);
                }
            }
            Wrapping::PerLetter => {
                for &break_idx in new_line_breaks {
                    let mut line_len = break_idx.saturating_sub(line_start) as u16;

                    while line_len >= self.size.0 {
                        line_len -= self.size.0;
                        self.layout.push(self.size.0 as u16);
                    }

                    self.layout.push(line_len as u16);
                    line_start = break_idx + 1;
                }
                let remaining = self.current_length.saturating_sub(line_start);
                if remaining > 0 {
                    self.layout.push(remaining as u16);
                }
            }
        }

        // Computing where we need to add ' ' characters to erase what was there before
        let layout_diff = self.layout_diff(&old_layout, &self.layout);
        for (line_idx, (line_diff, old_line_width)) in
            layout_diff.iter().zip(old_layout.iter()).enumerate()
        {
            // if line_diff is negative, then the line was longer before than after the layout change.
            // we need to erase the characters that were there before the layout change
            if *line_diff < 0 {
                let line_diff: u16 = (-line_diff) as u16;
                let old_line_width: u16 = (*old_line_width).min(self.size.0);
                for i in 0..line_diff {
                    let position = match self.align {
                        TextAlign::Left => old_line_width - i - 1,
                        TextAlign::Center => {
                            let new_line_width = *self.layout.get(line_idx).unwrap_or(&0);
                            let offset_l = (self.size.0 - new_line_width) / 2;
                            let offset_r = offset_l + (self.size.0 - new_line_width) % 2;
                            if i % 2 == 0 {
                                offset_l - i / 2 - 1
                            } else {
                                self.size.0 - offset_r + i / 2
                            }
                        }
                        TextAlign::Right => self.size.0 - old_line_width + i,
                    };
                    self.eraser.push((position, line_idx as u16));
                }
            }
        }
    }

    fn recompute_layout(&mut self) {
        self.recompute_layout_from(0)
    }

    /// Returns the last index of the slice that is before the threshold
    /// If every index is higher, returns None
    /// If every index is lower, then the return value will be == indexes.len() - 1
    fn get_last_index_before_threshold(indexes: &[usize], threshold: usize) -> Option<usize> {
        let last_expected_idx = indexes.binary_search(&threshold);
        match last_expected_idx {
            Ok(idx) => Some(idx),
            // idx is the position where target would be inserted
            Err(idx) => {
                if idx == 0 {
                    None
                } else {
                    Some(idx - 1)
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
                let last_break_idx =
                    Self::get_last_index_before_threshold(&self.line_breaks, index);
                let (expected_line, line_start) = match last_break_idx {
                    Some(break_idx) => (break_idx + 1, self.line_breaks[break_idx] + 1),
                    None => (0, 0),
                };
                let index_on_line = index.saturating_sub(line_start);

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
            Wrapping::PerWord => {
                for (line, &line_width) in self.layout.iter().enumerate() {
                    let mut line_end = line_start + line_width;
                    while self.text[line_end as usize] == ' ' {
                        line_end += 1;
                    }
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

    pub fn set_style<T: Into<crate::text_formatting::CombinedFormat> + Copy>(&mut self, style: T) {
        if self.style != Some(style.into()) {
            self.style = Some(style.into());
        }
    }

    pub fn with_wrapping_mode(mut self, wrapping: Wrapping) -> Self {
        self.set_wrapping_mode(wrapping);
        self
    }

    pub fn with_align(mut self, align: TextAlign) -> Self {
        self.set_align(align);
        self
    }

    pub fn with_style<T: Into<crate::text_formatting::CombinedFormat> + Copy>(
        mut self,
        style: T,
    ) -> Self {
        self.set_style(style);
        self
    }
}

//TODO: add styles
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
        self.text_box.recompute_layout();
        self.cursor += 1;
    }

    pub fn remove_text(&mut self, start: usize, end: usize) {
        if start >= end || end > self.text_box.text_len() {
            return;
        }

        self.text_box.remove_text(start, end);

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
                let previous_storage_len = self.text_box.text.len();
                let new_len = self.text_box.text_len().min(max_length);
                self.text_box.text.truncate(new_len);
                self.text_box.text.resize(previous_storage_len, ' ');
                self.text_box.current_length = new_len;
                self.cursor = self.cursor.min(new_len);
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
    fn get_chars(&mut self) -> crate::CharContainer<'_> {
        if self.layout.is_empty() {
            self.recompute_layout();
        }
        if self.size.0 * self.size.1 == 0 {
            return crate::CharContainer::new_empty();
        }

        const NO_WRAPPING_POINTS: u16 = 3;

        let mut width_overflow_line_idx = Vec::new();
        let mut chars = (0..self.current_length)
            .filter_map(|i| self.get_char_placement(i))
            .filter(|(x, y, _)| {
                if *x == self.size.0 {
                    if *y < self.size.1 {
                        width_overflow_line_idx.push(*y);
                    }
                    false
                } else {
                    *x < self.size.0 && *y < self.size.1
                }
            })
            .collect::<Vec<_>>();

        // TODO
        if let Wrapping::Off = self.wrapping {
            if width_overflow_line_idx.len() > 0 {
                crate::log::log(&format!("s{} {:?}", self.size.1, &width_overflow_line_idx));
            }
            for y in width_overflow_line_idx {
                for i in 0..NO_WRAPPING_POINTS.min(self.size.0) {
                    chars.push((self.size.0 - 1 - i, y, '.'));
                }
            }
        }

        // To check if we are not erasing what we plan on rendering
        let mut eraser = if self.multiple_updates {
            let lookup = chars
                .iter()
                .map(|(x, y, _)| ((*x as u32) << 16) | *y as u32)
                .collect::<HashSet<_>>();
            self.eraser
                .drain(..)
                .filter(|(x, y)| {
                    let key = ((*x as u32) << 16) | *y as u32;
                    !lookup.contains(&key) && *y < self.size.1
                })
                .map(|(x, y)| (x, y, ' '))
                .collect::<Vec<_>>()
        } else {
            self.eraser
                .drain(..)
                .map(|(x, y)| (x, y, ' '))
                .collect::<Vec<_>>()
        };

        chars.append(&mut eraser);

        crate::CharContainer::new_owned(chars)
    }

    fn on_event(&mut self, event: crate::Event, actions: &mut crate::ActionList) {
        if let crate::Event::Resize(x, y) = event {
            self.size = (x, y);
            self.recompute_layout();
        }
        if self.listener.is_listening_for(event.into()) {
            (self.listener.on_event)(event, actions, self);
        }
    }
}

impl Displayable for TextInput {
    fn get_chars(&mut self) -> crate::CharContainer<'_> {
        self.text_box.get_chars()
    }

    fn on_event(&mut self, event: crate::Event, actions: &mut crate::ActionList) {
        let mut should_update = false;

        match event {
            crate::Event::Resize(..) => {
                self.text_box.on_event(event, actions);
                should_update = true;
            }

            crate::Event::WidgetFocused => {
                actions.add(crate::Action::ShowCursor);
                actions.add(crate::Action::SetCursorStyle(
                    crate::CursorStyle::BlinkingUnderScore,
                ));
                crate::log::log("FOCUS");
            }
            crate::Event::FocusLost => {
                actions.add(crate::Action::HideCursor);
                actions.add(crate::Action::SetCursorStyle(
                    crate::CursorStyle::SteadyBlock,
                ));
            }
            crate::Event::KeyPress(code) => match code {
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
            },
            _ => (),
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
