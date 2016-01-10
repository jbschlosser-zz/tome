extern crate mio;
extern crate tome;

use mio::Handler;
use mio::tcp::TcpStream;
use std::char;
use std::io::{Read, Write};
use std::net::{SocketAddr};
use std::str::FromStr;
use tome::{handle_server_data, Session, Context, UserInterface,
    FormattedString, Format, Color, get_key_codes_to_names};

fn update_ui(ui: &mut UserInterface, sess: &Session) {
    let scroll_index = sess.scrollback_buf.index();
    let history_index = sess.history.index();
    let output_win_height = ui.output_win_height();
    ui.update(
        sess.scrollback_buf.data.
            scrollback(scroll_index + output_win_height - 1),
        sess.history.data.scrollback(history_index),
        sess.cursor_index);
}

struct MyHandler<'a>(Context<'a>, UserInterface);
impl<'a> Handler for MyHandler<'a> {
    type Timeout = mio::tcp::TcpStream;
    type Message = ();

    fn ready(&mut self,
        event_loop: &mut mio::EventLoop<Self>,
        token: mio::Token,
        _: mio::EventSet)
    {
        if token == mio::Token(0) {
            // Read the input from stdin.
            let mut stdin = std::io::stdin();
            let mut buf = vec![0; 4096];
            let num = match stdin.read(&mut buf) {
                Ok(num) => num,
                Err(_) => 0
            };

            // Parse the bytes into keycodes.
            let mut keys_pressed = vec![];
            let mut esc_seq: Vec<u8> = vec![];
            for c in buf[0..num].iter() {
                if esc_seq.len() > 0 {
                    esc_seq.push(*c);
                    if self.0.key_codes_to_names.contains_key(&esc_seq) {
                        keys_pressed.push(esc_seq.clone());
                        esc_seq.clear();
                    }
                } else {
                    if *c == 27 { esc_seq.push(*c) } else {
                        keys_pressed.push(vec![*c]);
                    }
                }
            }
            if esc_seq.len() > 0 {
                keys_pressed.push(esc_seq.clone());
            }

            // Do the bindings.
            for keycode in keys_pressed.iter() {
                match self.0.do_binding(keycode) {
                    Some(keep_going) => {
                        if keep_going {
                            update_ui(&mut self.1,
                                self.0.get_current_session());
                        } else {
                            event_loop.shutdown();
                        }
                    },
                    None => {
                        let sess = self.0.get_current_session();
                        sess.scrollback_buf.data.push(
                            &FormattedString::with_color(
                                &format!("No binding found for keycode: {:?}\n",
                                keycode), Color::Red));
                        update_ui(&mut self.1, sess);
                    }
                }
            }
        } else if token == mio::Token(1) {
            let mut buffer = [0; 4096];
            let sess = self.0.get_current_session();
            sess.scrollback_buf.data.push(&FormattedString::with_color(
                &format!("Data received!\n"), Color::Red));
            update_ui(&mut self.1, sess);
            match sess.connection.read(&mut buffer) {
                Ok(a) =>  {
                    let chars = handle_server_data(&buffer[0..a], sess);
                    sess.scrollback_buf.data.push(&chars);

                    update_ui(&mut self.1, sess);
                },
                Err(_) => panic!("Error when reading from socket")
            }
        }
    }
    fn interrupted(&mut self, _: &mut mio::EventLoop<Self>) {
        // Resize.
        self.1.restart();
        let sess = self.0.get_current_session();
        update_ui(&mut self.1, sess);
    }
}

