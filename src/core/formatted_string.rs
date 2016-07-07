#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum Style {
    Normal,
    Bold,
    Standout
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
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

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
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

pub fn from_markup(s: &str) -> FormattedString {
    let mut fs = Vec::new();
    let mut format = Format::default();
    let mut escape = false;
    for c in s.chars() {
        if escape {
            match c {
                '{' => fs.push((c, format)),
                'x' => format = Format::default(),
                'b' => format.fg_color = Color::Black,
                'r' => format.fg_color = Color::Red,
                'g' => format.fg_color = Color::Green,
                'y' => format.fg_color = Color::Yellow,
                'u' => format.fg_color = Color::Blue,
                'm' => format.fg_color = Color::Magenta,
                'c' => format.fg_color = Color::Cyan,
                'w' => format.fg_color = Color::White,
                'h' => format.style = Style::Standout,
                _ => () // Unknown escape sequence.
            }
            escape = false;
        } else if c == '{' {
            escape = true;
        } else {
            fs.push((c, format));
        }
    }
    fs
}

pub fn to_string(fs: &FormattedString) -> String {
    let string: String = fs.iter().map(|&(ch, _)| ch).collect();
    string
}
