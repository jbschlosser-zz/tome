extern crate argparse;
extern crate log4rs;
#[macro_use] extern crate log;
extern crate mio;
extern crate regex;
#[macro_use] extern crate resin; // TODO: conditional compilation
extern crate tome;
extern crate xdg;

mod actions;
mod context;
mod indexed;
mod scripting;
mod session;
mod ui;

use argparse::{ArgumentParser, Store};
use mio::*;
use mio::tcp::TcpStream;
use std::cmp;
use std::io::Read;
use std::net::{SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;

use context::Context;
use indexed::Indexed;
use session::Session;
use ui::UserInterface;
use tome::{formatted_string, Color, RingBuffer};

fn main() {
    // Enable logging.
    log4rs::init_file("config/log.yaml", Default::default()).unwrap();

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

    // Set up polling.
    let poll = Poll::new().unwrap();

    // Monitor stdin.
    let stdin_fd = 0;
    let stdin = mio::unix::EventedFd(&stdin_fd);
    poll.register(&stdin, Token(0), Ready::readable(), PollOpt::level()).unwrap();

    // Connect to the server.
    let addr = match SocketAddr::from_str(&format!("{}:{}", &host, &port)) {
        Ok(a) => a,
        Err(e) => {
            println!("Error: bad host: {}", e);
            return;
        }
    };
    let stream = TcpStream::connect(&addr).unwrap();
    poll.register(&stream, Token(1), Ready::readable(), PollOpt::level()).unwrap();

    // Initialize the UI.
    let mut ui = UserInterface::init();
    let viewport_lines = ui.output_win_height();

    // Look for the config file; use a default path if something goes wrong.
    let config_filepath = get_config_filepath()
        .unwrap_or_else(|_| {
            let mut pb = PathBuf::new();
            pb.push("~/.config/tome/tome.scm");
            pb
        });

    // Set up the context.
    let mut context = Context::new(config_filepath, viewport_lines);
    context.sessions.push(Session::new(stream,
        Indexed::<_>::new(RingBuffer::new(None),
            move |buf| {
                cmp::max(buf.len(), viewport_lines) - viewport_lines
            })));

    // Load the config file.
    actions::reload_config(&mut context);

    // Display the initial UI state.
    update_ui(&mut ui, &context);
    
    // Run the polling loop.
    let mut events = Events::with_capacity(1024);
    'main: loop {
        match poll.poll(&mut events, None) {
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::Interrupted => {
                        // The assumption here is that an interrupted error
                        // generally corresponds to a screen resize.

                        // Resize.
                        ui.restart();
                        let viewport_lines = ui.output_win_height();
                        context.viewport_lines = viewport_lines;
                        for session in context.sessions.iter_mut() {
                            session.scrollback_buf.set_limit(
                                move |buf| {
                                    cmp::max(buf.len(), viewport_lines) - viewport_lines
                                });
                        }
                        update_ui(&mut ui, &context);
                        for session in context.sessions.iter_mut() {
                            poll.reregister(&session.connection, Token(1), Ready::readable(), PollOpt::edge())
                                .unwrap();
                        }
                    },
                    _ => break 'main // TODO: Handle this differently?
                }
            }
            _ => ()
        }
        for event in events.iter() {
            match event.token() {
                Token(0) => {
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
                            if context.key_codes_to_names.contains_key(&esc_seq) {
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
                        let keep_going = context.do_binding(keycode);
                        match keep_going {
                            Some(kp) => {
                                if kp {
                                    update_ui(&mut ui, &context);
                                } else {
                                    // Stop polling.
                                    break 'main;
                                }
                            },
                            None => {
                                actions::write_scrollback(
                                    &mut context,
                                    formatted_string::with_color(
                                        &format!("No binding found for keycode: {:?}\n",
                                        keycode), Color::Red));
                                update_ui(&mut ui, &context);
                            }
                        }
                    }
                },
                Token(1) => {
                    let mut buffer = [0; 4096];
                    let bytes_read = context.current_session_mut().
                        connection.read(&mut buffer);
                    match bytes_read {
                        Ok(a) =>  {
                            if a > 0 {
                                actions::receive_data(&mut context, &buffer[0..a]);
                                update_ui(&mut ui, &context);
                            } else {
                                // Reading 0 bytes indicates the connection was closed.
                                poll.deregister(&context.current_session().connection);
                            }

                            update_ui(&mut ui, &context);
                        },
                        Err(e) => {
                            match e.kind() {
                                std::io::ErrorKind::WouldBlock => (), // Happens during resizes.
                                _ => panic!("Error when reading from socket: {:?}", e)
                            }
                        }
                    }
                },
                _ => unreachable!()
            }
        }
    }

    // Clean up.
    ui.teardown();
}

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
