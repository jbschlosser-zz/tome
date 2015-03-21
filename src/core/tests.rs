extern crate test;

use super::*;
use self::test::Bencher;

#[test]
fn parse_esc_seq_tests() {
    // Start of esc sequence.
    assert_eq!(parse_esc_seq(&ParseState::NotInProgress, b'\x1b'),
        ParseState::InProgress(b"\x1b".to_vec()));

    // Middle of esc sequence.
    assert_eq!(parse_esc_seq(
        &ParseState::InProgress(b"\x1b[0;".to_vec()), b'5'),
        ParseState::InProgress(b"\x1b[0;5".to_vec()));

    // End of esc sequence.
    assert_eq!(parse_esc_seq(
        &ParseState::InProgress(b"\x1b[0;32".to_vec()), b'm'),
        ParseState::Success(b"\x1b[0;32m".to_vec()));

    // Esc sequence too big.
    assert_eq!(parse_esc_seq(
        &ParseState::InProgress(b"\x1babcdefghijklmno".to_vec()), b'p'),
        ParseState::Error(b"\x1babcdefghijklmnop".to_vec()));
}

#[test]
fn interpret_esc_seq_tests() {
    // Error cases.
    assert_eq!(interpret_esc_seq(b""), (None, None, None));
    assert_eq!(interpret_esc_seq(b"fkjtkekthgfgo"), (None, None, None));
    assert_eq!(interpret_esc_seq(b"\x1bm"), (None, None, None));
    assert_eq!(interpret_esc_seq(b"\x1b[123m"), (None, None, None));

    // Success cases.
    assert_eq!(interpret_esc_seq(b"\x1b[m"),
        (Some(Style::Normal), Some(Color::Default), Some(Color::Default)));
    assert_eq!(interpret_esc_seq(b"\x1b[0m"),
        (Some(Style::Normal), Some(Color::Default), Some(Color::Default)));
    assert_eq!(interpret_esc_seq(b"\x1b[1m"), (Some(Style::Bold), None, None));
    assert_eq!(interpret_esc_seq(b"\x1b[1;31m"),
        (Some(Style::Bold), Some(Color::Red), None));
    assert_eq!(interpret_esc_seq(b"\x1b[31;1m"),
        (Some(Style::Bold), Some(Color::Red), None));
    assert_eq!(interpret_esc_seq(b"\x1b[1;31;34;5;2;42m"),
        (Some(Style::Bold), Some(Color::Blue), Some(Color::Green)));
    assert_eq!(interpret_esc_seq(b"\x1b[1;31;;;2;42m"),
        (Some(Style::Bold), Some(Color::Red), Some(Color::Green)));
    assert_eq!(interpret_esc_seq(b"\x1b[42;;;31;;;2;;1m"),
        (Some(Style::Bold), Some(Color::Red), Some(Color::Green)));
}

#[test]
fn line_buffer_get_lines() {
    let mut line_buffer = LineBuffer::new(Some(3), None);
    let format = Format {
        style: Style::Normal, fg_color: Color::Default, bg_color: Color::Default
    };
    let test_str = FormattedString::with_format("Hello\nworld\nthis\nis\na\ntest", format);
    line_buffer.insert(&test_str);

    assert_eq!(line_buffer.get_lines(0, 1),
        vec![&FormattedString::with_format("test", format)]);
    assert_eq!(line_buffer.get_lines(1, 1),
        vec![&FormattedString::with_format("a", format)]);
    assert_eq!(line_buffer.get_lines(2, 1),
        vec![&FormattedString::with_format("is", format)]);
    assert_eq!(line_buffer.get_lines(3, 1), Vec::<&FormattedString>::new());

    assert_eq!(line_buffer.get_lines(0, 2),
        vec![&FormattedString::with_format("a", format),
            &FormattedString::with_format( "test", format)]);
    assert_eq!(line_buffer.get_lines(1, 2),
        vec![&FormattedString::with_format("is", format),
            &FormattedString::with_format( "a", format)]);
    assert_eq!(line_buffer.get_lines(2, 2),
        vec![&FormattedString::with_format("is", format)]);
    assert_eq!(line_buffer.get_lines(3, 2), Vec::<&FormattedString>::new());

    assert_eq!(line_buffer.get_lines(0, 3),
        vec![&FormattedString::with_format("is", format),
            &FormattedString::with_format("a", format),
            &FormattedString::with_format( "test", format)]);
    assert_eq!(line_buffer.get_lines(1, 3),
        vec![&FormattedString::with_format("is", format),
            &FormattedString::with_format( "a", format)]);
    assert_eq!(line_buffer.get_lines(2, 3),
        vec![&FormattedString::with_format("is", format)]);
    assert_eq!(line_buffer.get_lines(3, 3), Vec::<&FormattedString>::new());
    assert_eq!(line_buffer.get_lines(4, 3), Vec::<&FormattedString>::new());
    assert_eq!(line_buffer.get_lines(5, 3), Vec::<&FormattedString>::new());
}

