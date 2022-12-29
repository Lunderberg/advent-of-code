use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Color {
    Default,
    Red,
    BrightRed,
    Green,
    BrightGreen,
    Yellow,
    BrightYellow,
    Blue,
    BrightBlue,
    Magenta,
    BrightMagenta,
    Cyan,
    BrightCyan,
    RGB(u8, u8, u8),
}

pub struct Highlighted<'a, T> {
    color: Color,
    inner: &'a T,
}

impl Default for Color {
    fn default() -> Self {
        Color::Default
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::Default => Ok(()),
            Color::Red => write!(f, "\x1b[0;31m"),
            Color::BrightRed => write!(f, "\x1b[0;31m"),
            Color::Green => write!(f, "\x1b[0;32m"),
            Color::BrightGreen => write!(f, "\x1b[1;32m"),
            Color::Yellow => write!(f, "\x1b[0;33m"),
            Color::BrightYellow => write!(f, "\x1b[1;33m"),
            Color::Blue => write!(f, "\x1b[0;34m"),
            Color::BrightBlue => write!(f, "\x1b[1;34m"),
            Color::Magenta => write!(f, "\x1b[0;35m"),
            Color::BrightMagenta => write!(f, "\x1b[1;35m"),
            Color::Cyan => write!(f, "\x1b[0;36m"),
            Color::BrightCyan => write!(f, "\x1b[1;36m"),
            Color::RGB(r, g, b) => write!(f, "\x1b[38;2;{r};{g};{b}m"),
        }
    }
}

impl Color {
    pub fn disable(&self) -> &'static str {
        match self {
            Color::Default => "",
            _ => "\x1b[0m",
        }
    }

    pub fn highlight<T>(self, inner: &T) -> Highlighted<'_, T> {
        Highlighted { color: self, inner }
    }
}

impl<'a, T> Display for Highlighted<'a, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.color, self.inner, self.color.disable())
    }
}
