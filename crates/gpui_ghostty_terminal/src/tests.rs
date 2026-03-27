use gpui::{KeyBinding, KeyContext, Keymap, Keystroke, actions};
use std::any::TypeId;

use crate::{TerminalConfig, TerminalSession};

actions!(tab_shadow_test, [RootTab, TerminalTab]);

fn osc_color_response(ps: u8, (r, g, b): (u8, u8, u8)) -> String {
    let r16 = u16::from(r) * 0x0101;
    let g16 = u16::from(g) * 0x0101;
    let b16 = u16::from(b) * 0x0101;

    format!("\x1b]{};rgb:{:04x}/{:04x}/{:04x}\x1b\\", ps, r16, g16, b16)
}

fn viewport_index_for_cell(viewport: &str, row: u16, col: u16) -> usize {
    let row = row.max(1) as usize;
    let col = col.max(1) as usize;

    use unicode_width::UnicodeWidthChar as _;

    let mut current_row = 1usize;
    let mut offset = 0usize;

    for segment in viewport.split_inclusive('\n') {
        let line = segment.strip_suffix('\n').unwrap_or(segment);

        if current_row == row {
            if col == 1 {
                return offset;
            }

            let mut current_col = 1usize;
            for (byte_index, ch) in line.char_indices() {
                let width = ch.width().unwrap_or(0);
                if width == 0 {
                    continue;
                }

                if current_col == col {
                    return offset + byte_index;
                }

                let next_col = current_col.saturating_add(width);
                if col < next_col {
                    return offset + byte_index;
                }

                current_col = next_col;
            }

            return offset + line.len();
        }

        offset = offset.saturating_add(segment.len());
        current_row += 1;
    }

    viewport.len()
}

#[test]
fn terminal_tab_binding_shadows_root_tab_binding() {
    let mut keymap = Keymap::default();
    keymap.add_bindings([
        KeyBinding::new("tab", RootTab, Some("Root")),
        KeyBinding::new("tab", TerminalTab, Some("Terminal")),
    ]);

    let mut root = KeyContext::default();
    root.add("Root");
    let mut terminal = KeyContext::default();
    terminal.add("Terminal");

    let (bindings, pending) =
        keymap.bindings_for_input(&[Keystroke::parse("tab").unwrap()], &[root, terminal]);

    assert!(!pending);
    assert_eq!(
        bindings[0].action().as_any().type_id(),
        TypeId::of::<TerminalTab>()
    );
}

#[test]
fn tracks_bracketed_paste_mode_from_output() {
    let mut session = TerminalSession::new(TerminalConfig::default()).unwrap();
    assert!(!session.bracketed_paste_enabled());

    session.feed(b"\x1b[?2004h").unwrap();
    assert!(session.bracketed_paste_enabled());

    session.feed(b"\x1b[?2004l").unwrap();
    assert!(!session.bracketed_paste_enabled());
}

#[test]
fn tracks_mouse_reporting_mode_from_output() {
    let mut session = TerminalSession::new(TerminalConfig::default()).unwrap();
    assert!(!session.mouse_reporting_enabled());
    assert!(!session.mouse_sgr_enabled());

    session.feed(b"\x1b[?1000;1006h").unwrap();
    assert!(session.mouse_reporting_enabled());
    assert!(session.mouse_sgr_enabled());

    session.feed(b"\x1b[?1000l").unwrap();
    assert!(!session.mouse_reporting_enabled());
    assert!(session.mouse_sgr_enabled());

    session.feed(b"\x1b[?1006l").unwrap();
    assert!(!session.mouse_sgr_enabled());
}

#[test]
fn viewport_index_maps_row_and_column_to_byte_index() {
    let viewport = "abc\ndef";
    assert_eq!(viewport_index_for_cell(viewport, 1, 1), 0);
    assert_eq!(viewport_index_for_cell(viewport, 1, 2), 1);
    assert_eq!(viewport_index_for_cell(viewport, 1, 4), 3);
    assert_eq!(viewport_index_for_cell(viewport, 2, 1), 4);
    assert_eq!(viewport_index_for_cell(viewport, 2, 3), 6);
}

