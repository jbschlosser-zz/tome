use formatted_string::{Style, Color};
use parse_state::ParseState;
use regex::Regex;

const SEQ_BEGIN: u8 = 0x1B;
const SEQ_END: u8 = 0x6D;
const SEQ_MAX_SIZE: usize = 15;

pub fn parse(old_state: &ParseState, byte: u8) -> ParseState {
    match *old_state {
        ParseState::NotInProgress => {
            if byte == SEQ_BEGIN { ParseState::InProgress(vec![byte]) }
            else { ParseState::NotInProgress }
        },
        ParseState::InProgress(ref b) => {
            let mut bytes = b.clone();
            bytes.push(byte);

            // Check if the command has exceeded the max size.
            if bytes.len() > SEQ_MAX_SIZE {
                return ParseState::Error(bytes);
            }

            if byte == SEQ_END {
                return ParseState::Success(bytes);
            }

            ParseState::InProgress(bytes)
        },
        ParseState::Success(_) => parse(&ParseState::NotInProgress, byte),
        ParseState::Error(_) => parse(&ParseState::NotInProgress, byte)
    }
}

pub fn interpret(esc_seq: &[u8]) -> (Option<Style>, Option<Color>, Option<Color>) {
    let seq_str = match ::std::str::from_utf8(esc_seq) {
        Ok(s) => s,
        Err(_) => return (None, None, None)
    };
    let full_regex =
        Regex::new(r"^\x{1B}\[([0-9;]*)m$").unwrap();
    let inside = match full_regex.captures(seq_str) {
        Some(caps) => caps.at(1),
        None => None
    };
    let mut parts = Vec::new();
    match inside {
        // Special case for ESC[m.
        Some("") => return
            (Some(Style::Normal), Some(Color::Default), Some(Color::Default)),
        Some(ins) => {
            for part in ins.split(";") {
                if part.len() > 0 { parts.push(part); }
            }
        }
        None => return (None, None, None)
    }

    // Extract the attributes from the escape sequence.
    let mut style = None;
    let mut fg_color = None;
    let mut bg_color = None;
    for part in parts {
        match part {
            "0" => {
                style = Some(Style::Normal);
                fg_color = Some(Color::Default);
                bg_color = Some(Color::Default);
            },
            "1" => style = Some(Style::Bold),
            "30" => fg_color = Some(Color::Black),
            "31" => fg_color = Some(Color::Red),
            "32" => fg_color = Some(Color::Green),
            "33" => fg_color = Some(Color::Yellow),
            "34" => fg_color = Some(Color::Blue),
            "35" => fg_color = Some(Color::Magenta),
            "36" => fg_color = Some(Color::Cyan),
            "37" => fg_color = Some(Color::White),
            "39" => fg_color = Some(Color::Default),
            "40" => bg_color = Some(Color::Black),
            "41" => bg_color = Some(Color::Red),
            "42" => bg_color = Some(Color::Green),
            "43" => bg_color = Some(Color::Yellow),
            "44" => bg_color = Some(Color::Blue),
            "45" => bg_color = Some(Color::Magenta),
            "46" => bg_color = Some(Color::Cyan),
            "47" => bg_color = Some(Color::White),
            "49" => bg_color = Some(Color::Default),
            _ => () // Ignore unknown parts.
        }
    }

    return (style, fg_color, bg_color);
}
