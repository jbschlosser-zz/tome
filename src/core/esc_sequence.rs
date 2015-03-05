extern crate regex;

use color_char::Style;
use color_char::Color;
use self::regex::Regex;

pub const ESC_SEQUENCE_BEGIN: u8 = 0x1B;
pub const ESC_SEQUENCE_END: u8 = 0x6D;
pub const ESC_SEQUENCE_MAX_SIZE: usize = 15;

#[derive(Debug)]
pub struct EscSequence {
    cmd: Vec<u8>
}

impl EscSequence {
    pub fn new() -> EscSequence {
        EscSequence { cmd: Vec::new() }
    }
    pub fn update(&mut self, byte: u8) -> Option<String> {
        // No sequence starts until the start byte is seen.
        if self.cmd.len() == 0 && byte != ESC_SEQUENCE_BEGIN {
            return None;
        }

        // Check if the command has reached the max allowed size.
        if self.cmd.len() == ESC_SEQUENCE_MAX_SIZE {
            // Boom. Ran out of space for the command :(
            self.cmd.clear();
            return None;
        }

        // Add the byte to the command.
        self.cmd.push(byte);

        // Check for the end of the sequence.
        if byte == ESC_SEQUENCE_END {
            let res = match String::from_utf8(self.cmd.clone()) {
                Ok(s) => Some(s),
                Err(_) => None
            };

            self.cmd.clear();
            return res;
        }

        return None;
    }
    pub fn in_progress(&self) -> bool {
        self.cmd.len() > 0
    }
    pub fn parse(seq: &str) -> (Option<Style>, Option<Color>, Option<Color>) {
        let full_regex =
            Regex::new(r"^\x{1B}\[([0-9;]*)m$").unwrap();
        let inside = match full_regex.captures(seq) {
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
}
