use super::*;
use crate::{Displayable, symbols::progress_bar::{bar, block}};

#[derive(Clone, Copy)]
pub enum ProgressBarType {
    HorizontalNineLevels,
    HorizontalThreeLevels,
    VerticalNineLevels,
    VerticalThreeLevels,
}

#[derive(Clone)]
pub struct ProgressBar {
    bar_type: ProgressBarType,
    amt: f32,
    listener: Listener<Self>,
    color: Option<crate::text_formatting::CombinedFormat>,
    axis_len: u16,
}

impl ProgressBar {
    pub fn new(bar_type: ProgressBarType, amt: f32, listener: Listener<Self>) -> Self {
        Self {
            bar_type,
            amt,
            listener,
            color: None,
            axis_len: 0,
        }
    }

    pub fn set_amt(&mut self, amt: f32) {
        self.amt = amt;
    }

    pub fn set_color(&mut self, color: crate::text_formatting::Format) {
        self.color = Some(crate::text_formatting::CombinedFormat::from(color));
    }

    fn scaled_amt(&self) -> f32 {
        self.amt.clamp(0.0, 1.0) * self.axis_len as f32
    }

    fn get_set(&self) -> crate::symbols::progress_bar::Set {
        match self.bar_type {
            ProgressBarType::HorizontalNineLevels => block::NINE_LEVELS,
            ProgressBarType::HorizontalThreeLevels => block::THREE_LEVELS,
            ProgressBarType::VerticalNineLevels => bar::NINE_LEVELS,
            ProgressBarType::VerticalThreeLevels => bar::THREE_LEVELS,
        }
    }

    fn is_horizontal(&self) -> bool {
        matches!(
            self.bar_type,
            ProgressBarType::HorizontalNineLevels | ProgressBarType::HorizontalThreeLevels
        )
    }
}

impl Displayable for ProgressBar {
    fn get_chars(&mut self) -> crate::CharContainer<'_> {
        let mut r = crate::CharContainer::new_empty();
        if let Some(color) = self.color {
            r.set_format(color);
        }
        let scaled = self.scaled_amt();
        let filled = scaled.floor() as usize;
        let partial_fill = scaled.fract();
        let set = self.get_set();
        let chars = (0..self.axis_len)
            .map(|i| {
                let ch = if (i as usize) < filled {
                    set.full
                } else if (i as usize) == filled && partial_fill > 0.0 {
                    set.get_level(partial_fill)
                } else {
                    ' '
                };
                if self.is_horizontal() {
                    (i, 0, ch)
                } else {
                    (0, i, ch)
                }
            })
            .collect();
        r.add_owned(chars);

        r
    }

    fn on_event(&mut self, event: crate::Event, actions: &mut crate::ActionList) {
        if let crate::Event::Resize(x, y) = event {
            self.axis_len = if self.is_horizontal() { x } else { y };
        }

        if self.listener.is_listening_for(event.into()) {
            (self.listener.on_event)(event, actions, self);
        }
    }
}
