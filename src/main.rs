#![feature(net)]
#![feature(std_misc)]

extern crate mio;
extern crate rustbox;
extern crate tome;

use mio::{Handler, TryRead};
use mio::net::tcp;
use std::char;
use std::net::SocketAddr;
use std::str::FromStr;
use tome::{handle_server_data, Session, Context, UserInterface, KeyEvent};
use tome::{Attributes, Color, Style, make_color_string};

struct MyHandler(Context, tcp::TcpStream, UserInterface);
impl Handler<(), ()> for MyHandler {
    fn readable(&mut self,
        event_loop: &mut mio::EventLoop<(), ()>,
        token: mio::Token,
        _: mio::ReadHint)
    {
        if token == mio::Token(0) {
            match self.2.check_for_event() {
                Ok(KeyEvent(_, _, ch)) => {
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
                    let history_index = sess.input_buf.1;
                    let ui_height = self.2.height();
                    self.2.update(&sess.output_buf.0.get_lines(scroll_index, ui_height),
                        &sess.input_buf.0.get_lines(history_index, 1));
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
                    let scroll_index = sess.output_buf.1;
                    let history_index = sess.input_buf.1;
                    let ui_height = self.2.height();
                    self.2.update(&sess.output_buf.0.get_lines(scroll_index, ui_height),
                        &sess.input_buf.0.get_lines(history_index, 1));
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

    // Run the main loop.
    let mut context = Context::new();
    context.sessions.push(Session::new());
    context.session_index = Some(0);
    let attrs = Attributes {
        style: Style::Normal, fg_color: Color::Default, bg_color: Color::Default
    };
    context.get_current_session().unwrap().input_buf.0.insert(
        &make_color_string("hello", attrs));
    let ui = UserInterface::init();
    let _ = event_loop.run(&mut MyHandler(context, stream, ui));
}