#[test]
fn viewport_index_accounts_for_wide_characters() {
    let viewport = "Ｗa\n";
    assert_eq!(viewport_index_for_cell(viewport, 1, 1), 0);
    assert_eq!(viewport_index_for_cell(viewport, 1, 2), 0);
    assert_eq!(viewport_index_for_cell(viewport, 1, 3), "Ｗ".len());
    assert_eq!(viewport_index_for_cell(viewport, 1, 4), "Ｗ".len() + 1);
}

#[test]
fn tracks_modes_across_chunk_boundaries() {
    let mut session = TerminalSession::new(TerminalConfig::default()).unwrap();
    session.feed(b"\x1b[?1000;").unwrap();
    assert!(!session.mouse_reporting_enabled());

    session.feed(b"1006h").unwrap();
    assert!(session.mouse_reporting_enabled());
    assert!(session.mouse_sgr_enabled());
}

#[test]
fn tracks_osc_title_across_chunk_boundaries() {
    let mut session = TerminalSession::new(TerminalConfig::default()).unwrap();
    session.feed(b"\x1b]0;hi").unwrap();
    assert!(session.title().is_none());

    session.feed(b"\x07").unwrap();
    assert_eq!(session.title(), Some("hi"));
}

#[test]
fn tracks_osc_52_clipboard_across_chunk_boundaries() {
    let mut session = TerminalSession::new(TerminalConfig::default()).unwrap();
    session.feed(b"\x1b]52;c;").unwrap();
    assert!(session.take_clipboard_write().is_none());

    session.feed(b"aGk=\x07").unwrap();
    assert_eq!(session.take_clipboard_write().as_deref(), Some("hi"));
}

#[test]
fn responds_to_csi_6n_cursor_position_request() {
    let mut session = TerminalSession::new(TerminalConfig::default()).unwrap();
    let mut response = Vec::new();

    session
        .feed_with_pty_responses(b"hi\x1b[6n", |bytes| {
            response.extend_from_slice(bytes);
        })
        .unwrap();

    assert_eq!(response, b"\x1b[1;3R");
}

#[test]
fn responds_to_csi_6n_across_chunk_boundaries() {
    let mut session = TerminalSession::new(TerminalConfig::default()).unwrap();
    let mut response = Vec::new();

    session
        .feed_with_pty_responses(b"hi\x1b[", |bytes| {
            response.extend_from_slice(bytes);
        })
        .unwrap();
    assert!(response.is_empty());

    session
        .feed_with_pty_responses(b"6n", |bytes| {
            response.extend_from_slice(bytes);
        })
        .unwrap();

    assert_eq!(response, b"\x1b[1;3R");
}

#[test]
fn responds_to_csi_5n_device_status_request() {
    let mut session = TerminalSession::new(TerminalConfig::default()).unwrap();
    let mut response = Vec::new();

    session
        .feed_with_pty_responses(b"\x1b[5n", |bytes| {
            response.extend_from_slice(bytes);
        })
        .unwrap();

    assert_eq!(response, b"\x1b[0n");
}

#[test]
fn responds_to_csi_5n_across_chunk_boundaries() {
    let mut session = TerminalSession::new(TerminalConfig::default()).unwrap();
    let mut response = Vec::new();

    session
        .feed_with_pty_responses(b"\x1b[", |bytes| {
            response.extend_from_slice(bytes);
        })
        .unwrap();
    assert!(response.is_empty());

    session
        .feed_with_pty_responses(b"5n", |bytes| {
            response.extend_from_slice(bytes);
        })
        .unwrap();

    assert_eq!(response, b"\x1b[0n");
}

#[test]
fn responds_to_osc_10_default_foreground_color_query() {
    let mut session = TerminalSession::new(TerminalConfig::default()).unwrap();
    let mut response = Vec::new();

    session
        .feed_with_pty_responses(b"\x1b]10;?\x1b\\", |bytes| {
            response.extend_from_slice(bytes);
        })
        .unwrap();

    let expected = osc_color_response(10, (0xFF, 0xFF, 0xFF));
    assert_eq!(response, expected.as_bytes());
}

#[test]
fn responds_to_osc_11_default_background_color_query() {
    let mut session = TerminalSession::new(TerminalConfig::default()).unwrap();
    let mut response = Vec::new();

    session
        .feed_with_pty_responses(b"\x1b]11;?\x1b\\", |bytes| {
            response.extend_from_slice(bytes);
        })
        .unwrap();

    let expected = osc_color_response(11, (0x00, 0x00, 0x00));
    assert_eq!(response, expected.as_bytes());
}

