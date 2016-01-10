use context::Context;
use std::io::Write;
use formatted_string;
use formatted_string::{Color, Format, FormattedString};

// Actions to be used directly for key bindings.
pub fn quit(_: &mut Context) -> bool { false }
pub fn prev_page(context: &mut Context) -> bool {
    context.current_session().scrollback_buf.increment_index(1);
    true
}
pub fn next_page(context: &mut Context) -> bool {
    context.current_session().scrollback_buf.decrement_index(1);
    true
}
pub fn backspace_input(context: &mut Context) -> bool {
    let sess = context.current_session();
    let cursor = sess.cursor_index;
    if cursor > 0 {
        let index = sess.history.index();
        sess.history.data.get_recent_mut(index).remove(cursor - 1);
        sess.cursor_index -= 1;
    }
    true
}
pub fn delete_input_char(context: &mut Context) -> bool {
    let sess = context.current_session();
    let input_len = sess.history.data.get_recent(
        sess.history.index()).len();
    let cursor = sess.cursor_index;
    if cursor < input_len {
        let index = sess.history.index();
        sess.history.data.get_recent_mut(index).remove(cursor);
    }
    true
}
pub fn send_input(context: &mut Context) -> bool {
    // Send the input to the server.
    let send_data = {
        let sess = context.current_session();
        let mut send_data = String::new();
        send_data.push_str(&formatted_string::to_string(
            sess.history.data.get_recent(sess.history.index())));
        send_data.push_str("\r\n");
        sess.connection.write(send_data.as_bytes()); // TODO: Check result.
        send_data
    };

    // Add the input to the scrollback buffer.
    write_scrollback(context,
        formatted_string::with_color(&send_data, Color::Yellow));

    // Add the input to the history.
    let sess = context.current_session();
    if sess.history.index() > 0 {
        sess.history.reset_index();
        sess.history.data.get_recent_mut(0).clear();
        sess.history.data.push(
            formatted_string::with_format(&send_data, Format::default()));
    } else {
        sess.history.data.push(FormattedString::new());
    }

    // Reset the cursor.
    sess.cursor_index = 0;
    true
}
pub fn cursor_left(context: &mut Context) -> bool {
    let sess = context.current_session();
    let cursor = sess.cursor_index;
    if cursor > 0 {
        sess.cursor_index -= 1;
    }
    true
}
pub fn cursor_right(context: &mut Context) -> bool {
    let sess = context.current_session();
    let input_len = sess.history.data.get_recent(
        sess.history.index()).len();
    let cursor = sess.cursor_index;
    if cursor < input_len {
        sess.cursor_index += 1;
    }
    true
}
pub fn cursor_up(context: &mut Context) -> bool {
    let sess = context.current_session();
    sess.history.increment_index(1);
    sess.cursor_index = sess.history.data.get_recent(
        sess.history.index()).len();
    true
}
pub fn cursor_down(context: &mut Context) -> bool {
    let sess = context.current_session();
    sess.history.decrement_index(1);
    sess.cursor_index = sess.history.data.get_recent(
        sess.history.index()).len();
    true
}
pub fn delete_to_cursor(context: &mut Context) -> bool {
    let sess = context.current_session();
    let curr_line = sess.history.data.get_recent_mut(0);
    let after_cursor = curr_line.split_off(sess.cursor_index);
    curr_line.clear();
    curr_line.extend(after_cursor);
    sess.cursor_index = 0;
    true
}

// Actions with arguments.
pub fn write_scrollback(context: &mut Context, data: FormattedString) {
    for (ch, format) in data {
        match ch {
            '\r' => (),
            '\n' => context.current_session().scrollback_buf.data.
                push(FormattedString::new()),
            _ => context.current_session().scrollback_buf.data.
                get_recent_mut(0).push((ch, format))
        }
    }
}
pub fn insert_input_char(context: &mut Context, ch: char) {
    let sess = context.current_session();
    let hist_index = sess.history.index();
    sess.history.data.get_recent_mut(hist_index).insert(
        sess.cursor_index, (ch, Format::default()));
    sess.cursor_index += 1;
}
