#![allow(non_snake_case)]

use dioxus::prelude::*;
use nix::pty::forkpty;
use nix::unistd::{read, ForkResult};
use regex::Regex;
use std::fs::File;
use std::io::Write;
use std::os::fd::IntoRawFd;
use std::os::unix::io::{FromRawFd, RawFd};
use std::process::Command;

// read from file descriptor
fn read_from_fd(fd: RawFd) -> Option<Vec<u8>> {
    let mut read_buffer = [0; 65536];
    let read_result = read(fd, &mut read_buffer);
    // TODO: look into if let for Results
    match read_result {
        Ok(bytes_read) => Some(read_buffer[..bytes_read].to_vec()),
        Err(_e) => None,
    }
}

fn remove_ansi_escape_codes(input: &str) -> String {
    let ansi_escape_code_regex = Regex::new(r"\x1B\[([0-9]{1,2}(;[0-9]{1,2})?)?[m|K]").unwrap();

    ansi_escape_code_regex
        .replace_all(input, "")
        .to_string()
        .replace("bash-3.2$", "")
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

fn user_command(stdout_fd: i32, input: &str) -> String {
    let mut output_file: File = unsafe { File::from_raw_fd(stdout_fd) };

    if let Err(e) = write!(output_file, "ls\n") {
        panic!("There was an error writing the output: {:?}", e)
    }
    match output_file.flush() {
        Ok(_) => (),
        Err(_) => panic!("There was an error flushing the output!"),
    }
    let read_buffer: Vec<u8> = vec![];
    // TODO: append the results from the response to an array and then change the UI to loop
    // through the array and render the data
    loop {
        match read_from_fd(stdout_fd) {
            Some(read_bytes) => {
                let std_out: String = String::from_utf8(read_bytes).unwrap();
                let bash_response = remove_ansi_escape_codes(&std_out);
                if !bash_response.contains(input) {
                    return bash_response;
                }
            }
            None => {
                println!("{:?}", String::from_utf8(read_buffer).unwrap());
                std::process::exit(0)
            }
        }
    }
}

fn App(cx: Scope) -> Element {
    let default_shell = String::from("bash");
    // let default_shell = std::env::var("SHELL").expect("Unable to load shell");
    let stdout_fd: i32 = spawn_pty_with_shell(default_shell);

    let user_input = use_state(cx, || "".to_string());
    let command = use_state(cx, || "".to_string());

    let handle_input_submit = move |event: KeyboardEvent| {
        if event.key().to_string() == "Enter" {
            let response: String = user_command(stdout_fd, user_input);
            command.set(response);
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
                    // "Why don't you just Google the answer?"
                    command
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