#[test]
fn responds_to_osc_10_and_11_use_configured_defaults() {
    let config = TerminalConfig {
        default_fg: ghostty_vt::Rgb {
            r: 0x11,
            g: 0x22,
            b: 0x33,
        },
        default_bg: ghostty_vt::Rgb {
            r: 0x44,
            g: 0x55,
            b: 0x66,
        },
        ..TerminalConfig::default()
    };
    let mut session = TerminalSession::new(config).unwrap();
    let mut response = Vec::new();

    session
        .feed_with_pty_responses(b"\x1b]10;?\x1b\\\x1b]11;?\x1b\\", |bytes| {
            response.extend_from_slice(bytes);
        })
        .unwrap();

    let expected_fg = osc_color_response(10, (0x11, 0x22, 0x33));
    let expected_bg = osc_color_response(11, (0x44, 0x55, 0x66));
    let mut expected = Vec::new();
    expected.extend_from_slice(expected_fg.as_bytes());
    expected.extend_from_slice(expected_bg.as_bytes());
    assert_eq!(response, expected);
}

#[test]
fn responds_to_osc_11_across_chunk_boundaries() {
    let mut session = TerminalSession::new(TerminalConfig::default()).unwrap();
    let mut response = Vec::new();

    session
        .feed_with_pty_responses(b"\x1b]11;?\x1b", |bytes| {
            response.extend_from_slice(bytes);
        })
        .unwrap();
    assert!(response.is_empty());

    session
        .feed_with_pty_responses(b"\\", |bytes| {
            response.extend_from_slice(bytes);
        })
        .unwrap();

    let expected = osc_color_response(11, (0x00, 0x00, 0x00));
    assert_eq!(response, expected.as_bytes());
}

#[test]
fn responds_to_osc_11_query_terminated_by_bel() {
    let mut session = TerminalSession::new(TerminalConfig::default()).unwrap();
    let mut response = Vec::new();

    session
        .feed_with_pty_responses(b"\x1b]11;?\x07", |bytes| {
            response.extend_from_slice(bytes);
        })
        .unwrap();

    let expected = osc_color_response(11, (0x00, 0x00, 0x00));
    assert_eq!(response, expected.as_bytes());
}

#[test]
fn sgr_mouse_encoding_helpers_match_expected_format() {
    assert_eq!(
        crate::view::sgr_mouse_button_value(0, false, false, false, false),
        0
    );
    assert_eq!(
        crate::view::sgr_mouse_button_value(2, true, false, true, true),
        2 + 32 + 8 + 16
    );
    assert_eq!(
        crate::view::sgr_mouse_sequence(0, 1, 1, true),
        "\u{1b}[<0;1;1M"
    );
    assert_eq!(
        crate::view::sgr_mouse_sequence(0, 1, 1, false),
        "\u{1b}[<0;1;1m"
    );
}

#[test]
fn ctrl_c_encodes_to_etx_even_without_key_char() {
    let ctrl_c = Keystroke::parse("ctrl-c").unwrap();
    assert_eq!(crate::view::ctrl_byte_for_keystroke(&ctrl_c), Some(0x03));
}

#[test]
fn does_not_skip_enter_key_when_ime_in_progress() {
    let enter = Keystroke::parse("enter").unwrap();
    assert!(enter.is_ime_in_progress());
    assert!(!crate::view::should_skip_key_down_for_ime(true, &enter));

    let letter = Keystroke::parse("a").unwrap();
    assert!(letter.is_ime_in_progress());
    assert!(crate::view::should_skip_key_down_for_ime(true, &letter));

    let committed = Keystroke::parse("a->a").unwrap();
    assert!(!committed.is_ime_in_progress());
    assert!(!crate::view::should_skip_key_down_for_ime(true, &committed));
}

#[test]
fn byte_index_for_column_in_line_handles_wide_characters() {
    assert_eq!(crate::view::byte_index_for_column_in_line("Ｗa", 1), 0);
    assert_eq!(crate::view::byte_index_for_column_in_line("Ｗa", 2), 0);
    assert_eq!(
        crate::view::byte_index_for_column_in_line("Ｗa", 3),
        "Ｗ".len()
    );
    assert_eq!(
        crate::view::byte_index_for_column_in_line("Ｗa", 4),
        "Ｗ".len() + 1
    );
}

