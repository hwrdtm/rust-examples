use std::io::{Read, Write};
use std::os::fd::{FromRawFd, IntoRawFd};
use std::process::{Command, Stdio};

fn main() {
    let mut file = std::fs::File::create("./node.txt").unwrap();

    // Use Command and Stdio piped to run: echo -n 'thing' | base64
    let echo_cmd = Command::new("echo")
        .arg("-n")
        .arg("thing")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let encode_cmd = Command::new("base64")
        .stdin(Stdio::from(echo_cmd.stdout.unwrap()))
        // Write the result of encode_cmd to file.
        .stdout(Stdio::from(file))
        .spawn()
        .unwrap();
}
