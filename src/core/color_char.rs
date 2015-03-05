#[derive(Debug, Eq, PartialEq, Copy)]
pub enum Style {
    Normal,
    Bold,
    Standout
}

#[derive(Debug, Eq, PartialEq, Copy)]
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

#[derive(Debug, Eq, PartialEq, Copy)]
pub struct Attributes {
    pub style: Style,
    pub fg_color: Color,
    pub bg_color: Color
}

#[derive(Debug, Eq, PartialEq, Copy)]
pub struct ColorChar {
    pub ch: char,
    pub attrs: Attributes
}

impl ColorChar {
    pub fn new(ch: char, attrs: Attributes) -> ColorChar {
        ColorChar { ch: ch, attrs: attrs }
    }
}

pub fn make_color_string(s: &str, attrs: Attributes) -> Vec<ColorChar> {
    let mut color_str = Vec::new();
    for ch in s.chars() {
        color_str.push(ColorChar::new(ch, attrs));
    }
    color_str
}
