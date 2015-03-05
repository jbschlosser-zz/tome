use telnet::Telnet;
use esc_sequence::EscSequence;
use color_char::Style;
use color_char::Color;
use color_char::Attributes;
use line_buffer::LineBuffer;

#[derive(Debug)]
pub struct Session {
    pub telnet: Telnet,
    pub esc_seq: EscSequence,
    pub attrs: Attributes,
    pub output: (LineBuffer, usize)
}

impl Session {
    pub fn new() -> Session {
        Session {
            telnet: Telnet::new(),
            esc_seq: EscSequence::new(),
            attrs: Attributes {
                style: Style::Normal,
                fg_color: Color::Default,
                bg_color: Color::Default
            },
            output: (LineBuffer::new(None), 0) // TODO: Change this.
        }
    }
}

use color_char::ColorChar;
pub fn handle_server_data(data: &[u8], session: &mut Session) -> Vec<ColorChar> {
    let mut chars = Vec::new();
    for byte in data {
        match (session.telnet.update(*byte), session.telnet.in_progress()) {
            (Some(_), _) => (), //println!("telnet cmd: {:?}", cmd),
            (None, true) => (), // Telnet command in progress.
            (None, false) => {
                match (session.esc_seq.update(*byte),
                    session.esc_seq.in_progress()) {
                    (Some(seq), _) => {
                        // Set attributes according to the escape sequence.
                        //println!("escape seq: {:?}", seq);
                        let attrs = EscSequence::parse(&seq);
                        match attrs.0 {
                            Some(style) => session.attrs.style = style,
                            None => ()
                        }
                        match attrs.1 {
                            Some(fg_color) => session.attrs.fg_color = fg_color,
                            None => ()
                        }
                        match attrs.2 {
                            Some(bg_color) => session.attrs.bg_color = bg_color,
                            None => ()
                        }
                    },
                    (None, true) => (), // Esc sequence in progress.
                    (None, false) => {
                        // TODO: Properly convert to UTF-8.
                        chars.push(ColorChar::new(*byte as char, session.attrs));
                    }
                }
            }
        }
    }

    chars
}
