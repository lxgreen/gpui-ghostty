use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use gpui::{App, AppContext, Application, KeyBinding, px};
use gpui_ghostty_terminal::view::{
    Copy, CopyLastOutput, Paste, SelectAll, TerminalInput, TerminalView,
};
use gpui_ghostty_terminal::{
    TerminalConfig, TerminalSession, load_config, terminal_font, window_options_for_config,
};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("cmd-a", SelectAll, None),
            KeyBinding::new("cmd-c", Copy, None),
            KeyBinding::new("cmd-shift-c", CopyLastOutput, None),
            KeyBinding::new("cmd-v", Paste, None),
        ]);

        // Load config before opening window so we can set background appearance
        let config = load_config().unwrap_or_else(|_| TerminalConfig::default());
        let options = window_options_for_config(&config);

        cx.open_window(options, |window, cx| {
            let pty_system = native_pty_system();
            let pty_pair = pty_system
                .openpty(PtySize {
                    rows: config.rows,
                    cols: config.cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .expect("openpty failed");

            let master = pty_pair.master;

            // Use command from config, or fall back to $SHELL, or /bin/zsh
            let shell_cmd = config.command.clone().unwrap_or_else(|| {
                std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string())
            });

            // Parse command - if it contains spaces, treat as command + args
            let mut parts = shell_cmd.split_whitespace();
            let shell = parts.next().unwrap_or("/bin/zsh");
            let mut cmd = CommandBuilder::new(shell);

            // Add remaining parts as arguments
            for arg in parts {
                cmd.arg(arg);
            }

            // If no args were in the command string, add login flag for shells
            if shell_cmd.split_whitespace().count() == 1 {
                cmd.arg("-l");
            }

            cmd.env("TERM", "xterm-256color");
            cmd.env("COLORTERM", "truecolor");
            cmd.env("TERM_PROGRAM", "gpui-ghostty");

            // Enable Ghostty shell integration so fish emits OSC 133 markers.
            // Fish auto-sources vendor_conf.d/*.fish files from XDG_DATA_DIRS;
            // prepending the vendored integration directory activates it.
            const GHOSTTY_INTEGRATION_DIR: &str = concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../vendor/ghostty/src/shell-integration"
            );
            cmd.env("GHOSTTY_SHELL_INTEGRATION_XDG_DIR", GHOSTTY_INTEGRATION_DIR);
            cmd.env("GHOSTTY_SHELL_FEATURES", "no-cursor,no-sudo");
            let xdg = match std::env::var("XDG_DATA_DIRS") {
                Ok(existing) if !existing.is_empty() => {
                    format!("{}:{}", GHOSTTY_INTEGRATION_DIR, existing)
                }
                _ => GHOSTTY_INTEGRATION_DIR.to_string(),
            };
            cmd.env("XDG_DATA_DIRS", xdg);

            let mut child = pty_pair
                .slave
                .spawn_command(cmd)
                .expect("spawn shell failed");

            thread::spawn(move || {
                let _ = child.wait();
            });

            let mut pty_reader = master.try_clone_reader().expect("pty reader");
            let mut pty_writer = master.take_writer().expect("pty writer");

            let (stdin_tx, stdin_rx) = mpsc::channel::<Vec<u8>>();
            let (stdout_tx, stdout_rx) = mpsc::channel::<Vec<u8>>();
            let (resize_tx, resize_rx) = mpsc::channel::<(u16, u16)>();

            if let Ok(cmd) = std::env::var("GPUI_GHOSTTY_PTY_DEMO_COMMAND") {
                let stdin_tx = stdin_tx.clone();
                thread::spawn(move || {
                    thread::sleep(Duration::from_millis(300));
                    let mut cmd = cmd;
                    if !cmd.ends_with('\n') {
                        cmd.push('\n');
                    }
                    let _ = stdin_tx.send(cmd.into_bytes());
                });
            }

            thread::spawn(move || {
                while let Ok(bytes) = stdin_rx.recv() {
                    if pty_writer.write_all(&bytes).is_err() {
                        break;
                    }
                    let _ = pty_writer.flush();
                }
            });

            thread::spawn(move || {
                let mut buf = [0u8; 8192];
                loop {
                    let n = match pty_reader.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => n,
                        Err(_) => break,
                    };
                    let _ = stdout_tx.send(buf[..n].to_vec());
                }
            });

            // Handle PTY resize events in a separate thread
            // Move the master into this thread since it owns the resize capability
            thread::spawn(move || {
                while let Ok((cols, rows)) = resize_rx.recv() {
                    let _ = master.resize(PtySize {
                        rows,
                        cols,
                        pixel_width: 0,
                        pixel_height: 0,
                    });
                }
            });

            // Clone config for use in view setup
            let config_for_view = config.clone();

            // Set up PTY resize callback
            let resize_callback =
                gpui_ghostty_terminal::view::TerminalResizeCallback::new(move |cols, rows| {
                    let _ = resize_tx.send((cols, rows));
                });

            let view = cx.new(|cx| {
                let focus_handle = cx.focus_handle();
                focus_handle.focus(window, cx);

                let session = TerminalSession::new(config).expect("vt init");
                let stdin_tx = stdin_tx.clone();
                let input = TerminalInput::new(move |bytes| {
                    let _ = stdin_tx.send(bytes.to_vec());
                });

                let mut view = TerminalView::new_with_input(session, focus_handle, input);
                // Apply font settings from config
                view.set_font(terminal_font(&config_for_view));
                if let Some(size) = config_for_view.font_size {
                    view.set_font_size(px(size));
                }
                // Set resize callback to notify PTY of size changes
                view.set_resize_callback(resize_callback);
                view
            });

            let view_for_task = view.clone();
            window
                .spawn(cx, async move |cx| {
                    loop {
                        cx.background_executor()
                            .timer(Duration::from_millis(16))
                            .await;
                        let mut batch = Vec::new();
                        while let Ok(chunk) = stdout_rx.try_recv() {
                            batch.extend_from_slice(&chunk);
                        }
                        if batch.is_empty() {
                            continue;
                        }

                        cx.update(|_, cx| {
                            view_for_task.update(cx, |this, cx| {
                                this.queue_output_bytes(&batch, cx);
                            });
                        })
                        .ok();
                    }
                })
                .detach();

            view
        })
        .unwrap();
    });
}
