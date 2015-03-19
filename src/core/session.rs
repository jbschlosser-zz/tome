use telnet::Telnet;
use esc_sequence::EscSequence;
use formatted_string::{FormattedString, Format, Color, Style};
use line_buffer::LineBuffer;

#[derive(Debug)]
pub struct Session {
    pub telnet: Telnet,
    pub esc_seq: EscSequence,
    pub char_format: Format,
    pub input_buf: (LineBuffer, usize),
    pub output_buf: (LineBuffer, usize)
}

impl Session {
    pub fn new() -> Session {
        Session {
            telnet: Telnet::new(),
            esc_seq: EscSequence::new(),
            char_format: Format {
                style: Style::Normal,
                fg_color: Color::Default,
                bg_color: Color::Default
            },
            input_buf: (LineBuffer::new(None, None), 0), // TODO: Change this.
            output_buf: (LineBuffer::new(None, None), 0) // TODO: Change this.
        }
    }
}

pub fn handle_server_data(data: &[u8], session: &mut Session) -> FormattedString {
    let mut out_str = FormattedString::new();
    for byte in data {
        match (session.telnet.update(*byte), session.telnet.in_progress()) {
            (Some(_), _) => (), //println!("telnet cmd: {:?}", cmd),
            (None, true) => (), // Telnet command in progress.
            (None, false) => {
                match (session.esc_seq.update(*byte),
                    session.esc_seq.in_progress()) {
                    (Some(seq), _) => {
                        // Set char format according to the escape sequence.
                        //println!("escape seq: {:?}", seq);
                        let format = EscSequence::parse(&seq);
                        match format.0 {
                            Some(style) => session.char_format.style = style,
                            None => ()
                        }
                        match format.1 {
                            Some(fg_color) => session.char_format.fg_color = fg_color,
                            None => ()
                        }
                        match format.2 {
                            Some(bg_color) => session.char_format.bg_color = bg_color,
                            None => ()
                        }
                    },
                    (None, true) => (), // Esc sequence in progress.
                    (None, false) => {
                        // TODO: Properly convert to UTF-8.
                        out_str.push(*byte as char, session.char_format);
                    }
                }
            }
        }
    }

    out_str
}