fn main() {
    // Set up event loop.
    let mut event_loop = mio::EventLoop::new().unwrap();

    // Monitor stdin.
    let stdin = mio::Io::from_raw_fd(0);
    event_loop.register(&stdin, mio::Token(0), mio::EventSet::readable(),
        mio::PollOpt::empty()).unwrap();

    // Set up the context.
    let mut context = Context::new();
    context.key_codes_to_names = get_key_codes_to_names();
    for (code, name) in context.key_codes_to_names.iter() {
        context.key_names_to_codes.insert(name.clone(), code.clone());
    }

    // Set up the key bindings.
    context.bind_key("q", |_: &mut Session| false);
    context.bind_key("PAGEUP", |sess: &mut Session| {
        sess.scrollback_buf.increment_index(1);
        true
    });
    context.bind_key("PAGEDOWN", |sess: &mut Session| {
        sess.scrollback_buf.decrement_index(1);
        true
    });
    context.bind_key("BACKSPACE", |sess: &mut Session| {
        let cursor = sess.cursor_index;
        if cursor > 0 {
            let index = sess.history.index();
            sess.history.data.get_line_mut(index).remove(cursor - 1);
            sess.cursor_index -= 1;
        }
        true
    });
    context.bind_key("DELETE", |sess: &mut Session| {
        let input_len = sess.history.data.get_line(
            sess.history.index()).len();
        let cursor = sess.cursor_index;
        if cursor < input_len {
            let index = sess.history.index();
            sess.history.data.get_line_mut(index).remove(cursor);
        }
        true
    });
    context.bind_key("ENTER", |sess: &mut Session| {
        // Send the input to the server.
        let mut send_data = String::new();
        send_data.push_str(sess.history.data.get_line(
            sess.history.index()).to_str());
        send_data.push_str("\r\n");
        sess.connection.write(send_data.as_bytes()); // TODO: Check result.

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
    });

    // Carriage return. TODO: Clean up this hackery.
    let enter_keycode = context.key_names_to_codes.get("ENTER").unwrap().clone();
    let enter_action = context.bindings.get(&enter_keycode).unwrap().clone();
    context.bindings.insert(vec![13], enter_action);

    context.bind_key("LEFT", |sess: &mut Session| {
        let cursor = sess.cursor_index;
        if cursor > 0 {
            sess.cursor_index -= 1;
        }
        true
    });
    context.bind_key("RIGHT", |sess: &mut Session| {
        let input_len = sess.history.data.get_line(
            sess.history.index()).len();
        let cursor = sess.cursor_index;
        if cursor < input_len {
            sess.cursor_index += 1;
        }
        true
    });
    context.bind_key("UP", |sess: &mut Session| {
        sess.history.increment_index(1);
        sess.cursor_index = sess.history.data.get_line(
            sess.history.index()).len();
        true
    });
    context.bind_key("DOWN", |sess: &mut Session| {
        sess.history.decrement_index(1);
        sess.cursor_index = sess.history.data.get_line(
            sess.history.index()).len();
        true
    });

    // Keys that should be displayed directly.
    for i in 0x20u8..0x71u8 {
        let name = (i as char).to_string();
        context.bind_key(&name, move |sess: &mut Session| {
            let hist_index = sess.history.index();
            sess.history.data.get_line_mut(hist_index).push(
                char::from_u32(i as u32).unwrap(), Format::default());
            sess.cursor_index += 1;
            true
        });
    }
    for i in 0x72u8..0x7Fu8 {
        let name = (i as char).to_string();
        context.bind_key(&name, move |sess: &mut Session| {
            let hist_index = sess.history.index();
            sess.history.data.get_line_mut(hist_index).push(
                char::from_u32(i as u32).unwrap(), Format::default());
            sess.cursor_index += 1;
            true
        });
    }

    // Initialize the UI.
    let ui = UserInterface::init();

    // Connect to the server.
    let stream = TcpStream::connect(
        //&SocketAddr::from_str("66.228.38.196:8679").unwrap()).unwrap();
        &SocketAddr::from_str("127.0.0.1:4000").unwrap()).unwrap();
    event_loop.register(&stream, mio::Token(1), mio::EventSet::readable(),
        mio::PollOpt::empty()).unwrap();
    context.sessions.push(Session::new(stream));
    context.session_index = 0;

    let mut handler = MyHandler(context, ui);
    let _ = event_loop.run(&mut handler);

    // Clean up.
    handler.1.teardown();
}
