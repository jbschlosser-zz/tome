#![feature(io)]
#![feature(net)]

extern crate mio;
extern crate tome;

use mio::{Handler, TryRead};
use std::char;
use std::io::Read;
use std::net::TcpStream;
use tome::{handle_server_data, Session, Context, UserInterface};
use tome::{ColorChar, Attributes, Color, Style, make_color_string};

fn update_ui(ui: &mut UserInterface, sess: &Session) {
    let scroll_index = sess.output_buf.1;
    let history_index = sess.input_buf.1;
    let ui_height = UserInterface::height();
    ui.update(&sess.output_buf.0.get_lines(scroll_index, ui_height),
        &sess.input_buf.0.get_lines(history_index, 1));
}

struct MyHandler(Context, TcpStream, UserInterface);
impl Handler for MyHandler {
    type Timeout = mio::NonBlock<TcpStream>;
    type Message = ();

    fn readable(&mut self,
        event_loop: &mut mio::EventLoop<MyHandler>,
        token: mio::Token,
        _: mio::ReadHint)
    {
        if token == mio::Token(0) {
            match self.0.do_binding(self.2.check_for_event()) {
                Some(keep_going) => {
                    if keep_going {
                        update_ui(&mut self.2, self.0.get_current_session().unwrap());
                    } else {
                        event_loop.shutdown();
                    }
                },
                None => ()
            }
        } else if token == mio::Token(1) {
            let mut buffer = [0; 4096];
            match self.1.read(&mut buffer) {
                Ok(a) =>  {
                    let sess = self.0.get_current_session().unwrap();
                    let chars = handle_server_data(&buffer[0..a], sess);
                    sess.output_buf.0.insert(&chars);

                    update_ui(&mut self.2, sess);
                },
                Err(_) => panic!("Error when reading from socket")
            }
        }
    }
}

fn main() {
    // Set up event loop.
    let mut event_loop = mio::EventLoop::new().unwrap();

    // Monitor stdin.
    let stdin = mio::Io::new(0);
    event_loop.register(&stdin, mio::Token(0)).unwrap();

    // Connect to server.
    let stream = TcpStream::connect("66.228.38.196:8679").unwrap();
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
    context.bindings.insert(0x71, Box::new(|_: &mut Session| false));
    /*context.bindings.insert(0x75, Box::new(|sess: &mut Session| {
        sess.output_buf.1 += 1;
        true
    }));
    context.bindings.insert(0x64, Box::new(|sess: &mut Session| {
        if sess.output_buf.1 > 0 {
            sess.output_buf.1 -= 1;
        }
        true
    }));*/
    for i in 0x20..0x71 {
        context.bindings.insert(i, Box::new(move |sess: &mut Session| {
            sess.input_buf.0.insert_single(
                ColorChar {ch: char::from_u32(i as u32).unwrap(), attrs: attrs});
            true
        }));
    }
    for i in 0x72..0x7F {
        context.bindings.insert(i, Box::new(move |sess: &mut Session| {
            sess.input_buf.0.insert_single(
                ColorChar {ch: char::from_u32(i as u32).unwrap(), attrs: attrs});
            true
        }));
    }
    let ui = UserInterface::init();
    let _ = event_loop.run(&mut MyHandler(context, stream, ui));

    // Clean up.
    UserInterface::teardown();
}
