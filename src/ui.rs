extern crate ncurses;

use tome::{FormattedString, Format, Color, Style};

static BLACK_ON_DEFAULT_BG: i16 = 1;
static RED_ON_DEFAULT_BG: i16 = 2;
static GREEN_ON_DEFAULT_BG: i16 = 3;
static YELLOW_ON_DEFAULT_BG: i16 = 4;
static BLUE_ON_DEFAULT_BG: i16 = 5;
static MAGENTA_ON_DEFAULT_BG: i16 = 6;
static CYAN_ON_DEFAULT_BG: i16 = 7;
static WHITE_ON_DEFAULT_BG: i16 = 8;
static INPUT_LINE_COLOR_PAIR: i16 = 9;

fn convert_char(ch: char, format: Format) -> ncurses::chtype {
    // Handle the fg color.
    let mut out_char = ch as ncurses::chtype;
    match format.fg_color {
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
    match format.style {
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
        ncurses::keypad(ncurses::stdscr(), true);
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
        ncurses::keypad(output_win, true); 
        let input_win = ncurses::newwin(1, ui_width, ui_height - 1, 0);
        ncurses::keypad(input_win, true); 
        ncurses::wbkgd(input_win, ncurses::COLOR_PAIR(INPUT_LINE_COLOR_PAIR));
        UserInterface {
            output_win: output_win,
            input_win: input_win
        }
    }
    pub fn restart(&mut self) {
        // Shut it down.
        self.teardown();
        ncurses::refresh();
        ncurses::clear();

        // Start it up.
        let new_ui = UserInterface::init();

        // Set up the new windows.
        self.input_win = new_ui.input_win;
        self.output_win = new_ui.output_win;
    }
    pub fn teardown(&mut self) {
        ncurses::delwin(self.input_win);
        ncurses::delwin(self.output_win);
        ncurses::endwin();
    }
    pub fn update<'a, I: Iterator<Item=&'a FormattedString>>(&mut self,
        output_lines: I,
        input_line: I,
        cursor_index: usize)
    {
        // Write the output buffer.
        ncurses::werase(self.output_win);
        UserInterface::write_lines_to_window(
            &self.output_win, output_lines.take(self.output_win_height()));
        ncurses::wrefresh(self.output_win);

        // Write the input line.
        ncurses::werase(self.input_win);
        UserInterface::write_lines_to_window(
            &self.input_win, input_line.take(1));
        ncurses::wmove(self.input_win, 0, cursor_index as i32);
        ncurses::wrefresh(self.input_win);
    }
    fn write_lines_to_window<'a, I: Iterator<Item=&'a FormattedString>>(
        win: &ncurses::WINDOW, lines: I)
    {
        for (i, line) in lines.enumerate() {
            if i > 0 {
                ncurses::waddch(*win, 0xA);
            }
            for &(ch, format) in line.iter() {
                ncurses::waddch(*win, convert_char(ch, format));
            }
        }
    }
    pub fn width() -> usize { Self::win_width(ncurses::stdscr()) }
    pub fn height() -> usize { Self::win_height(ncurses::stdscr()) }
    pub fn output_win_height(&self) -> usize {
        Self::win_height(self.output_win)
    }
    fn win_width(win: ncurses::WINDOW) -> usize {
        let mut x = 0;
        let mut y = 0;
        ncurses::getmaxyx(win, &mut y, &mut x);
        return x as usize;
    }
    fn win_height(win: ncurses::WINDOW) -> usize {
        let mut x = 0;
        let mut y = 0;
        ncurses::getmaxyx(win, &mut y, &mut x);
        return y as usize;
    }
}