#[test]
fn line_buffer_max_length() {
    let mut line_buffer = LineBuffer::new(None, Some(5));
    let format = Format {
        style: Style::Normal, fg_color: Color::Default, bg_color: Color::Default
    };
    let test_str = FormattedString::with_format("Hello world this is a test", format);
    line_buffer.insert(&test_str);
    assert_eq!(line_buffer.get_lines(0, 50),
        vec![&FormattedString::with_format("Hello", format),
        &FormattedString::with_format(" worl", format),
        &FormattedString::with_format("d thi", format),
        &FormattedString::with_format("s is ", format),
        &FormattedString::with_format("a tes", format),
        &FormattedString::with_format("t", format)]);
}

#[test]
fn formatted_string_tests() {
    let mut str1 = FormattedString::new();
    assert_eq!(str1.to_str(), "");
    assert_eq!(str1.formats(), vec![]);
    str1.push('a', Format::with_fg(Color::Red));
    str1.push('b', Format::with_fg(Color::Blue));
    assert_eq!(str1.to_str(), "ab");
    assert_eq!(str1.formats(),
        vec![Format::with_fg(Color::Red), Format::with_fg(Color::Blue)]);

    let str2 = FormattedString::with_format("testing", Format::default());
    assert_eq!(str2.to_str(), "testing");
    assert_eq!(str2.formats(), vec![Format::default(); 7]);

    let str3 = FormattedString::with_color("hello world", Color::Green);
    assert_eq!(str3.to_str(), "hello world");
    assert_eq!(str3.formats(), vec![Format::with_fg(Color::Green); 11]);
}

#[bench]
fn bench_handle_server_data(b: &mut Bencher) {
    let mut bb = [0; 100000];
    let mut selector = 0;
    let mut sub = 0;
    for i in 0..bb.len() {
        match selector {
            0 => {
                // Telnet
                match sub {
                    0 => { bb[i] = 0xFF; sub += 1; },
                    1 => { bb[i] = 0xFD; sub += 1; },
                    2 => { bb[i] = 0x5; sub += 1; },
                    _ => { selector = 1; sub = 0; }
                }
            },
            1 => {
                // ESC sequence.
                match sub {
                    0 => { bb[i] = 0x1B; sub += 1; },
                    1 => { bb[i] = 0x5B; sub += 1; },
                    2 => { bb[i] = 0x33; sub += 1; },
                    3 => { bb[i] = 0x31; sub += 1; },
                    4 => { bb[i] = 0x6D; sub += 1; },
                    _ => { selector = 2; sub = 0; }
                }
            },
            2 => { bb[i] = 0x57; selector = 0; sub = 0; }
            _ => panic!("logic error")
        }
    }
    let mut session = Session::new();
    b.iter(|| {
        handle_server_data(&bb, &mut session)
    });
}

#[bench]
fn bench_insert(b: &mut Bencher) {
    let mut buffer = LineBuffer::new(None, None);
    let mut s = String::new();
    for _ in 0..10000 {
        s.push_str("this is a test of the emergency broadcast system\n");
    }
    let format = Format {
        style: Style::Normal, fg_color: Color::Default, bg_color: Color::Default
    };
    let cs = FormattedString::with_format(&s, format);
    b.iter(|| {
        buffer.insert(&cs)
    });
}

#[bench]
fn bench_get_lines(b: &mut Bencher) {
    let mut buffer = LineBuffer::new(None, None);
    let format = Format {
        style: Style::Normal, fg_color: Color::Default, bg_color: Color::Default
    };
    for _ in 0..50000 {
        let cs =
            FormattedString::with_format("this is a test of the emergency broadcast system\n", format);
        buffer.insert(&cs);
    }
    b.iter(|| {
        buffer.get_lines(1000, 50)
    });
}
