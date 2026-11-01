use crossterm::{
    execute, style::{Attribute, Attributes, Color, Print, ResetColor, SetAttribute, SetAttributes, SetBackgroundColor, SetForegroundColor},
};
use std::ops::BitOr;

#[derive(Default, Copy, Clone, PartialEq)]
pub struct CombinedFormat {
    foreground: Option<Color>,
    background: Option<Color>, // TODO: implement
    bold: u8,      // 0 = not specified, 1 = bold, 2 = not bold
    underline: u8, // 0 = not specified, 1 = underline, 2 = not underline
    italic: u8,    // 0 = not specified, 1 = italic, 2 = not italic
    dim: u8,       // 0 = not specified, 1 = dim, 2 = not dim
}

#[derive(Clone, Copy)]
pub enum Format {
    Attribute(Attribute),
    FrontColor(Color),
    BackColor(Color),
}

pub mod attributes {
    use super::*; 
    
    pub static BOLD: Format = Format::Attribute(Attribute::Bold);
    pub static UNDERLINE: Format = Format::Attribute(Attribute::Underlined);
    pub static ITALIC: Format = Format::Attribute(Attribute::Italic);
    pub static DIM: Format = Format::Attribute(Attribute::Dim);
    pub static NO_BOLD: Format = Format::Attribute(Attribute::NoBold);
    pub static NO_ITALIC: Format = Format::Attribute(Attribute::NoItalic);
    pub static NORMAL: Format = Format::Attribute(Attribute::NormalIntensity);
    pub static NO_UNDERLINE: Format = Format::Attribute(Attribute::NoUnderline);
}

pub mod colors {
    use super::*;

    pub mod front {
        use super::*;
        
        pub static BLACK: Format = Format::FrontColor(Color::Black);
        pub static DARKGREY: Format = Format::FrontColor(Color::DarkGrey);
        pub static RED: Format = Format::FrontColor(Color::Red);
        pub static DARKRED: Format = Format::FrontColor(Color::DarkRed);
        pub static GREEN: Format = Format::FrontColor(Color::Green);
        pub static DARKGREEN: Format = Format::FrontColor(Color::DarkGreen);
        pub static YELLOW: Format = Format::FrontColor(Color::Yellow);
        pub static DARKYELLOW: Format = Format::FrontColor(Color::DarkYellow);
        pub static BLUE: Format = Format::FrontColor(Color::Blue);
        pub static DARKBLUE: Format = Format::FrontColor(Color::DarkBlue);
        pub static MAGENTA: Format = Format::FrontColor(Color::Magenta);
        pub static DARKMAGENTA: Format = Format::FrontColor(Color::DarkMagenta);
        pub static CYAN: Format = Format::FrontColor(Color::Cyan);
        pub static DARKCYAN: Format = Format::FrontColor(Color::DarkCyan);
        pub static WHITE: Format = Format::FrontColor(Color::White);
        pub static GREY: Format = Format::FrontColor(Color::Grey);        
    }

    pub mod back {
        use super::*;
        
        pub static BLACK: Format = Format::BackColor(Color::Black);
        pub static DARKGREY: Format = Format::BackColor(Color::DarkGrey);
        pub static RED: Format = Format::BackColor(Color::Red);
        pub static DARKRED: Format = Format::BackColor(Color::DarkRed);
        pub static GREEN: Format = Format::BackColor(Color::Green);
        pub static DARKGREEN: Format = Format::BackColor(Color::DarkGreen);
        pub static YELLOW: Format = Format::BackColor(Color::Yellow);
        pub static DARKYELLOW: Format = Format::BackColor(Color::DarkYellow);
        pub static BLUE: Format = Format::BackColor(Color::Blue);
        pub static DARKBLUE: Format = Format::BackColor(Color::DarkBlue);
        pub static MAGENTA: Format = Format::BackColor(Color::Magenta);
        pub static DARKMAGENTA: Format = Format::BackColor(Color::DarkMagenta);
        pub static CYAN: Format = Format::BackColor(Color::Cyan);
        pub static DARKCYAN: Format = Format::BackColor(Color::DarkCyan);
        pub static WHITE: Format = Format::BackColor(Color::White);
        pub static GREY: Format = Format::BackColor(Color::Grey);        
    }
}

