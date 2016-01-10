use formatted_string::{FormattedString, Format, Color, Style};
use indexed::Indexed;
use ring_buffer::RingBuffer;
use server_data::ParseState;
use mio::tcp::TcpStream;

pub struct Session {
    pub connection: TcpStream,
    pub telnet_state: ParseState,
    pub esc_seq_state: ParseState,
    pub char_format: Format,
    pub scrollback_buf: Indexed<RingBuffer<FormattedString>>
}

impl Session {
    pub fn new(connection: TcpStream) -> Session {
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
            scrollback_buf: scrollback_buf
        }
    }
}
