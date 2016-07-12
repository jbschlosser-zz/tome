use indexed::Indexed;
use tome::{FormattedString, Format, Color, Style, ParseState, RingBuffer,
    SearchResult};
use mio::tcp::TcpStream;

pub struct Session {
    pub connection: TcpStream,
    pub telnet_state: ParseState,
    pub esc_seq_state: ParseState,
    pub char_format: Format,
    pub scrollback_buf: Indexed<RingBuffer<FormattedString>>,
    pub prev_search_result: Option<SearchResult>
}

impl Session {
    pub fn new(connection: TcpStream,
        mut buffer: Indexed<RingBuffer<FormattedString>>) -> Session
    {
        if buffer.data.len() == 0 {
            // Ensure that a line is present so that the buffer
            // can be indexed.
            buffer.data.push(FormattedString::new());
        }
        Session {
            connection: connection,
            telnet_state: ParseState::NotInProgress,
            esc_seq_state: ParseState::NotInProgress,
            char_format: Format {
                style: Style::Normal,
                fg_color: Color::Default,
                bg_color: Color::Default
            },
            scrollback_buf: buffer,
            prev_search_result: None
        }
    }
}