impl Format {
    fn combine(self) -> CombinedFormat {
        match self {
            Format::Attribute(attr) => match attr {
                Attribute::Bold => CombinedFormat {
                    bold: 1,
                    ..Default::default()
                },
                Attribute::Underlined => CombinedFormat {
                    underline: 1,
                    ..Default::default()
                },
                Attribute::Italic => CombinedFormat {
                    italic: 1,
                    ..Default::default()
                },
                Attribute::Dim => CombinedFormat {
                    dim: 1,
                    ..Default::default()
                },
                Attribute::NormalIntensity => CombinedFormat {
                    dim: 2,
                    bold: 2,
                    ..Default::default()
                },
                Attribute::NoItalic => CombinedFormat {
                    italic: 2,
                    ..Default::default()
                },
                Attribute::NoUnderline => CombinedFormat {
                    underline: 2,
                    ..Default::default()
                },
                _ => todo!(),
            },
            Format::FrontColor(color) => CombinedFormat {
                foreground: Some(color),
                ..Default::default()
            },
            Format::BackColor(color) => CombinedFormat {
                background: Some(color),
                ..Default::default()
            }
        }
    }
}

impl BitOr for Format {
    type Output = CombinedFormat;

    fn bitor(self, rhs: Format) -> Self::Output {
        self.combine() | rhs.combine()
    }
}

impl BitOr<Format> for CombinedFormat {
    type Output = CombinedFormat;

    fn bitor(self, rhs: Format) -> Self::Output {
        self | rhs.combine()
    }
}

impl BitOr<CombinedFormat> for CombinedFormat {
    type Output = CombinedFormat;

    fn bitor(mut self, rhs: CombinedFormat) -> Self::Output {
        self.bold = if rhs.bold != 0 { rhs.bold } else { self.bold };
        self.underline = if rhs.underline != 0 { rhs.underline } else { self.underline };
        self.italic = if rhs.italic != 0 { rhs.italic } else { self.italic };
        self.dim = if rhs.dim != 0 { rhs.dim } else { self.dim };
        self.foreground = rhs.foreground.or(self.foreground);
        self.background = rhs.background.or(self.background);
        self
    }
}

impl From<Format> for CombinedFormat {
    fn from(value: Format) -> Self {
        CombinedFormat::default() | value
    }
}

impl CombinedFormat {
    pub fn is_italic(&self) -> bool {
        self.italic == 1
    }
    
    pub fn is_bold(&self) -> bool {
        self.bold == 1
    }

    pub fn is_dim(&self) -> bool {
        self.dim == 1
    }

    pub fn is_underline(&self) -> bool {
        self.underline == 1
    }
    
    pub(crate) fn reset_global() {
        let _ = execute!(
            std::io::stdout(),
            SetAttribute(Attribute::Reset),
            ResetColor
        );
    }
    
    pub(crate) fn apply(&self) {
        if let Some(color) = self.foreground {
            let _ = execute!(
                std::io::stdout(),
                SetForegroundColor(color),
            );
        }
        if let Some(color) = self.background {
            let _ = execute!(
                std::io::stdout(),
                SetBackgroundColor(color),
            );
        }
        let bold = if self.bold == 1 { Attribute::Bold } else { Attribute::NoBold };
        let underline = if self.underline == 1 { Attribute::Underlined } else { Attribute::NoUnderline };
        let italic = if self.italic == 1 { Attribute::Italic } else { Attribute::NoItalic };

        let mut attributes = Attributes::default();
        attributes.set(bold);
        attributes.set(underline);
        attributes.set(italic);
        if self.dim == 1 {
            attributes.set(Attribute::Dim);
        }
        let _ = execute!(
            std::io::stdout(),
            SetAttributes(attributes),
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::text_formatting::attributes::*;
    use crate::text_formatting::colors::*;

    #[test]
    fn combination() {
        use super::*;
        let format = BOLD | front::RED | back::BLUE | UNDERLINE | ITALIC;

        assert_eq!(format.bold, 1);
        assert_eq!(format.underline, 1);
        assert_eq!(format.italic, 1);
        assert_eq!(format.foreground, Some(Color::Red));
        assert_eq!(format.background, Some(Color::Blue));
    }
}
