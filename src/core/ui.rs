extern crate rustbox;

pub use self::rustbox::Event;
pub use self::rustbox::Event::KeyEvent;
pub use self::rustbox::Event::ResizeEvent;
pub use self::rustbox::EventResult;

use color_char::{ColorChar, Color, Style};
use line_buffer::LineBuffer;
use self::rustbox::{InitOptions, RustBox};
use std::default::Default;
use std::time::duration::Duration;

fn convert_color(input: Color) -> rustbox::Color {
    match input {
        Color::Default => rustbox::Color::Default,
        Color::Black => rustbox::Color::Black,
        Color::Red => rustbox::Color::Red,
        Color::Green => rustbox::Color::Green,
        Color::Yellow => rustbox::Color::Yellow,
        Color::Blue => rustbox::Color::Blue,
        Color::Magenta => rustbox::Color::Magenta,
        Color::Cyan => rustbox::Color::Cyan,
        Color::White => rustbox::Color::White
    }
}

fn convert_style(input: Style) -> rustbox::Style {
    match input {
        Style::Normal => rustbox::RB_NORMAL,
        Style::Bold => rustbox::RB_BOLD,
        Style::Standout => rustbox::RB_REVERSE,
    }
}

struct Window {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    bg_color: rustbox::Color,
    fg_color: rustbox::Color
}

pub struct UserInterface {
    rustbox: RustBox,
    output_win: Window,
    input_line: Window
}

impl UserInterface {
    pub fn init() -> UserInterface {
        let rustbox = match RustBox::init(InitOptions {
                buffer_stderr: true,
                ..Default::default()
            }) {
                Result::Ok(v) => v,
                Result::Err(e) => panic!("{}", e)
            };
        let ui_width = rustbox.width();
        let ui_height = rustbox.height();
        UserInterface {
            rustbox: rustbox,
            output_win: Window {x: 0, y: 0, width: ui_width, height: ui_height - 1,
                bg_color: rustbox::Color::Blue, fg_color: rustbox::Color::Default},
            input_line: Window {x: 0, y: ui_height - 1, width: ui_width, height: 1,
                bg_color: rustbox::Color::Cyan, fg_color: rustbox::Color::Black}
        }
    }
    pub fn update(&mut self, output_lines: &[&[ColorChar]], input_line: &[&[ColorChar]]) {
        self.rustbox.clear();
        
        // Write the output buffer.
        UserInterface::write_lines_to_window(&self.rustbox, &self.output_win, output_lines);

        // Write the input line.
        UserInterface::write_lines_to_window(&self.rustbox, &self.input_line, input_line);

        self.rustbox.present();
    }
    /*pub fn resize(&mut self) {
        let ui_width = self.rustbox.width();
        let ui_height = self.rustbox.height();
        self.output_win = Window {x: 0, y: 0, width: ui_width, height: ui_height - 1,
            bg_color: rustbox::Color::Blue, fg_color: rustbox::Color::Default};
        self.input_line = Window {x: 0, y: ui_height - 1, width: ui_width, height: 1,
            bg_color: rustbox::Color::Cyan, fg_color: rustbox::Color::Black};
    }*/
    fn write_lines_to_window(rustbox: &RustBox, win: &Window, lines: &[&[ColorChar]]) {
        // Fit the lines to the window size.
        let mut screen_buf = LineBuffer::new(Some(win.height), Some(win.width));
        for i in 0..lines.len() {
            screen_buf.insert(&lines[i]);
            if i != lines.len() - 1 { screen_buf.move_to_next_line(); }
        }

        // Write the lines.
        //let mut loc_x = win.x;
        //let mut loc_y = win.y;
        for i in win.x..(win.x + win.width) {
            for j in win.y..(win.y + win.height) {
                rustbox.print(i, j, rustbox::RB_NORMAL, win.fg_color, win.bg_color, " ");
            }
        }
        /*let lines_to_print = screen_buf.get_lines(0, win.height);
        for line in lines_to_print.iter() {
            for ch in line.iter() {
                let style = convert_style(ch.attrs.style);
                let fg_color =
                    if ch.attrs.fg_color == Color::Default {
                        win.fg_color
                    } else {
                        convert_color(ch.attrs.fg_color)
                    };
                let bg_color =
                    if ch.attrs.bg_color == Color::Default {
                        win.bg_color
                    } else {
                        convert_color(ch.attrs.bg_color)
                    };
                rustbox.print(loc_x, loc_y, style, fg_color,
                    bg_color, &ch.ch.to_string());
                loc_x += 1;
            }
            loc_x = win.x;
            loc_y += 1;
        }*/
    }
    pub fn check_for_event(&self) -> EventResult<Event> {
        self.rustbox.peek_event(Duration::milliseconds(0))
    }
    pub fn width(&self) -> usize {
        self.rustbox.width()
    }
    pub fn height(&self) -> usize {
        self.rustbox.height()
    }
}
