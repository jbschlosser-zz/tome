extern crate argparse;
extern crate log4rs;
#[macro_use] extern crate log;
extern crate mio;
#[macro_use] extern crate resin;
extern crate tome;
extern crate xdg;

mod actions;
mod context;
mod indexed;
mod scripting;
mod server_data;
mod session;
mod ui;

use argparse::{ArgumentParser, Store};
use mio::Handler;
use mio::tcp::TcpStream;
use std::io::Read;
use std::net::{SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;

use context::Context;
use session::Session;
use ui::UserInterface;
use tome::{formatted_string, Color, RingBuffer};

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

// Helper function to read the config filepath.
fn get_config_filepath() -> Result<PathBuf, String> {
    let xdg_dirs = match xdg::BaseDirectories::with_prefix("tome") {
        Ok(b) => b,
        Err(e) => return Err(format!("{}", e))
    };
    match xdg_dirs.find_config_file("tome.scm") {
        Some(fp) => Ok(fp),
        None => Err("Could not find config file".to_string())
    }
}

struct MainHandler {
    context: Context,
    ui: UserInterface
}

impl MainHandler {
    pub fn new(context: Context, ui: UserInterface) -> Self {
        MainHandler {context: context, ui: ui}
    }
}

impl Handler for MainHandler {
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
                let keep_going = self.context.do_binding(keycode);
                match keep_going {
                    Some(kp) => {
                        if kp {
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
            let bytes_read = self.context.current_session_mut().
                connection.read(&mut buffer);
            match bytes_read {
                Ok(a) =>  {
                    let string = server_data::handle_server_data(&buffer[0..a],
                        self.context.current_session_mut());
                    actions::write_scrollback(
                        &mut self.context, string);

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

    // Parse arguments.
    let mut host = "127.0.0.1".to_string();
    let mut port = "4000".to_string();
    {
        // test: 66.228.38.196 8679
        let mut ap = ArgumentParser::new();
        ap.set_description("Example: tome 127.0.0.1 4000");
        ap.refer(&mut host)
            .add_argument("host", Store, "Server IP address");
        ap.refer(&mut port)
            .add_argument("port", Store, "Port number");
        ap.parse_args_or_exit();
    }

    // Set up event loop.
    let mut event_loop = mio::EventLoop::new().unwrap();

    // Monitor stdin.
    let stdin = mio::Io::from_raw_fd(0);
    event_loop.register(&stdin, mio::Token(0), mio::EventSet::readable(),
        mio::PollOpt::empty()).unwrap();

    // Connect to the server.
    let addr = match SocketAddr::from_str(&format!("{}:{}", &host, &port)) {
        Ok(a) => a,
        Err(e) => {
            println!("Error: bad host: {}", e);
            return;
        }
    };
    let stream = TcpStream::connect(&addr).unwrap();
    event_loop.register(&stream, mio::Token(1), mio::EventSet::readable(),
        mio::PollOpt::empty()).unwrap();

    // Look for the config file; use a default path if something goes wrong.
    let config_filepath = get_config_filepath()
        .unwrap_or_else(|_| {
            let mut pb = PathBuf::new();
            pb.push("~/.config/tome/tome.scm");
            pb
        });

    // Set up the context.
    let mut context = Context::new(config_filepath);
    context.sessions.push(Session::new(stream, RingBuffer::new(None)));

    // Initialize the UI.
    let ui = UserInterface::init();

    // Load the config file.
    actions::reload_config(&mut context);
    
    // Run the event loop.
    let mut handler = MainHandler::new(context, ui);
    let _ = event_loop.run(&mut handler);

    // Clean up.
    handler.ui.teardown();
}
