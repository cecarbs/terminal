#![allow(non_snake_case)]

use dioxus::prelude::*;
use nix::pty::forkpty;
use nix::unistd::{read, ForkResult};
use std::env;
use std::fs::File;
use std::os::fd::IntoRawFd;
use std::os::unix::io::{FromRawFd, RawFd};
use std::process::{Command, Stdio};

fn read_from_fd(fd: RawFd) -> Option<Vec<u8>> {
    let mut read_buffer = [0; 65536];
    let read_result = read(fd, &mut read_buffer);
    match read_result {
        Ok(bytes_read) => Some(read_buffer[..bytes_read].to_vec()),
        Err(_e) => None,
    }
}
fn spawn_pty_with_shell(default_shell: String) -> RawFd {
    match unsafe { forkpty(None, None) } {
        Ok(fork_pty_res) => {
            let stdout_fd = fork_pty_res.master;
            if let ForkResult::Child = fork_pty_res.fork_result {
                Command::new(default_shell)
                    .spawn()
                    .expect("failed to spawn");
                std::thread::sleep(std::time::Duration::from_millis(2000));
                std::process::exit(0);
            }
            stdout_fd.into_raw_fd()
        }
        Err(e) => {
            panic!("failed to fork {:?}", e);
        }
    }
}

fn App(cx: Scope) -> Element {
    let user_input = use_state(cx, || "".to_string());
    let command = use_state(cx, || "".to_string());

    let handle_input_submit = move |event: KeyboardEvent| {
        if event.key().to_string() == "Enter" {
            println!("Enter was pressed");
            command.set(user_input.to_string());
            user_input.set("".to_string());
        };
    };

    render! {
        div {
            background_color: "#000",
            color: "#0f0",
            height: "100vh",
            display: "flex",
            flex_direction: "column",
            font_family: "monospace",
            width: "100%",

            div {
                flex_grow: "1",
                overflow_y: "auto",
                padding: "10px",
                p {
                    if command.chars().count() > 0 {
                    "Why don't you just Google the answer?"
                    } else {
                        ""
                    }
                }
            }

            div {
                display: "flex",
                padding: "10px",
                width: "100%",
                padding_right: "16px",

                span {
                    color: "#00ff00",
                    margin_right: "5px",
                    font_size: "16px",
                    "$"
                }

                input {

                    background_color: "transparent",
                    flex: 1,
                    width: "100%",
                    border: "none",
                    outline: "none",
                    color: "#0f0",
                    font_family: "monospace",
                    font_size: "16px",
                    oninput: move |evt| user_input.set(evt.value.clone()),
                    onkeypress: handle_input_submit,
                    value: "{user_input}",

                }
            }
        }
    }
}

fn main() {
    dioxus_desktop::launch_cfg(
        App,
        dioxus_desktop::Config::new()
            .with_custom_head(r#"<link rel="stylesheet" href="tailwind.css">"#.to_string()),
    )
}