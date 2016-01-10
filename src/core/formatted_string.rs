#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Style {
    Normal,
    Bold,
    Standout
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Color {
    Default,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Format {
    pub style: Style,
    pub fg_color: Color,
    pub bg_color: Color
}

impl Format {
    pub fn default() -> Format {
        Format { style: Style::Normal, fg_color: Color::Default,
            bg_color: Color::Default }
    }
    pub fn with_fg(color: Color) -> Format {
        Format { style: Style::Normal, fg_color: color,
            bg_color: Color::Default }
    }
}

pub type FormattedString = Vec<(char, Format)>;

pub fn with_format(s: &str, format: Format) -> FormattedString {
    // TODO: clean this up.
	let mut string = String::new();
    string.push_str(s);
    let mut fs = FormattedString::new();
    for ch in string.chars() {
        fs.push((ch, format));
    }
    fs
}

pub fn with_color(s: &str, color: Color) -> FormattedString {
    with_format(s, Format::with_fg(color))
}

pub fn to_string(fs: &FormattedString) -> String {
    let string: String = fs.iter().map(|&(ch, _)| ch).collect();
    string
}
