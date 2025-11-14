// Integration test for RawIpcStream abstraction
// This test will create a TCP listener and connect to it using the word_socket_connect logic.
// It verifies that the abstraction works for TCP (cross-platform).

#[cfg(test)]
mod tests {
    use std::net::{TcpListener, TcpStream};
    use std::thread;
    use std::io::{Read, Write};
    use super::RawIpcStream;

    #[test]
    fn test_raw_ipcstream_tcp() {
        // Start a TCP listener in a background thread
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind failed");
        let addr = listener.local_addr().unwrap();

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept failed");
            let mut buf = [0u8; 4];
            stream.read_exact(&mut buf).expect("read failed");
            assert_eq!(&buf, b"ping");
            stream.write_all(b"pong").expect("write failed");
        });

        // Connect as client using RawIpcStream::Tcp
        let mut stream = RawIpcStream::Tcp(TcpStream::connect(addr).expect("connect failed"));
        stream.write_all(b"ping").expect("write failed");
        let mut buf = [0u8; 4];
        stream.read_exact(&mut buf).expect("read failed");
        assert_eq!(&buf, b"pong");

        handle.join().unwrap();
    }
}
