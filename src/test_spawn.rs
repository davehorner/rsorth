use std::process::{Command, Child};
use std::thread;
use std::time::Duration;

fn spawn_forth_side(script: &str) -> Child {
    Command::new("target/debug/sorth.exe")
        .arg(script)
        .spawn()
        .expect("Failed to spawn process")
}

fn main() {
    // Start the publisher (replace with your actual publisher script)
    let mut pub_proc = spawn_forth_side("std/publisher.f");
    // Give the publisher a moment to start
    thread::sleep(Duration::from_millis(500));
    // Start the subscriber (replace with your actual subscriber script)
    let mut sub_proc = spawn_forth_side("std/subscriber.f");

    // Wait for both to finish
    let pub_status = pub_proc.wait().expect("Publisher process failed");
    let sub_status = sub_proc.wait().expect("Subscriber process failed");

    println!("Publisher exited with: {:?}", pub_status);
    println!("Subscriber exited with: {:?}", sub_status);
}
