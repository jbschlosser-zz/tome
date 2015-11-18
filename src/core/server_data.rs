use formatted_string::{FormattedString, Style, Color};
use regex::Regex;
use session::Session;

#[derive(Debug, Eq, PartialEq)]
pub enum ParseState {
    NotInProgress,
    InProgress(Vec<u8>),
    Success(Vec<u8>),
    Error(Vec<u8>)
}

pub fn handle_server_data(data: &[u8], session: &mut Session) -> FormattedString {
    let mut out_str = FormattedString::new();
    for byte in data {
        // Apply the telnet layer.
        let new_telnet_state = parse_telnet(&session.telnet_state, *byte);
        match new_telnet_state {
            ParseState::NotInProgress => {
                // Apply the esc sequence layer.
                let new_esc_seq_state = parse_esc_seq(&session.esc_seq_state, *byte);
                match new_esc_seq_state {
                    ParseState::NotInProgress => {
                        // TODO: Properly convert to UTF-8.
                        out_str.push(*byte as char, session.char_format);
                    },
                    ParseState::InProgress(_) => (),
                    ParseState::Success(ref seq) => {
                        handle_esc_seq(&seq, session);
                    },
                    // TODO: Log the error.
                    ParseState::Error(ref bad_seq) => {}
                }
                session.esc_seq_state = new_esc_seq_state;
            },
            ParseState::InProgress(_) => (),
            ParseState::Success(ref cmd) => {
                handle_telnet_cmd(&cmd, session);
            },
            ParseState::Error(ref bad_cmd) => {}
        }
        session.telnet_state = new_telnet_state;
    }

    out_str
}

pub const TELNET_SE: u8 = 240; // End of subnegotiation parameters.
pub const TELNET_NOP: u8 = 241; // No operation.
pub const TELNET_DATA_MARK: u8 = 242; // The data stream portion of a Synch. This should always be accompanied by a TCP Urgent notification.
pub const TELNET_BREAK: u8 = 243; // NVT character BRK.
pub const TELNET_IP: u8 = 244; // The function IP (interrupt process).
pub const TELNET_AO: u8 = 245; // The function AO (abort output).
pub const TELNET_AYT: u8 = 246; // The function AYT (are you there).
pub const TELNET_EC: u8 = 247; // The function EC (erase character).
pub const TELNET_EL: u8 = 248; // The function EL (erase line).
pub const TELNET_GA: u8 = 249; // The GA (go ahead) signal.
pub const TELNET_SB: u8 = 250; // Indicates that what follows is subnegotiation of the indicated option.
pub const TELNET_WILL: u8 = 251; // Indicates the desire to begin performing, or confirmation that you are now performing, the indicated option.
pub const TELNET_WONT: u8 = 252; // Indicates the refusal to perform, or continue performing, the indicated option.
pub const TELNET_DO: u8 = 253; // Indicates the request that the other party perform, or
                      // confirmation that you are expecting the other party to perform, the
                      // indicated option.
pub const TELNET_DONT: u8 = 254; // Indicates the demand that the other party stop performing,
                        // or confirmation that you are no longer expecting the other party
                        // to perform, the indicated option.
pub const TELNET_IAC: u8 = 255; // Interpret As Command. Indicates the start of a telnet option
                       // negotiation.
 
pub const TELNET_MAX_COMMAND_SIZE: usize = 64;

pub fn handle_telnet_cmd(_: &[u8], _: &mut Session) {
    // TODO: Implement this.
}

pub fn parse_telnet(old_state: &ParseState, byte: u8) -> ParseState {
    match *old_state {
        ParseState::NotInProgress => {
            if byte == TELNET_IAC { ParseState::InProgress(vec![byte]) }
            else { ParseState::NotInProgress }
        },
        ParseState::InProgress(ref b) => {
            let mut bytes = b.clone();
            bytes.push(byte);

            // Check if the command has exceeded the max allowed size.
            if bytes.len() > TELNET_MAX_COMMAND_SIZE {
                // Boom. Ran out of space for the command :(
                return ParseState::Error(bytes);
            }

            // Determine the next state.
            if bytes.len() == 2 {
                match byte {
                    // Two byte commands.
                    TELNET_IAC|TELNET_NOP|TELNET_DATA_MARK|TELNET_BREAK|TELNET_IP|TELNET_AO|TELNET_AYT|TELNET_EC|TELNET_EL|TELNET_GA => {
                        return ParseState::Success(bytes);
                    },
                    // Three byte commands.
                    TELNET_WILL|TELNET_WONT|TELNET_DO|TELNET_DONT|TELNET_SB => (),
                    // Unknown command.
                    _ => return ParseState::Error(bytes)
                }
            } else if bytes.len() == 3 {
                let prev_byte = bytes[bytes.len() - 2];
                match prev_byte {
                    // Three byte commands.
                    TELNET_WILL|TELNET_WONT|TELNET_DO|TELNET_DONT => {
                        return ParseState::Success(bytes);
                    },
                    // Sub-negotiation can span an arbitrary number of bytes.
                    TELNET_SB => (),
                    // Unexpected command.
                    _ => return ParseState::Error(bytes)
                }
            } else if bytes.len() > 3 {
                // Sub-negotiation is assumed, since that is the only command
                // that can be this long. Check if the most recent bytes are
                // IAC,SE. This ends sub-negotiation.
                let prev_byte = bytes[bytes.len() - 2];
                if prev_byte == TELNET_IAC && byte == TELNET_SE {
                    return ParseState::Success(bytes);
                }
            }

            ParseState::InProgress(bytes)
        },
        ParseState::Success(_) => parse_telnet(&ParseState::NotInProgress, byte),
        ParseState::Error(_) => parse_telnet(&ParseState::NotInProgress, byte)
    }
}

pub const ESC_SEQUENCE_BEGIN: u8 = 0x1B;
pub const ESC_SEQUENCE_END: u8 = 0x6D;
pub const ESC_SEQUENCE_MAX_SIZE: usize = 15;

pub fn handle_esc_seq(seq: &[u8], session: &mut Session) {
    // Use the esc sequence to update the char format for the session.
    let (style, fg_color, bg_color) = interpret_esc_seq(seq);
    if let Some(s) = style {
        session.char_format.style = s;
    }
    if let Some(f) = fg_color {
        session.char_format.fg_color = f;
    }
    if let Some(b) = bg_color {
        session.char_format.bg_color = b;
    }
}

pub fn parse_esc_seq(old_state: &ParseState, byte: u8) -> ParseState {
    match *old_state {
        ParseState::NotInProgress => {
            if byte == ESC_SEQUENCE_BEGIN { ParseState::InProgress(vec![byte]) }
            else { ParseState::NotInProgress }
        },
        ParseState::InProgress(ref b) => {
            let mut bytes = b.clone();
            bytes.push(byte);

            // Check if the command has exceeded the max size.
            if bytes.len() > ESC_SEQUENCE_MAX_SIZE {
                return ParseState::Error(bytes);
            }

            if byte == ESC_SEQUENCE_END {
                return ParseState::Success(bytes);
            }

            ParseState::InProgress(bytes)
        },
        ParseState::Success(_) => parse_esc_seq(&ParseState::NotInProgress, byte),
        ParseState::Error(_) => parse_esc_seq(&ParseState::NotInProgress, byte)
    }
}

pub fn interpret_esc_seq(esc_seq: &[u8]) -> (Option<Style>, Option<Color>, Option<Color>) {
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
