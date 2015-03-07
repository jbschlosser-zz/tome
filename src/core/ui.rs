extern crate ncurses;

use color_char::{ColorChar, Color, Style};
use line_buffer::LineBuffer;
use std::default::Default;
use std::time::duration::Duration;

static BLACK_ON_DEFAULT_BG: i16 = 1;
static RED_ON_DEFAULT_BG: i16 = 2;
static GREEN_ON_DEFAULT_BG: i16 = 3;
static YELLOW_ON_DEFAULT_BG: i16 = 4;
static BLUE_ON_DEFAULT_BG: i16 = 5;
static MAGENTA_ON_DEFAULT_BG: i16 = 6;
static CYAN_ON_DEFAULT_BG: i16 = 7;
static WHITE_ON_DEFAULT_BG: i16 = 8;
static INPUT_LINE_COLOR_PAIR: i16 = 9;

fn convert_char(ch: ColorChar) -> ncurses::chtype {
    // Handle the fg color.
    let mut out_char = ch.ch as ncurses::chtype;
    match ch.attrs.fg_color {
        Color::Black => out_char = out_char | ncurses::COLOR_PAIR(BLACK_ON_DEFAULT_BG),
        Color::Red => out_char = out_char | ncurses::COLOR_PAIR(RED_ON_DEFAULT_BG),
        Color::Green => out_char = out_char | ncurses::COLOR_PAIR(GREEN_ON_DEFAULT_BG),
        Color::Yellow => out_char = out_char | ncurses::COLOR_PAIR(YELLOW_ON_DEFAULT_BG),
        Color::Blue => out_char = out_char | ncurses::COLOR_PAIR(BLUE_ON_DEFAULT_BG),
        Color::Magenta => out_char = out_char |
            ncurses::COLOR_PAIR(MAGENTA_ON_DEFAULT_BG),
        Color::Cyan => out_char = out_char | ncurses::COLOR_PAIR(CYAN_ON_DEFAULT_BG),
        Color::White => out_char = out_char | ncurses::COLOR_PAIR(WHITE_ON_DEFAULT_BG),
        _ => ()
    };

    // TODO: Handle the bg color.

    // Handle the style.
    match ch.attrs.style {
        Style::Normal => out_char = out_char | ncurses::A_NORMAL(),
        Style::Bold => out_char = out_char | ncurses::A_BOLD(),
        Style::Standout => out_char = out_char | ncurses::A_REVERSE(),
    }

    return out_char;
}

pub struct UserInterface {
    output_win: ncurses::WINDOW,
    input_win: ncurses::WINDOW
}

impl UserInterface {
    pub fn init() -> UserInterface {
        ncurses::initscr();
        ncurses::keypad(ncurses::stdscr, true);
        ncurses::cbreak();
        ncurses::noecho();

        // Init colors.
        ncurses::start_color();
        ncurses::use_default_colors();
        ncurses::init_pair(BLACK_ON_DEFAULT_BG, 0, -1);
        ncurses::init_pair(RED_ON_DEFAULT_BG, 1, -1);
        ncurses::init_pair(GREEN_ON_DEFAULT_BG, 2, -1);
        ncurses::init_pair(YELLOW_ON_DEFAULT_BG, 3, -1);
        ncurses::init_pair(BLUE_ON_DEFAULT_BG, 4, -1);
        ncurses::init_pair(MAGENTA_ON_DEFAULT_BG, 5, -1);
        ncurses::init_pair(CYAN_ON_DEFAULT_BG, 6, -1);
        ncurses::init_pair(WHITE_ON_DEFAULT_BG, 7, -1);
        ncurses::init_pair(INPUT_LINE_COLOR_PAIR, 0, 6);

        let ui_width = UserInterface::width() as i32;
        let ui_height = UserInterface::height() as i32;
        let output_win = ncurses::newwin(ui_height - 1, ui_width, 0, 0);
        ncurses::scrollok(output_win, true);
        let input_win = ncurses::newwin(1, ui_width, ui_height - 1, 0);
        ncurses::wbkgd(input_win, ncurses::COLOR_PAIR(INPUT_LINE_COLOR_PAIR));
        UserInterface {
            output_win: output_win,
            input_win: input_win
        }
    }
    pub fn teardown() {
        ncurses::endwin();
    }
    pub fn update(&mut self, output_lines: &[&[ColorChar]], input_line: &[&[ColorChar]]) {
        // Write the output buffer.
        ncurses::werase(self.output_win);
        UserInterface::write_lines_to_window(&self.output_win, output_lines);
        ncurses::wrefresh(self.output_win);

        // Write the input line.
        ncurses::werase(self.input_win);
        UserInterface::write_lines_to_window(&self.input_win, input_line);
        ncurses::wrefresh(self.input_win);
    }
    /*pub fn resize(&mut self) {
    }*/
    fn write_lines_to_window(win: &ncurses::WINDOW, lines: &[&[ColorChar]]) {
        for i in 0..lines.len() {
            for ch in lines[i] {
                ncurses::waddch(*win, convert_char(*ch));
            }
            if i != lines.len() - 1 { ncurses::waddch(*win, 0xA); }
        }
    }
    pub fn check_for_event(&self) -> i32 {
        ncurses::getch()
    }
    pub fn width() -> usize {
        let mut x = 0;
        let mut y = 0;
        ncurses::getmaxyx(ncurses::stdscr, &mut y, &mut x);
        return x as usize;
    }
    pub fn height() -> usize {
        let mut x = 0;
        let mut y = 0;
        ncurses::getmaxyx(ncurses::stdscr, &mut y, &mut x);
        return y as usize;
    }
    fn window_width(win: &ncurses::WINDOW) -> usize {
        let mut x = 0;
        let mut y = 0;
        ncurses::getmaxyx(*win, &mut y, &mut x);
        return x as usize;
    }
    fn window_height(win: &ncurses::WINDOW) -> usize {
        let mut x = 0;
        let mut y = 0;
        ncurses::getmaxyx(*win, &mut y, &mut x);
        return y as usize;
    }
}
