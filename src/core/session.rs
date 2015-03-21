use formatted_string::{Format, Color, Style};
use line_buffer::LineBuffer;
use server_data::ParseState;

pub struct Session {
    pub telnet_state: ParseState,
    pub esc_seq_state: ParseState,
    pub char_format: Format,
    pub input_buf: (LineBuffer, usize),
    pub output_buf: (LineBuffer, usize)
}

impl Session {
    pub fn new() -> Session {
        Session {
            telnet_state: ParseState::NotInProgress,
            esc_seq_state: ParseState::NotInProgress,
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
