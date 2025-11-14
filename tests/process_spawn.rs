// Integration test for process.spawn in rsorth
// This test will use the interpreter to run publisher.f and subscriber.f using process.spawn

use std::process::Command;

#[test]
fn test_process_spawn_in_rsorth() {
    // Path to the rsorth interpreter
    let exe = if cfg!(windows) {
        "target/debug/sorth.exe"
    } else {
        "target/debug/sorth"
    };

    use std::thread;
    use std::sync::mpsc;

    // Channel to collect outputs
    let (tx, rx) = mpsc::channel();

    // Start subscriber in a thread
    let exe_sub = exe.to_string();
    let tx_sub = tx.clone();
    let sub_handle = thread::spawn(move || {
        let output = Command::new(&exe_sub)
            .arg("tests/iox_sub.f")
            .output()
            .expect("Failed to run iox_sub.f");
        tx_sub.send(("sub", output)).unwrap();
    });

    // Give the subscriber a moment to start
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Start publisher in a thread
    let exe_pub = exe.to_string();
    let tx_pub = tx.clone();
    let pub_handle = thread::spawn(move || {
        let output = Command::new(&exe_pub)
            .arg("tests/iox_pub.f")
            .output()
            .expect("Failed to run iox_pub.f");
        tx_pub.send(("pub", output)).unwrap();
    });

    // Wait for both to finish
    sub_handle.join().unwrap();
    pub_handle.join().unwrap();

    // Collect outputs
    let mut pub_ok = false;
    let mut sub_ok = false;
    for _ in 0..2 {
        let (role, output) = rx.recv().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("--- {} stdout ---\n{}", role, stdout);
        println!("--- {} stderr ---\n{}", role, stderr);
        assert!(output.status.success(), "{} failed: {:?}\nstdout:\n{}\nstderr:\n{}", role, output, stdout, stderr);
        if role == "pub" {
            pub_ok = stdout.contains("hello from forth");
        } else if role == "sub" {
            sub_ok = stdout.contains("received:");
        }
    }
    assert!(pub_ok, "iox_pub.f did not output expected message");
    assert!(sub_ok, "iox_sub.f did not output expected message");
}
