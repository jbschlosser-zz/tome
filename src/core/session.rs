use formatted_string::{FormattedString, Format, Color, Style};
use ring_buffer::RingBuffer;
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

impl<T> HasLength for RingBuffer<T> {
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
    pub history: Indexed<RingBuffer<FormattedString>>,
    pub cursor_index: usize,
    pub scrollback_buf: Indexed<RingBuffer<FormattedString>>
}

impl Session {
    pub fn new(connection: TcpStream) -> Session {
        let mut history = Indexed::<_>::new(RingBuffer::new(None));
        history.data.push(FormattedString::new());
        let mut scrollback_buf = Indexed::<_>::new(RingBuffer::new(None));
        scrollback_buf.data.push(FormattedString::new());
        Session {
            connection: connection,
            telnet_state: ParseState::NotInProgress,
            esc_seq_state: ParseState::NotInProgress,
            char_format: Format {
                style: Style::Normal,
                fg_color: Color::Default,
                bg_color: Color::Default
            },
            history: history,
            cursor_index: 0,
            scrollback_buf: scrollback_buf
        }
    }
}
