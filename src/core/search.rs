use formatted_string::{self, FormattedString};
use regex::Regex;
use ring_buffer::RingBuffer;
use std::error::Error;

#[derive(Copy, Clone)]
pub struct SearchResult {
    pub line_number: usize,
    pub begin_index: usize,
    pub end_index: usize
}

pub fn search_buffer(buffer: &RingBuffer<FormattedString>, search_str: &str,
                     starting_line: usize) -> Result<Option<SearchResult>, String>
{
    // Compile the regex.
    let regex = match Regex::new(search_str) {
        Ok(r) => r,
        Err(e) => return Err(e.description().to_string())
    };

    // Search through the buffer.
    for i in starting_line..buffer.len() {
        let line = formatted_string::to_string(buffer.get_recent(i));
        match regex.find(&line) {
            Some((start, end)) => {
                return Ok(Some(SearchResult {
                    line_number: i,
                    begin_index: start,
                    end_index: end
                }))
            },
            None => ()
        }
    }

    Ok(None)
}
