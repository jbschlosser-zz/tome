use std::cmp;
use std::slice::{Iter, IterMut};
use std::iter::Chain;
use std::vec::Vec;
use formatted_string::{FormattedString, Format};

#[derive(Debug, Eq, PartialEq)]
pub struct LineBuffer {
    lines: Vec<FormattedString>,
    max_lines: Option<usize>,
    max_line_length: Option<usize>,
    line_index: usize
}

impl LineBuffer {
    pub fn new(max_lines: Option<usize>, max_line_length: Option<usize>)
        -> Self
    {
        LineBuffer {
            lines: vec![FormattedString::new()],
            max_lines: max_lines,
            max_line_length: max_line_length,
            line_index: 0
        }
    }
    pub fn len(&self) -> usize { self.lines.len() }
    pub fn push(&mut self, data: &FormattedString) {
        for (ch, format) in data.iter() {
            self.push_single(ch, format);
        }
    }
    pub fn push_single(&mut self, ch: char, format: Format) {
        match (ch, self.max_line_length) {
            ('\r', _) => (),
            ('\n', _) => self.move_to_next_line(),
            (_, None) => self.lines[self.line_index].push(ch, format),
            (_, Some(m)) => {
                if self.lines[self.line_index].len() == m {
                    self.move_to_next_line();
                }
                self.lines[self.line_index].push(ch, format);
            }
        }
    }
    pub fn get_line(&self, scroll: usize) -> &FormattedString {
        let mut sb = scroll;
        if sb >= self.len() {
            sb = self.len() - 1;
        }
        
        if sb <= self.line_index {
            &self.lines[self.line_index - sb]
        } else {
            &self.lines[self.line_index + self.len() - sb]
        }
    }
    pub fn get_line_mut(&mut self, scroll: usize) -> &mut FormattedString {
        let mut sb = scroll;
        if sb >= self.len() {
            sb = self.len() - 1;
        }
        
        let curr = self.line_index;
        if sb <= self.line_index {
            &mut self.lines[curr - sb]
        } else {
            let len = self.len();
            &mut self.lines[curr + len - sb]
        }
    }
    pub fn get_lines(&self, scrollback: usize, max_lines: usize)
        -> Vec<&FormattedString>
    {
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
            lines.push(&self.lines[i]);
            i = if i == 0 {num_lines - 1} else {i - 1};
        }
        let mut lines_rev = Vec::with_capacity(num_lines); 
        while lines.len() > 0 {
            lines_rev.push(lines.pop().unwrap());
        }
        lines_rev
    }
    pub fn iter<'a>(&'a self, start_line: usize) ->
        Chain<Iter<'a, FormattedString>, Iter<'a, FormattedString>>
    {
        let (second, first) = self.lines.split_at(self.line_index + 1);
		let first_skip = cmp::min(start_line, first.len());
		let second_skip =
            if start_line > first.len() {
                cmp::min(start_line - first.len(), second.len())
            } else { 0 };
        (&first[first_skip..]).iter().chain((&second[second_skip..]).iter())
    }
    pub fn iter_mut<'a>(&'a mut self, start_line: usize) ->
        Chain<IterMut<'a, FormattedString>, IterMut<'a, FormattedString>>
    {
        let (second, first) = self.lines.split_at_mut(self.line_index + 1);
		let first_skip = cmp::min(start_line, first.len());
		let second_skip =
            if start_line > first.len() {
                cmp::min(start_line - first.len(), second.len())
            } else { 0 };
        (&mut first[first_skip..]).iter_mut()
            .chain((&mut second[second_skip..]).iter_mut())
    }
    pub fn scrollback<'a>(&'a self, scroll: usize) ->
        Chain<Iter<'a, FormattedString>, Iter<'a, FormattedString>>
    {
        let start_line =
            if scroll >= self.len() {
                // Scrolled back too far- stop at the beginning.
                0
            } else {
                self.len() - scroll - 1
            };
        self.iter(start_line)
    }
    pub fn scrollback_mut<'a>(&'a mut self, scroll: usize) ->
        Chain<IterMut<'a, FormattedString>, IterMut<'a, FormattedString>>
    {
        let start_line =
            if scroll >= self.len() {
                // Scrolled back too far- stop at the beginning.
                0
            } else {
                self.len() - scroll - 1
            };
        self.iter_mut(start_line)
    }
    pub fn move_to_next_line(&mut self) {
        match self.max_lines {
            Some(s) if self.lines.len() == s => {
                self.line_index = (self.line_index + 1) % s;
                self.lines[self.line_index] = FormattedString::new();
            },
            _ => {
                self.lines.push(FormattedString::new());
                self.line_index += 1;
            }
        }
    }
}

#[test]
fn test_line_buffer_iter_no_wrap() {
    let mut buffer = LineBuffer::new(Some(4), None);
    buffer.push(&FormattedString::with_format(
        "abc\ndef\nghi\n", Format::default()));

    let mut iter = buffer.iter(0);
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("abc", Format::default()));
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("def", Format::default()));
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("ghi", Format::default()));
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("", Format::default()));
    assert_eq!(iter.next(), None);

    iter = buffer.iter(1);
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("def", Format::default()));
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("ghi", Format::default()));
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("", Format::default()));
    assert_eq!(iter.next(), None);

    iter = buffer.iter(2);
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("ghi", Format::default()));
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("", Format::default()));
    assert_eq!(iter.next(), None);

    iter = buffer.iter(3);
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("", Format::default()));
    assert_eq!(iter.next(), None);

    iter = buffer.iter(4);
    assert_eq!(iter.next(), None);
}

#[test]
fn test_line_buffer_iter_with_wrap() {
    // Buffer with wrapping.
    let mut buffer = LineBuffer::new(Some(3), None);
    buffer.push(&FormattedString::with_format(
        "abc\ndef\nghi\njkl", Format::default()));

    let mut iter = buffer.iter(0);
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("def", Format::default()));
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("ghi", Format::default()));
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("jkl", Format::default()));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_line_buffer_scrollback_no_wrap() {
    let mut buffer = LineBuffer::new(Some(4), None);
    buffer.push(&FormattedString::with_format(
        "abc\ndef\nghi\n", Format::default()));

    let mut iter = buffer.scrollback(0);
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("", Format::default()));
    assert_eq!(iter.next(), None);

    iter = buffer.scrollback(1);
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("ghi", Format::default()));
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("", Format::default()));
    assert_eq!(iter.next(), None);

    iter = buffer.scrollback(2);
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("def", Format::default()));
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("ghi", Format::default()));
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("", Format::default()));
    assert_eq!(iter.next(), None);

    iter = buffer.scrollback(3);
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("abc", Format::default()));
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("def", Format::default()));
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("ghi", Format::default()));
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("", Format::default()));
    assert_eq!(iter.next(), None);

    iter = buffer.scrollback(4);
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("abc", Format::default()));
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("def", Format::default()));
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("ghi", Format::default()));
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("", Format::default()));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_line_buffer_scrollback_with_wrap() {
    // Buffer with wrapping.
    let mut buffer = LineBuffer::new(Some(3), None);
    buffer.push(&FormattedString::with_format(
        "abc\ndef\nghi\njkl", Format::default()));

    let mut iter = buffer.scrollback(1);
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("ghi", Format::default()));
    assert_eq!(iter.next().unwrap(),
        &FormattedString::with_format("jkl", Format::default()));
    assert_eq!(iter.next(), None);
}
