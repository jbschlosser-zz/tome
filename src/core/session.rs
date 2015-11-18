use formatted_string::{FormattedString, Format, Color, Style};
use line_buffer::LineBuffer;
use server_data::ParseState;
use mio::tcp::TcpStream;

pub trait HasLength {
    fn len(&self) -> usize;
}

pub struct Indexed<T> {
    index: usize,
    pub data: T
}

impl<T: HasLength> Indexed<T> {
    pub fn new(data: T) -> Indexed<T> {
        Indexed { index: 0, data: data }
    }
    pub fn index(&self) -> usize { self.index }
    pub fn increment_index(&mut self, amount: usize) {
        if self.index + amount < self.data.len() {
            self.index += amount;
        } else {
            self.index = self.data.len() - 1;
        }
    }
    pub fn decrement_index(&mut self, amount: usize) {
        if amount > self.index {
            self.index = 0;
        } else {
            self.index -= amount;
        }
    }
    pub fn reset_index(&mut self) {
        self.index = 0;
    }
}

impl HasLength for LineBuffer {
    fn len(&self) -> usize { self.len() }
}

impl HasLength for FormattedString {
    fn len(&self) -> usize { self.len() }
}

pub struct Session {
    pub connection: TcpStream,
    pub telnet_state: ParseState,
    pub esc_seq_state: ParseState,
    pub char_format: Format,
    pub history: Indexed<LineBuffer>,
    pub cursor_index: usize,
    pub scrollback_buf: Indexed<LineBuffer>
}

impl Session {
    pub fn new(connection: TcpStream) -> Session {
        Session {
            connection: connection,
            telnet_state: ParseState::NotInProgress,
            esc_seq_state: ParseState::NotInProgress,
            char_format: Format {
                style: Style::Normal,
                fg_color: Color::Default,
                bg_color: Color::Default
            },
            history: Indexed::<_>::new(LineBuffer::new(None, None)),
            cursor_index: 0,
            scrollback_buf: Indexed::<_>::new(LineBuffer::new(None, None))
        }
    }
}
