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
use tome::{ColorChar, EscSequence, Session, Context};

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
    let mut loc_x = 0;
    let mut loc_y = 0;
    for line in lines.iter() {
        for ch in line.iter() {
            let style = convert_style(ch.attrs.style);
            let fg_color = convert_color(ch.attrs.fg_color);
            let bg_color = convert_color(ch.attrs.bg_color);
            match ch.ch {
                '\n' => { loc_x = 0; loc_y += 1; }
                '\r' => (),
                c => {
                    rustbox.print(loc_x, loc_y, style, fg_color, bg_color, &c.to_string());
                    loc_x += 1;
                    if loc_x >= rustbox.width() {
                        loc_x = 0;
                        loc_y += 1;
                    }
                }
            }
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
                    match char::from_u32(ch) {
                        Some('q') => event_loop.shutdown(),
                        _ => ()
                    }
                },
                Err(_) => panic!("Bad event found"),
                _ => ()
            }
        } else if token == mio::Token(1) {
            let mut bb = [0; 4096];
            match self.1.read_slice(&mut bb) {
                Err(_) => println!("An error occurred"),
                Ok(None) => println!("Would block"),
                Ok(Some(a)) => {
                    let chars = handle_server_data(&bb[0..a], &mut self.0.sessions[0]);
                    self.0.sessions[0].output.0.insert(&chars);
                    self.0.sessions[0].output.1 = 5;
                    display_chars(&self.0.sessions[0].output.0.get_lines(
                        self.0.sessions[0].output.1, self.2.width()), &self.2);
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
    let _ = event_loop.run(&mut MyHandler(context, stream, rustbox));

    /*let mut bindings: HashMap<i64, Box<Fn(&mut Session) -> bool>> = HashMap::new();
    bindings.insert(2, Box::new(|s| { s.x = s.x + 1; s.x == s.y }));
    bindings.insert(3, Box::new(|s| s.x != s.y));
    let mut sess = Session {x: 1, y: 2};
    bindings.get(&2).unwrap()(&mut sess);
    println!("{:?}", sess);*/
}
