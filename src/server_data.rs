extern crate regex;

use session::Session;
use std::io::Write;
use tome::{esc_seq, telnet, FormattedString, ParseState};

pub fn handle_server_data(data: &[u8], session: &mut Session) -> FormattedString {
    let mut out_str = FormattedString::new();
    for byte in data {
        // Apply the telnet layer.
        let new_telnet_state = telnet::parse(&session.telnet_state, *byte);
        match new_telnet_state {
            ParseState::NotInProgress => {
                // Apply the esc sequence layer.
                let new_esc_seq_state =
                    esc_seq::parse(&session.esc_seq_state, *byte);
                match new_esc_seq_state {
                    ParseState::NotInProgress => {
                        // TODO: Properly convert to UTF-8.
                        out_str.push((*byte as char, session.char_format));
                    },
                    ParseState::InProgress(_) => (),
                    ParseState::Success(ref seq) => {
                        handle_esc_seq(&seq, session);
                    },
                    ParseState::Error(ref bad_seq) => {
                        warn!("Bad escape sequence encountered: {:?}", bad_seq);
                    }
                }
                session.esc_seq_state = new_esc_seq_state;
            },
            ParseState::InProgress(_) => (),
            ParseState::Success(ref cmd) => {
                info!("Telnet command encountered: {:?}", cmd);
                handle_telnet_cmd(&cmd, session);
            },
            ParseState::Error(ref bad_cmd) => {
                warn!("Bad telnet command encountered: {:?}", bad_cmd);
            }
        }
        session.telnet_state = new_telnet_state;
    }

    out_str
}

pub fn handle_telnet_cmd(cmd: &[u8], session: &mut Session) {
    // TODO: Implement this.
    if cmd.len() == 3 && &cmd[..3] == &[telnet::IAC, telnet::WILL, telnet::GMCP] {
        info!("IAC WILL GMCP received");
        session.connection.write(&[telnet::IAC, telnet::DO, telnet::GMCP]);
    }

    if cmd.len() > 3 && &cmd[..3] == &[telnet::IAC, telnet::SB, telnet::GMCP] {
        let mid: Vec<u8> = (&cmd[3..cmd.len() - 2])
            .iter()
            .map(|b| *b)
            .collect();
        let mid_str = match String::from_utf8(mid) {
            Ok(m) => m,
            Err(_) => return
        };
        info!("Received GMCP message: {}", &mid_str);
    }
}

pub fn handle_esc_seq(seq: &[u8], session: &mut Session) {
    // Use the esc sequence to update the char format for the session.
    let (style, fg_color, bg_color) = esc_seq::interpret(seq);
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
