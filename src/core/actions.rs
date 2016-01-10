use context::Context;
use std::io::Write;
use formatted_string;
use formatted_string::{Color, Format, FormattedString};

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
    // Send the input to the server.
    let send_data = {
        let mut send_data = String::new();
        send_data.push_str(&formatted_string::to_string(
            context.history.data.get_recent(context.history.index())));
        send_data.push_str("\r\n");
        context.current_session_mut().connection.write(
            send_data.as_bytes()); // TODO: Check result.
        send_data
    };

    // Add the input to the scrollback buffer.
    write_scrollback(context,
        formatted_string::with_color(&send_data, Color::Yellow));

    // Add the input to the history.
    if context.history.index() > 0 {
        context.history.reset_index();
        context.history.data.get_recent_mut(0).clear();
        context.history.data.push(
            formatted_string::with_format(&send_data, Format::default()));
    } else {
        context.history.data.push(FormattedString::new());
    }

    // Reset the cursor.
    context.cursor_index = 0;
    true
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
pub fn cursor_up(context: &mut Context) -> bool {
    context.history.increment_index(1);
    context.cursor_index = context.history.data.get_recent(
        context.history.index()).len();
    true
}
pub fn cursor_down(context: &mut Context) -> bool {
    context.history.decrement_index(1);
    context.cursor_index = context.history.data.get_recent(
        context.history.index()).len();
    true
}
pub fn delete_to_cursor(context: &mut Context) -> bool {
    let curr_line = context.history.data.get_recent_mut(0);
    let after_cursor = curr_line.split_off(context.cursor_index);
    curr_line.clear();
    curr_line.extend(after_cursor);
    context.cursor_index = 0;
    true
}

// Actions with arguments.
pub fn write_scrollback(context: &mut Context, data: FormattedString) {
    for (ch, format) in data {
        match ch {
            '\r' => (),
            '\n' => context.current_session_mut().scrollback_buf.data.
                push(FormattedString::new()),
            _ => context.current_session_mut().scrollback_buf.data.
                get_recent_mut(0).push((ch, format))
        }
    }
}
pub fn insert_input_char(context: &mut Context, ch: char) {
    let hist_index = context.history.index();
    context.history.data.get_recent_mut(hist_index).insert(
        context.cursor_index, (ch, Format::default()));
    context.cursor_index += 1;
}
