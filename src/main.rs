#![feature(net)]
#![feature(std_misc)]

extern crate mio;
extern crate regex;
extern crate rustbox;
extern crate tome;

use mio::Handler;
use mio::TryRead;
use mio::net::tcp;
use rustbox::{Color, Style, InitOptions, RustBox};
use std::char;
use std::default::Default;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::duration::Duration;
use tome::handle_server_data;
use tome::{ColorChar, Session, Context, LineBuffer};
fn convert_color(input: tome::Color) -> Color {
    match input {
        tome::Color::Default => Color::Default,
        tome::Color::Black => Color::Black,
        tome::Color::Red => Color::Red,
        tome::Color::Green => Color::Green,
        tome::Color::Yellow => Color::Yellow,
        tome::Color::Blue => Color::Blue,
        tome::Color::Magenta => Color::Magenta,
        tome::Color::Cyan => Color::Cyan,
        tome::Color::White => Color::White
    }
}

fn convert_style(input: tome::Style) -> Style {
    match input {
        tome::Style::Normal => rustbox::RB_NORMAL,
        tome::Style::Bold => rustbox::RB_BOLD,
        tome::Style::Standout => rustbox::RB_REVERSE,
    }
}

fn display_chars(lines: &[&[ColorChar]], rustbox: &RustBox) {
    // Fit the lines to the screen size.
    let mut screen_buf = LineBuffer::new(
        Some(rustbox.height()), Some(rustbox.width()));
    for line in lines.iter() {
        screen_buf.insert(&line);
        screen_buf.move_to_next_line();
    }

    // Print the lines.
    rustbox.clear();
    let mut loc_x = 0;
    let mut loc_y = 0;
    let lines_to_print = screen_buf.get_lines(0, rustbox.height());
    for line in lines_to_print.iter() {
        for ch in line.iter() {
            let style = convert_style(ch.attrs.style);
            let fg_color = convert_color(ch.attrs.fg_color);
            let bg_color = convert_color(ch.attrs.bg_color);
            rustbox.print(loc_x, loc_y, style, fg_color, bg_color, &ch.ch.to_string());
            loc_x += 1;
        }
        loc_x = 0;
        loc_y += 1;
    }
    rustbox.present();
}

struct MyHandler(Context, tcp::TcpStream, RustBox);
impl Handler<(), ()> for MyHandler {
    fn readable(&mut self,
        event_loop: &mut mio::EventLoop<(), ()>,
        token: mio::Token,
        _: mio::ReadHint)
    {
        if token == mio::Token(0) {
            match self.2.peek_event(Duration::milliseconds(0)) {
                Ok(rustbox::Event::KeyEvent(_, _, ch)) => {
                    let sess = self.0.get_current_session().unwrap();
                    match char::from_u32(ch) {
                        Some('q') => event_loop.shutdown(),
                        Some('u') => {
                            sess.output_buf.1 += 1;
                        },
                        Some('d') => {
                            if sess.output_buf.1 > 0 {
                                sess.output_buf.1 -= 1;
                            }
                        },
                        _ => ()
                    }

                    let scroll_index = sess.output_buf.1;
                    display_chars(&sess.output_buf.0.get_lines(
                        scroll_index, self.2.width()), &self.2);
                },
                Err(_) => panic!("Bad event found"),
                _ => ()
            }
        } else if token == mio::Token(1) {
            let mut bb = [0; 4096];
            match self.1.read_slice(&mut bb) {
                Err(_) => panic!("An error occurred"),
                Ok(None) => panic!("Would block"),
                Ok(Some(a)) => {
                    let sess = self.0.get_current_session().unwrap();
                    let chars = handle_server_data(&bb[0..a], sess);
                    sess.output_buf.0.insert(&chars);
                    sess.output_buf.1 = 5;
                    let scroll_index = sess.output_buf.1;
                    display_chars(&sess.output_buf.0.get_lines(
                        scroll_index, self.2.width()), &self.2);
                }
            }
        }
    }
}

fn main() {
    // Set up event loop.
    let mut event_loop = mio::EventLoop::<(), ()>::new().unwrap();

    // Monitor stdin.
    event_loop.register(&mio::Io::new(0), mio::Token(0)).unwrap();

    // Connect to server.
    let sock = tcp::TcpSocket::v4().unwrap();
    let sock_addr = SocketAddr::from_str("66.228.38.196:8679").unwrap();
    let result = sock.connect(&sock_addr).unwrap();
    let stream = result.0;
    event_loop.register(&stream, mio::Token(1)).unwrap();

    // Rustbox stuff.
    let rustbox = match RustBox::init(InitOptions {
        buffer_stderr: true,
        ..Default::default()
    }) {
        Result::Ok(v) => v,
        Result::Err(e) => panic!("{}", e)
    };
    rustbox.present();

    // Run the main loop.
    let mut context = Context::new();
    context.sessions.push(Session::new());
    context.session_index = Some(0);
    let _ = event_loop.run(&mut MyHandler(context, stream, rustbox));
}