#[test]
fn maps_common_box_drawing_glyphs() {
    for ch in ['─', '│', '┌', '┐', '└', '┘', '├', '┤', '┬', '┴', '┼'] {
        assert!(
            crate::view::box_drawing_mask(ch).is_some(),
            "expected mask for {ch}"
        );
    }
    assert!(crate::view::box_drawing_mask('X').is_none());
}

// OSC 133 shell integration — last command output capture.
//
// Shells emit OSC 133;C after printing the newline that follows the command
// line (cursor already on the first output row) and OSC 133;D after the
// command finishes (cursor on the line after the last output row).
//
// Sequences used below:
//   \x1b]133;C\x07   = OSC 133;C  BEL-terminated
//   \x1b]133;D;0\x07 = OSC 133;D  with exit code 0, BEL-terminated

#[test]
fn osc133_captures_single_line_output() {
    let mut session = TerminalSession::new(TerminalConfig::default()).unwrap();

    // Cursor starts at row 1.  Simulate shell moving to next line before C.
    let mut responses = Vec::<u8>::new();
    // Move cursor to row 2 (simulate shell \r\n after command line).
    session
        .feed_with_pty_responses(b"\r\n", |b| responses.extend_from_slice(b))
        .unwrap();
    // OSC 133;C — output start, cursor is at row 2.
    session
        .feed_with_pty_responses(b"\x1b]133;C\x07", |b| responses.extend_from_slice(b))
        .unwrap();
    // Command output: "hello\r\n" — printed at row 2, cursor moves to row 3.
    session
        .feed_with_pty_responses(b"hello\r\n", |b| responses.extend_from_slice(b))
        .unwrap();
    // OSC 133;D;0 — output end, cursor is at row 3.
    session
        .feed_with_pty_responses(b"\x1b]133;D;0\x07", |b| responses.extend_from_slice(b))
        .unwrap();

    let output = session.take_last_command_output();
    assert_eq!(output.as_deref(), Some("hello"));

    // Consumed — second call returns None.
    assert!(session.take_last_command_output().is_none());
}

#[test]
fn osc133_captures_multi_line_output() {
    let mut session = TerminalSession::new(TerminalConfig::default()).unwrap();

    let mut noop = |_: &[u8]| {};
    session.feed_with_pty_responses(b"\r\n", &mut noop).unwrap();
    session
        .feed_with_pty_responses(b"\x1b]133;C\x07", &mut noop)
        .unwrap();
    session
        .feed_with_pty_responses(b"line1\r\nline2\r\nline3\r\n", &mut noop)
        .unwrap();
    session
        .feed_with_pty_responses(b"\x1b]133;D;0\x07", &mut noop)
        .unwrap();

    let output = session.take_last_command_output().unwrap();
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines, ["line1", "line2", "line3"]);
}

#[test]
fn osc133_captures_output_with_st_terminator() {
    let mut session = TerminalSession::new(TerminalConfig::default()).unwrap();

    let mut noop = |_: &[u8]| {};
    session.feed_with_pty_responses(b"\r\n", &mut noop).unwrap();
    // Use ESC \ (ST) instead of BEL.
    session
        .feed_with_pty_responses(b"\x1b]133;C\x1b\\", &mut noop)
        .unwrap();
    session
        .feed_with_pty_responses(b"result\r\n", &mut noop)
        .unwrap();
    session
        .feed_with_pty_responses(b"\x1b]133;D;0\x1b\\", &mut noop)
        .unwrap();

    assert_eq!(
        session.take_last_command_output().as_deref(),
        Some("result")
    );
}

#[test]
fn osc133_returns_none_when_no_output() {
    let mut session = TerminalSession::new(TerminalConfig::default()).unwrap();

    let mut noop = |_: &[u8]| {};
    session.feed_with_pty_responses(b"\r\n", &mut noop).unwrap();
    session
        .feed_with_pty_responses(b"\x1b]133;C\x07", &mut noop)
        .unwrap();
    // No output — cursor stays at same row, D fires immediately.
    session
        .feed_with_pty_responses(b"\x1b]133;D;0\x07", &mut noop)
        .unwrap();

    assert!(session.take_last_command_output().is_none());
}
