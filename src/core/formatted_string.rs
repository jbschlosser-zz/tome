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

#[derive(Debug, Eq, PartialEq)]
pub struct FormattedString {
    chars: String,
    formats: Vec<Format>
}

impl FormattedString {
    pub fn new() -> FormattedString {
        FormattedString { chars: String::new(), formats: Vec::new() }
    }
    pub fn with_format(chars: &str, format: Format) -> FormattedString {
        FormattedString { chars: String::from_str(chars),
            formats: vec![format; chars.len()]}
    }
    pub fn with_color(chars: &str, color: Color) -> FormattedString {
        FormattedString::with_format(chars, Format::with_fg(color))
    }
    pub fn push(&mut self, ch: char, format: Format) {
        self.chars.push(ch);
        self.formats.push(format);
    }
    pub fn insert(&mut self, index: usize, ch: char, format: Format) {
        self.chars.insert(index, ch);
        self.formats.insert(index, format);
    }
    pub fn remove(&mut self, index: usize) {
        self.chars.remove(index);
        self.formats.remove(index);
    }
    pub fn clear(&mut self) {
        self.chars.clear();
        self.formats.clear();
    }
    pub fn append(&mut self, other: &FormattedString) {
        self.chars.reserve(other.len());
        self.formats.reserve(other.len());
        for (ch, format) in other.iter() {
            self.push(ch, format);
        }
    }
    pub fn change_format(&mut self, index: usize, format: Format) {
        assert!(index < self.formats.len());
        self.formats[index] = format;
    }
    pub fn len(&self) -> usize { self.chars.len() }
    pub fn iter(&self) -> FormattedStringIterator {
        FormattedStringIterator::new(self)
    }
    pub fn to_str(&self) -> &str { &self.chars }
    // TODO: This is for testing at the moment. There's probably a better
    // way to get this functionality.
    pub fn formats(&self) -> &[Format] { &self.formats }
}

pub struct FormattedStringIterator<'a> {
    string: &'a FormattedString,
    curr: usize
}

impl<'a> FormattedStringIterator<'a> {
    pub fn new(string: &'a FormattedString) -> FormattedStringIterator {
        FormattedStringIterator { string: string, curr: 0 }
    }
}

impl<'a> Iterator for FormattedStringIterator<'a> {
    type Item = (char, Format);

    fn next(&mut self) -> Option<(char, Format)> {
        if self.curr < self.string.len() {
            self.curr += 1;
            Some((self.string.chars.char_at(self.curr - 1),
                self.string.formats[self.curr - 1]))
        } else { None }
    }
}
