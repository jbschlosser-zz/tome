extern crate test;

use super::*;
use self::test::Bencher;

#[test]
fn esc_sequence_empty() {
    let mut seq = EscSequence::new();
    assert!(!seq.in_progress());
    assert_eq!(seq.update(ESC_SEQUENCE_BEGIN), None);
    assert!(seq.in_progress());
    assert_eq!(seq.update(ESC_SEQUENCE_END).unwrap(), "\u{1b}m");
    assert!(!seq.in_progress());
}

#[test]
fn esc_sequence_full() {
    let mut seq = EscSequence::new();
    assert!(!seq.in_progress());
    assert_eq!(seq.update(ESC_SEQUENCE_BEGIN), None);
    assert!(seq.in_progress());
    assert_eq!(seq.update('[' as u8), None);
    assert!(seq.in_progress());
    assert_eq!(seq.update('1' as u8), None);
    assert!(seq.in_progress());
    assert_eq!(seq.update(';' as u8), None);
    assert!(seq.in_progress());
    assert_eq!(seq.update('3' as u8), None);
    assert!(seq.in_progress());
    assert_eq!(seq.update('1' as u8), None);
    assert!(seq.in_progress());
    assert_eq!(seq.update(ESC_SEQUENCE_END).unwrap(), "\u{1b}[1;31m");
    assert!(!seq.in_progress());
}

#[test]
fn parse_tests() {
    // Error cases.
    assert_eq!(EscSequence::parse(""), (None, None, None));
    assert_eq!(EscSequence::parse("fkjtkekthgfgo"), (None, None, None));
    assert_eq!(EscSequence::parse("\u{1b}m"), (None, None, None));
    assert_eq!(EscSequence::parse("\u{1b}[123m"), (None, None, None));

    // Success cases.
    assert_eq!(EscSequence::parse("\u{1b}[m"),
        (Some(Style::Normal), Some(Color::Default), Some(Color::Default)));
    assert_eq!(EscSequence::parse("\u{1b}[0m"),
        (Some(Style::Normal), Some(Color::Default), Some(Color::Default)));
    assert_eq!(EscSequence::parse("\u{1b}[1m"), (Some(Style::Bold), None, None));
    assert_eq!(EscSequence::parse("\u{1b}[1;31m"),
        (Some(Style::Bold), Some(Color::Red), None));
    assert_eq!(EscSequence::parse("\u{1b}[31;1m"),
        (Some(Style::Bold), Some(Color::Red), None));
    assert_eq!(EscSequence::parse("\u{1b}[1;31;34;5;2;42m"),
        (Some(Style::Bold), Some(Color::Blue), Some(Color::Green)));
    assert_eq!(EscSequence::parse("\u{1b}[1;31;;;2;42m"),
        (Some(Style::Bold), Some(Color::Red), Some(Color::Green)));
    assert_eq!(EscSequence::parse("\u{1b}[42;;;31;;;2;;1m"),
        (Some(Style::Bold), Some(Color::Red), Some(Color::Green)));
}

#[test]
fn line_buffer_get_lines() {
    let mut line_buffer = LineBuffer::new(Some(3), None);
    let attrs = Attributes {
        style: Style::Normal, fg_color: Color::Default, bg_color: Color::Default
    };
    let test_str = make_color_string("Hello\nworld\nthis\nis\na\ntest", attrs);
    line_buffer.insert(&test_str);

    assert_eq!(line_buffer.get_lines(0, 1),
        vec![make_color_string("test", attrs)]);
    assert_eq!(line_buffer.get_lines(1, 1),
        vec![make_color_string("a", attrs)]);
    assert_eq!(line_buffer.get_lines(2, 1),
        vec![make_color_string("is", attrs)]);
    assert_eq!(line_buffer.get_lines(3, 1), Vec::<&[ColorChar]>::new());

    assert_eq!(line_buffer.get_lines(0, 2),
        vec![make_color_string("a", attrs),
            make_color_string( "test", attrs)]);
    assert_eq!(line_buffer.get_lines(1, 2),
        vec![make_color_string("is", attrs),
            make_color_string( "a", attrs)]);
    assert_eq!(line_buffer.get_lines(2, 2),
        vec![make_color_string("is", attrs)]);
    assert_eq!(line_buffer.get_lines(3, 2), Vec::<&[ColorChar]>::new());

    assert_eq!(line_buffer.get_lines(0, 3),
        vec![make_color_string("is", attrs),
            make_color_string("a", attrs),
            make_color_string( "test", attrs)]);
    assert_eq!(line_buffer.get_lines(1, 3),
        vec![make_color_string("is", attrs),
            make_color_string( "a", attrs)]);
    assert_eq!(line_buffer.get_lines(2, 3),
        vec![make_color_string("is", attrs)]);
    assert_eq!(line_buffer.get_lines(3, 3), Vec::<&[ColorChar]>::new());
    assert_eq!(line_buffer.get_lines(4, 3), Vec::<&[ColorChar]>::new());
    assert_eq!(line_buffer.get_lines(5, 3), Vec::<&[ColorChar]>::new());
}

#[test]
fn line_buffer_max_length() {
    let mut line_buffer = LineBuffer::new(None, Some(5));
    let attrs = Attributes {
        style: Style::Normal, fg_color: Color::Default, bg_color: Color::Default
    };
    let test_str = make_color_string("Hello world this is a test", attrs);
    line_buffer.insert(&test_str);
    assert_eq!(line_buffer.get_lines(0, 50),
        vec![make_color_string("Hello", attrs),
        make_color_string(" worl", attrs),
        make_color_string("d thi", attrs),
        make_color_string("s is ", attrs),
        make_color_string("a tes", attrs),
        make_color_string("t", attrs)]);
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
    let attrs = Attributes {
        style: Style::Normal, fg_color: Color::Default, bg_color: Color::Default
    };
    let cs = make_color_string(&s, attrs);
    b.iter(|| {
        buffer.insert(&cs)
    });
}

#[bench]
fn bench_get_lines(b: &mut Bencher) {
    let mut buffer = LineBuffer::new(None, None);
    let attrs = Attributes {
        style: Style::Normal, fg_color: Color::Default, bg_color: Color::Default
    };
    for _ in 0..50000 {
        let cs =
            make_color_string("this is a test of the emergency broadcast system\n", attrs);
        buffer.insert(&cs);
    }
    b.iter(|| {
        buffer.get_lines(1000, 50)
    });
}
