use context::Context;
use resin::Datum;
use scripting;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::PathBuf;
use tome::{formatted_string, Color, Format, FormattedString, RingBuffer};

// Actions to be used directly for key bindings.
pub fn quit(_: &mut Context) -> bool { false }
pub fn prev_page(context: &mut Context) -> bool {
    context.current_session_mut().scrollback_buf.increment_index(1);
    true
}
pub fn next_page(context: &mut Context) -> bool {
    context.current_session_mut().scrollback_buf.decrement_index(1);
    true
}
pub fn backspace_input(context: &mut Context) -> bool {
    let cursor = context.cursor_index;
    if cursor > 0 {
        let index = context.history.index();
        context.history.data.get_recent_mut(index).remove(cursor - 1);
        context.cursor_index -= 1;
    }
    true
}
pub fn delete_input_char(context: &mut Context) -> bool {
    let input_len = context.history.data.get_recent(
        context.history.index()).len();
    let cursor = context.cursor_index;
    if cursor < input_len {
        let index = context.history.index();
        context.history.data.get_recent_mut(index).remove(cursor);
    }
    true
}
pub fn send_input(context: &mut Context) -> bool {
    // Check for an input hook. If one exists, run it; otherwise, just send
    // the contents of the input line.
    let input_line_contents = formatted_string::to_string(
        context.history.data.get_recent(context.history.index()));
    let to_send: Result<Vec<String>, String> =
        match context.interpreter.root().get("send-input-hook") {
            Some(h) => {
                let expr = list!(h, Datum::String(input_line_contents.clone()));
                match context.interpreter.evaluate_datum(&expr) {
                    Ok(d) => convert_to_string_vec(d),
                    Err((e, _)) => Err(e.msg)
                }
            },
            None => {
                Ok(vec![input_line_contents.clone()])
            }
        };

    match to_send {
        Ok(svec) => {
            for s in svec {
                // Write to the network.
                send_data(context, &s, true);
                
                // Add to the scrollback buffer.
                write_scrollback(context,
                    formatted_string::with_color(
                        &format!("{}\n", &s),
                        Color::Yellow));
            }
        },
        Err(e) => {
            // Write the error to the scrollback buffer.
            write_scrollback(context,
                formatted_string::with_color(
                    &format!("{}\n", &e),
                    Color::Red));
        }
    }

    // Add the input to the history and clear the input line.
    if context.history.index() > 0 {
        // History has been scrolled back and needs to be reset.
        context.history.reset_index();
        context.history.data.get_recent_mut(0).clear();
        write_to_line_buffer(
            &mut context.history.data,
            formatted_string::with_format(
                &format!("{}\n", &input_line_contents),
                Format::default()));
    } else {
        // Input line already contains the right data; just move
        // to the next line.
        context.history.data.push(FormattedString::new());
    }

    // Reset the cursor.
    context.cursor_index = 0;
    true
}
// Helper function to return the converted vec or an error string.
fn convert_to_string_vec(datum: Datum) -> Result<Vec<String>, String> {
    let mut svec = Vec::new();
    let (dvec, _) = datum.as_vec();
    for d in dvec {
        if let Datum::String(s) = d {
            svec.push(s);
        } else {
            return Err("Expected a string or list of strings from the input line hook"
                .to_string())
        }
    }
    Ok(svec)
}
pub fn cursor_left(context: &mut Context) -> bool {
    let cursor = context.cursor_index;
    if cursor > 0 {
        context.cursor_index -= 1;
    }
    true
}
pub fn cursor_right(context: &mut Context) -> bool {
    let input_len = context.history.data.get_recent(
        context.history.index()).len();
    let cursor = context.cursor_index;
    if cursor < input_len {
        context.cursor_index += 1;
    }
    true
}
pub fn history_prev(context: &mut Context) -> bool {
    context.history.increment_index(1);
    context.cursor_index = context.history.data.get_recent(
        context.history.index()).len();
    true
}
pub fn history_next(context: &mut Context) -> bool {
    context.history.decrement_index(1);
    context.cursor_index = context.history.data.get_recent(
        context.history.index()).len();
    true
}
pub fn delete_to_cursor(context: &mut Context) -> bool {
    let history_index = context.history.index();
    let curr_line = context.history.data.get_recent_mut(history_index);
    let after_cursor = curr_line.split_off(context.cursor_index);
    curr_line.clear();
    curr_line.extend(after_cursor);
    context.cursor_index = 0;
    true
}
pub fn reload_config(context: &mut Context) -> bool {
    // Read the config file (if it exists).
    context.interpreter = scripting::get_interpreter();
    let _ = read_file_contents(&context.config_filepath)
        .map_err(|e| {
        write_scrollback(context,
            formatted_string::with_color(
                &format!("Warning: failed to read config file! ({})\n", e),
                Color::Yellow));
    }).map(|contents: String| {
        match context.interpreter.evaluate(&contents) {
            Ok(_) => (),
            Err(e) => {
                println!("bad news");
                write_scrollback(context,
                    formatted_string::with_color(
                    &format!("Warning: config file error:\n{}\n", e),
                    Color::Yellow))
            }
        }
    });
    true
}
// Helper function to read a file's contents.
fn read_file_contents(filepath: &PathBuf) -> io::Result<String> {
    let mut file = try!(File::open(filepath));
    let mut file_contents = String::new();
    try!(file.read_to_string(&mut file_contents));
    Ok(file_contents)
}

// Actions with arguments.
pub fn write_scrollback(context: &mut Context, data: FormattedString) {
    write_to_line_buffer(
        &mut context.current_session_mut().scrollback_buf.data,
        data);
}
// Helper function to handle writing to buffers while being line-aware.
fn write_to_line_buffer(buffer: &mut RingBuffer<FormattedString>,
    data: FormattedString)
{
    for (ch, format) in data {
        match ch {
            '\r' => (),
            '\n' => buffer.push(FormattedString::new()),
            _ => buffer.get_recent_mut(0).push((ch, format))
        }
    }
}
pub fn send_data(context: &mut Context, data: &str, add_line_ending: bool) {
    // TODO: Check result.
    let data_to_send = format!("{}{}", data,
        if add_line_ending {"\r\n"} else {""});
    context.current_session_mut().connection.write(data_to_send.as_bytes());
}
pub fn insert_input_char(context: &mut Context, ch: char) {
    let hist_index = context.history.index();
    context.history.data.get_recent_mut(hist_index).insert(
        context.cursor_index, (ch, Format::default()));
    context.cursor_index += 1;
}