#![feature(collections)]

extern crate mio;
extern crate tome;

use mio::{Handler, TryRead, TryWrite};
use std::char;
use std::net::{TcpStream, SocketAddr};
use std::str::FromStr;
use tome::{handle_server_data, Session, Context, UserInterface,
    FormattedString, Format, Color, KEY_RESIZE};

fn update_ui(ui: &mut UserInterface, sess: &Session) {
    let scroll_index = sess.scrollback_buf.index();
    let history_index = sess.history.index();
    let ui_height = UserInterface::height();
    ui.update(
        &sess.scrollback_buf.data.get_lines(scroll_index, ui_height),
        &sess.history.data.get_lines(history_index, 1),
        sess.cursor_index);
}

struct MyHandler(Context, UserInterface);
impl Handler for MyHandler {
    type Timeout = mio::NonBlock<TcpStream>;
    type Message = ();

    fn readable(&mut self,
        event_loop: &mut mio::EventLoop<Self>,
        token: mio::Token,
        _: mio::ReadHint)
    {
        if token == mio::Token(0) {
            let key = self.1.check_for_event();
            match self.0.do_binding(key) {
                Some(keep_going) => {
                    if keep_going {
                        update_ui(&mut self.1, self.0.get_current_session().unwrap());
                    } else {
                        event_loop.shutdown();
                    }
                },
                None => {
                    let sess = self.0.get_current_session().unwrap();
                    sess.scrollback_buf.data.push(&FormattedString::with_color(
                        &format!("No binding found for key: {}\n", key), Color::Red));
                    update_ui(&mut self.1, sess);
                }
            }
        } else if token == mio::Token(1) {
            let mut buffer = [0; 4096];
            let sess = self.0.get_current_session().unwrap();
            match sess.connection.read_slice(&mut buffer) {
                Ok(Some(a)) =>  {
                    let chars = handle_server_data(&buffer[0..a], sess);
                    sess.scrollback_buf.data.push(&chars);

                    update_ui(&mut self.1, sess);
                },
                Ok(None) => (),
                Err(_) => panic!("Error when reading from socket")
            }
        }
    }
    fn writable(&mut self,
        _: &mut mio::EventLoop<Self>,
        _: mio::Token)
    {
        //if token == mio::Token(1) {
            let sess = self.0.get_current_session().unwrap();
            sess.scrollback_buf.data.push(&FormattedString::with_color(
                &format!("Connected!"), Color::Green));
            update_ui(&mut self.1, sess);
        //}
    }
    fn interrupted(&mut self, _: &mut mio::EventLoop<Self>) {
        // Resize.
        self.1.restart();
        let sess = self.0.get_current_session().unwrap();
        update_ui(&mut self.1, sess);
    }
}

fn main() {
    // Set up event loop.
    let mut event_loop = mio::EventLoop::new().unwrap();

    // Monitor stdin.
    let stdin = mio::Io::new(0);
    event_loop.register(&stdin, mio::Token(0)).unwrap();

    // Set up the context.
    let mut context = Context::new();

    // Set up the key bindings.
    // Q (quit)
    context.bindings.insert(0x71, Box::new(|_: &mut Session| false));

    // Page up
    context.bindings.insert(338, Box::new(|sess: &mut Session| {
        sess.scrollback_buf.decrement_index(1);
        true
    }));

    // Page down
    context.bindings.insert(339, Box::new(|sess: &mut Session| {
        sess.scrollback_buf.increment_index(1);
        true
    }));

    // Backspace
    context.bindings.insert(263, Box::new(|sess: &mut Session| {
        let cursor = sess.cursor_index;
        if cursor > 0 {
            let index = sess.history.index();
            sess.history.data.get_line_mut(index).remove(cursor - 1);
            sess.cursor_index -= 1;
        }
        true
    }));

    // Delete
    context.bindings.insert(330, Box::new(|sess: &mut Session| {
        let input_len = sess.history.data.get_line(
            sess.history.index()).len();
        let cursor = sess.cursor_index;
        if cursor < input_len {
            let index = sess.history.index();
            sess.history.data.get_line_mut(index).remove(cursor);
        }
        true
    }));

    // Enter
    context.bindings.insert(10, Box::new(|sess: &mut Session| {
        // Send the input to the server.
        let mut send_data = String::from_str(sess.history.data.get_line(
            sess.history.index()).to_str());
        send_data.push_str("\r\n");
        sess.connection.write_slice(send_data.as_bytes()); // TODO: Check result.

        // Add the input to the scrollback buffer.
        sess.scrollback_buf.data.push(
            &FormattedString::with_color(&send_data, Color::Yellow));

        // Add the input to the history.
        if sess.history.index() > 0 {
            sess.history.reset_index();
            sess.history.data.get_line_mut(0).clear();
            sess.history.data.push(
                &FormattedString::with_format(&send_data, Format::default()));
        } else {
            sess.history.data.move_to_next_line();
        }

        // Reset the cursor.
        sess.cursor_index = 0;
        true
    }));

    // Left arrow
    context.bindings.insert(260, Box::new(|sess: &mut Session| {
        let cursor = sess.cursor_index;
        if cursor > 0 {
            sess.cursor_index -= 1;
        }
        true
    }));

    // Right arrow
    context.bindings.insert(261, Box::new(|sess: &mut Session| {
        let input_len = sess.history.data.get_line(
            sess.history.index()).len();
        let cursor = sess.cursor_index;
        if cursor < input_len {
            sess.cursor_index += 1;
        }
        true
    }));

    // Up arrow
    context.bindings.insert(259, Box::new(|sess: &mut Session| {
        sess.history.increment_index(1);
        sess.cursor_index = sess.history.data.get_line(
            sess.history.index()).len();
        true
    }));

    // Down arrow
    context.bindings.insert(258, Box::new(|sess: &mut Session| {
        sess.history.decrement_index(1);
        sess.cursor_index = sess.history.data.get_line(
            sess.history.index()).len();
        true
    }));

    // KEY_RESIZE
    context.bindings.insert(KEY_RESIZE, Box::new(|_: &mut Session| {
        // Do nothing; the resize is handled elsewhere and this
        // key is unfortunately generated.
        true
    }));

    // Keys that should be displayed directly.
    for i in 0x20..0x71 {
        context.bindings.insert(i, Box::new(move |sess: &mut Session| {
            let hist_index = sess.history.index();
            sess.history.data.get_line_mut(hist_index).push(
                char::from_u32(i as u32).unwrap(), Format::default());
            sess.cursor_index += 1;
            true
        }));
    }
    for i in 0x72..0x7F {
        context.bindings.insert(i, Box::new(move |sess: &mut Session| {
            let hist_index = sess.history.index();
            sess.history.data.get_line_mut(hist_index).push(
                char::from_u32(i as u32).unwrap(), Format::default());
            sess.cursor_index += 1;
            true
        }));
    }

    // Initialize the UI.
    let ui = UserInterface::init();

    // Monitor the socket.
    let socket = mio::tcp::v4().unwrap();
    event_loop.register(&socket, mio::Token(1)).unwrap();

    // Connect to the server.
    let (stream, _) = socket.connect(
        &SocketAddr::from_str("66.228.38.196:8679").unwrap()).unwrap();
    context.sessions.push(Session::new(stream));
    context.session_index = Some(0);

    let mut handler = MyHandler(context, ui);
    let _ = event_loop.run(&mut handler);

    // Clean up.
    handler.1.teardown();
}
