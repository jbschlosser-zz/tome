use std::vec::Vec;
use color_char::ColorChar;

#[derive(Debug, Eq, PartialEq)]
pub struct LineBuffer {
    lines: Vec<Vec<ColorChar>>,
    max_lines: Option<usize>,
    max_line_length: Option<usize>,
    line_index: usize
}

impl LineBuffer {
    pub fn new(max_lines: Option<usize>, max_line_length: Option<usize>) -> LineBuffer {
        let mut lines = Vec::new();
        lines.push(Vec::new());
        LineBuffer {
            lines: lines,
            max_lines: max_lines,
            max_line_length: max_line_length,
            line_index: 0
        }
    }
    pub fn insert(&mut self, data: &[ColorChar]) {
        for ch in data {
            match (ch.ch, self.max_line_length) {
                ('\r', _) => (),
                ('\n', _) => self.move_to_next_line(),
                (_, None) => self.lines[self.line_index].push(*ch),
                (_, Some(m)) => {
                    if self.lines[self.line_index].len() == m {
                        self.move_to_next_line();
                    }
                    self.lines[self.line_index].push(*ch);
                }
            }
        }
    }
    pub fn get_lines(&self, scrollback: usize, max_lines: usize) -> Vec<&[ColorChar]> {
        if scrollback >= self.lines.len() { return Vec::new() };
        let starting_index =
            if scrollback <= self.line_index
                {self.line_index - scrollback}
            else
                {self.lines.len() - (scrollback - self.line_index)};
        let num_lines = if max_lines <= (self.lines.len() - scrollback) {max_lines}
            else {self.lines.len() - scrollback};
        let mut lines = Vec::with_capacity(num_lines);
        let mut i = starting_index;
        while lines.len() < num_lines {
            lines.push(&self.lines[i][..]);
            i = if i == 0 {num_lines - 1} else {i - 1};
        }
        let mut lines_rev = Vec::with_capacity(num_lines); 
        while lines.len() > 0 {
            lines_rev.push(lines.pop().unwrap());
        }
        lines_rev
    }
    pub fn move_to_next_line(&mut self) {
        match self.max_lines {
            Some(s) if self.lines.len() == s => {
                self.line_index = (self.line_index + 1) % s;
                self.lines[self.line_index] = Vec::new();
            },
            _ => {
                self.lines.push(Vec::new());
                self.line_index += 1;
            }
        }
    }
}
