extern crate log;
extern crate log4rs;
extern crate mio;
extern crate tome;

use mio::Handler;
use mio::tcp::TcpStream;
use std::io::Read;
use std::net::{SocketAddr};
use std::str::FromStr;
use tome::{actions, handle_server_data, Session, Context, UserInterface,
    formatted_string, Color};

fn update_ui(ui: &mut UserInterface, context: &Context) {
    let scroll_index = context.current_session().scrollback_buf.index();
    let history_index = context.history.index();
    let output_win_height = ui.output_win_height();
    ui.update(
        context.current_session().scrollback_buf.data
            .most_recent(scroll_index + output_win_height),
        context.history.data.most_recent(history_index + 1),
        context.cursor_index);
}

struct MainHandler<'a> {
    context: Context<'a>,
    ui: UserInterface
}

impl<'a> MainHandler<'a> {
    pub fn new(context: Context<'a>, ui: UserInterface) -> Self {
        MainHandler {context: context, ui: ui}
    }
}

impl<'a> Handler for MainHandler<'a> {
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
                    if self.context.key_codes_to_names.contains_key(&esc_seq) {
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
                match self.context.do_binding(keycode) {
                    Some(keep_going) => {
                        if keep_going {
                            update_ui(&mut self.ui, &self.context);
                        } else {
                            event_loop.shutdown();
                        }
                    },
                    None => {
                        actions::write_scrollback(
                            &mut self.context,
                            formatted_string::with_color(
                                &format!("No binding found for keycode: {:?}\n",
                                keycode), Color::Red));
                        update_ui(&mut self.ui, &self.context);
                    }
                }
            }
        } else if token == mio::Token(1) {
            let mut buffer = [0; 4096];
            actions::write_scrollback(
                &mut self.context,
                formatted_string::with_color(
                    &format!("Data received!\n"), Color::Red));
            update_ui(&mut self.ui, &self.context);
            match self.context.current_session_mut().
                connection.read(&mut buffer)
            {
                Ok(a) =>  {
                    let string = handle_server_data(&buffer[0..a],
                        self.context.current_session_mut());
                    actions::write_scrollback(&mut self.context, string);

                    update_ui(&mut self.ui, &self.context);
                },
                Err(_) => panic!("Error when reading from socket")
            }
        }
    }
    fn interrupted(&mut self, _: &mut mio::EventLoop<Self>) {
        // Resize.
        self.ui.restart();
        update_ui(&mut self.ui, &self.context);
    }
}

fn main() {
    // Enable logging.
    log4rs::init_file("config/log.toml", Default::default()).unwrap();

    // Set up event loop.
    let mut event_loop = mio::EventLoop::new().unwrap();

    // Monitor stdin.
    let stdin = mio::Io::from_raw_fd(0);
    event_loop.register(&stdin, mio::Token(0), mio::EventSet::readable(),
        mio::PollOpt::empty()).unwrap();

    // Set up the context.
    let mut context = Context::new();

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

    let mut handler = MainHandler::new(context, ui);
    let _ = event_loop.run(&mut handler);

    // Clean up.
    handler.ui.teardown();
}
