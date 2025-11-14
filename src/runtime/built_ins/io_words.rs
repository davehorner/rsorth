
use std::io::{self, Read, Write};
impl Read for RawIpcStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            #[cfg(unix)]
            RawIpcStream::Unix(s) => s.read(buf),
            #[cfg(windows)]
            RawIpcStream::NamedPipe(s) => s.read(buf),
            RawIpcStream::Tcp(s) => s.read(buf),
        }
    }
}

impl Write for RawIpcStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            #[cfg(unix)]
            RawIpcStream::Unix(s) => s.write(buf),
            #[cfg(windows)]
            RawIpcStream::NamedPipe(s) => s.write(buf),
            RawIpcStream::Tcp(s) => s.write(buf),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        match self {
            #[cfg(unix)]
            RawIpcStream::Unix(s) => s.flush(),
            #[cfg(windows)]
            RawIpcStream::NamedPipe(s) => s.flush(),
            RawIpcStream::Tcp(s) => s.flush(),
        }
    }
}

use std::{ collections::HashMap,
         fs::{ remove_file,
             File,
             OpenOptions },
         io::{ BufRead, BufReader, BufWriter, Seek },
         net::TcpStream,
         path::Path,
         sync::{ atomic::{ AtomicI64, Ordering },
             Mutex } };

#[cfg(unix)]
use std::os::unix::net::UnixStream;

#[cfg(unix)]
use std::os::unix::net::UnixStream;
#[cfg(windows)]
use named_pipe::PipeClient;
use lazy_static::lazy_static;
use crate::{ add_native_word,
             runtime::{ data_structures::value::ToValue,
                        error::{ self,
                                 script_error,
                                 script_error_str },
                        interpreter::Interpreter } };




pub enum RawIpcStream {
    #[cfg(unix)]
    Unix(UnixStream),
    #[cfg(windows)]
    NamedPipe(PipeClient),
    Tcp(TcpStream),
}

enum FileObject {
    File(File),
    Stream(RawIpcStream),
}


lazy_static!
{
    // The counter for generating new IDs.
    static ref FD_COUNTER: AtomicI64 = AtomicI64::new(4);

    // Keep a table to map generated FDs to file structs.
    static ref FILE_TABLE: Mutex<HashMap<i64, FileObject>> = Mutex::new(HashMap::new());
}

fn generate_fd() -> i64
{
    FD_COUNTER.fetch_add(1, Ordering::SeqCst)
}

fn add_file(fd: i64, file: File)
{
    FILE_TABLE.lock().unwrap().insert(fd, FileObject::File(file));
}


fn add_stream(fd: i64, stream: RawIpcStream) {
    FILE_TABLE.lock().unwrap().insert(fd, FileObject::Stream(stream));
}

fn get_file(interpreter: &mut dyn Interpreter, fd: i64) -> error::Result<FileObject>
{
    let table = FILE_TABLE.lock().unwrap();
    let file = table.get(&fd);

    match file
    {
        Some(file) =>
            {
                match file
                {
                    FileObject::File(file)     => Ok(FileObject::File(file.try_clone()?)),
                    // Cloning streams is not supported for all types; return an error for now
                    FileObject::Stream(_) => Err(std::io::Error::other("Cloning streams is not supported").into())
                }
            }

        None => script_error(interpreter, format!("File struct for fd {} not found.", fd))
    }
}

fn unregister_file(interpreter: &mut dyn Interpreter, fd: i64) -> error::Result<()>
{
    let mut table = FILE_TABLE.lock().unwrap();

    if !table.contains_key(&fd)
    {
        script_error(interpreter, format!("File struct not found for fd {}.", fd))?;
    }

    table.remove(&fd);

    Ok(())
}

fn flags_to_options(flags: i64) -> OpenOptions
{
    let mut options = OpenOptions::new();

    if flags & 0b0001 != 0
    {
        options.read(true);
    }

    if flags & 0b0010 != 0
    {
        options.write(true);
    }

    options
}



fn word_file_open(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let flags = interpreter.pop_as_int()?;
    let path = interpreter.pop_as_string()?;

    let options = flags_to_options(flags);

    match options.open(path.clone())
    {
        Ok(file) =>
            {
                let fd = generate_fd();

                add_file(fd, file);
                interpreter.push(fd.to_value());
            },

        Err(error) =>
            {
                script_error(interpreter, format!("Could not open file {}: {}", path, error))?;
            }
    }

    Ok(())
}

fn word_file_create(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let flags = interpreter.pop_as_int()?;
    let path = interpreter.pop_as_string()?;

    let mut options = flags_to_options(flags);

    options.create(true);
    options.truncate(true);

    match options.open(path.clone())
    {
        Ok(file) =>
            {
                let fd = generate_fd();

                add_file(fd, file);
                interpreter.push(fd.to_value());
            },

        Err(error) =>
            {
                script_error(interpreter, format!("Could not open file {}: {}", path, error))?;
            }
    }

    Ok(())
}

fn word_file_create_temp_file(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    script_error_str(interpreter, "Create temp file unimplemented.")
}

fn word_file_close(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let fd = interpreter.pop_as_int()?;

    unregister_file(interpreter, fd)?;

    Ok(())
}

fn word_file_delete(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let path = interpreter.pop_as_string()?;

    remove_file(&path)?;

    Ok(())
}


fn word_socket_connect(interpreter: &mut dyn Interpreter) -> error::Result<()> {
    let path = interpreter.pop_as_string()?;

    #[cfg(unix)]
    {
        // Try Unix domain socket first
        match UnixStream::connect(&path) {
            Ok(stream) => {
                let fd = generate_fd();
                add_stream(fd, RawIpcStream::Unix(stream));
                interpreter.push(fd.to_value());
                return Ok(());
            },
            Err(_) => {
                // Fallback to TCP
            }
        }
    }

    // Try TCP on all platforms
    if let Ok(stream) = TcpStream::connect(&path) {
        let fd = generate_fd();
        add_stream(fd, RawIpcStream::Tcp(stream));
        interpreter.push(fd.to_value());
        return Ok(());
    }

    #[cfg(windows)]
    {
        // Try named pipe
        if let Ok(pipe) = PipeClient::connect(&path) {
            let fd = generate_fd();
            add_stream(fd, RawIpcStream::NamedPipe(pipe));
            interpreter.push(fd.to_value());
            return Ok(());
        }
    }

    script_error(interpreter, format!("Failed to connect to any supported socket/pipe: {}", path))?
}

fn word_file_size_read(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let fd = interpreter.pop_as_int()?;
    let file = get_file(interpreter, fd)?;

    match file
    {
        FileObject::File(file) =>
            {
                let metadata = file.metadata()?;
                let size = metadata.len();

                interpreter.push(size.to_value());
            },

        FileObject::Stream(_) =>
            {
                script_error_str(interpreter, "Can not read size of a socket.")?;
            }
    }

    Ok(())
}

fn word_file_exists(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let path = interpreter.pop_as_string()?;

    interpreter.push(Path::new(&path).exists().to_value());
    Ok(())
}

fn word_file_is_open(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let fd = interpreter.pop_as_int()?;
    let file = get_file(interpreter, fd);

    interpreter.push(file.is_ok().to_value());

    Ok(())
}

fn word_file_is_eof(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    let fd = interpreter.pop_as_int()?;
    let file = get_file(interpreter, fd)?;

    match file
    {
        FileObject::File(mut file) =>
            {
                let current_pos = file.stream_position()?;
                let total_size = file.metadata()?.len();

                interpreter.push((current_pos == total_size).to_value());
            },

        FileObject::Stream(_) =>
            {
                script_error_str(interpreter, "Can not eof status of a socket.")?;
            }
    }

    Ok(())
}

fn word_file_read(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    script_error_str(interpreter, "Unimplemented.")
}

fn word_file_read_character(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    fn read<T>(interpreter: &mut dyn Interpreter, reader: &mut BufReader<T>) -> error::Result<()>
        where T: Read
    {
        let mut buffer = [0; 1];

        match reader.read(&mut buffer)
        {
            Ok(0) =>
                {
                    interpreter.push("".to_string().to_value());
                },

            Ok(_) =>
                {
                    interpreter.push(buffer[0].to_string().to_value());
                },

            Err(error) =>
                {
                    return script_error(interpreter,
                                        format!("Could not read from file: {}.", error))
                }
        }

        Ok(())
    }

    let fd = interpreter.pop_as_int()?;
    let file = get_file(interpreter, fd)?;

    match file
    {
        FileObject::File(file)     => read(interpreter, &mut BufReader::new(file)),
        FileObject::Stream(stream) => read(interpreter, &mut BufReader::new(stream)),
    }
}

fn word_file_read_string(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    fn read<T>(interpreter: &mut dyn Interpreter,
               reader: &mut BufReader<T>) -> error::Result<()>
        where T: Read
    {
        let mut string = String::new();

        match reader.read_to_string(&mut string)
        {
            Ok(0) =>
                {
                    interpreter.push("".to_string().to_value());
                },

            Ok(_) =>
                {
                    interpreter.push(string.to_value());
                },

            Err(error) =>
                {
                    return script_error(interpreter,
                                        format!("Could not read from file: {}.", error))
                }
        }

        Ok(())
    }

    let fd = interpreter.pop_as_int()?;
    let file = get_file(interpreter, fd)?;

    match file
    {
        FileObject::File(file)     => read(interpreter, &mut BufReader::new(file)),
        FileObject::Stream(stream) => read(interpreter, &mut BufReader::new(stream)),
    }
}

fn word_file_write(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    fn write<T>(interpreter: &mut dyn Interpreter,
               string: String,
               writer: &mut BufWriter<T>) -> error::Result<()>
        where T: Write
    {
        let bytes = string.into_bytes();

        match writer.write_all(bytes.as_slice())
        {
            // TODO: Handle partial writes.
            Ok(_) =>
                {
                    Ok(())
                },

            Err(error) =>
                {
                    script_error(interpreter, format!("Could not read from file: {}.", error))
                }
        }
    }

    // TODO: Implement ByteBuffer and better string conversion.
    let fd = interpreter.pop_as_int()?;
    let string = interpreter.pop_as_string()?;
    let file = get_file(interpreter, fd)?;

    match file
    {
        FileObject::File(file)     => write(interpreter, string, &mut BufWriter::new(file)),
        FileObject::Stream(stream) => write(interpreter, string, &mut BufWriter::new(stream)),
    }
}

fn word_file_line_read(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    fn read<T>(interpreter: &mut dyn Interpreter, buffer: &mut BufReader<T>) -> error::Result<()>
        where T: Read
    {
        let mut line = String::new();

        match buffer.read_line(&mut line)
        {
            Ok(0) =>
                {
                    interpreter.push("".to_string().to_value());
                },

            Ok(_) =>
                {
                    let line = line.trim_end_matches(&['\n', '\r'][..]).to_string();
                    interpreter.push(line.to_value());
                },

            Err(error) =>
                {
                    return script_error(interpreter,
                                        format!("Could not read from file: {}.", error))
                }
        }

        Ok(())
    }

    let fd = interpreter.pop_as_int()?;
    let file = get_file(interpreter, fd)?;

    match file
    {
        FileObject::File(file)     => read(interpreter, &mut BufReader::new(file)),
        FileObject::Stream(stream) => read(interpreter, &mut BufReader::new(stream)),
    }
}

fn word_file_line_write(interpreter: &mut dyn Interpreter) -> error::Result<()>
{
    fn write<T>(interpreter: &mut dyn Interpreter,
                string: String,
                writer: &mut BufWriter<T>) -> error::Result<()>
        where T: Write
    {
        let bytes = (string + "\n").into_bytes();

        match writer.write_all(bytes.as_slice())
        {
            // TODO: Handle partial writes.
            Ok(_) =>
                {
                    Ok(())
                },

            Err(error) =>
                {
                    script_error(interpreter, format!("Could not read from file: {}.", error))
                }
        }
    }

    // TODO: Implement better string conversion.
    let fd = interpreter.pop_as_int()?;
    let string = interpreter.pop_as_string()?;
    let file = get_file(interpreter, fd)?;

    match file
    {
        FileObject::File(file)     => write(interpreter, string, &mut BufWriter::new(file)),
        FileObject::Stream(stream) => write(interpreter, string, &mut BufWriter::new(stream)),
    }
}



pub fn register_io_words(interpreter: &mut dyn Interpreter)
{
    add_native_word!(interpreter, "file.open", word_file_open,
        "Open an existing file and return a fd.",
        "path flags -- fd");

    add_native_word!(interpreter, "file.create", word_file_create,
        "Create/open a file and return a fd.",
        "path flags -- fd");

    add_native_word!(interpreter, "file.create.tempfile", word_file_create_temp_file,
        "Create/open an unique temporary file and return it's fd.",
        "flags -- path fd");

    add_native_word!(interpreter, "file.close", word_file_close,
        "Take a fd and close it.",
        "fd -- ");

    add_native_word!(interpreter, "file.delete", word_file_delete,
        "Delete the specified file.",
        "file_path -- ");


    add_native_word!(interpreter, "socket.connect", word_socket_connect,
        "Connect to Unix domain socket at the given path.",
        "path -- fd");


    add_native_word!(interpreter, "file.size@", word_file_size_read,
        "Return the size of a file represented by a fd.",
        "fd -- size");


    add_native_word!(interpreter, "file.exists?", word_file_exists,
        "Does the file at the given path exist?",
        "path -- bool");

    add_native_word!(interpreter, "file.is_open?", word_file_is_open,
        "Is the fd currently valid?",
        "fd -- bool");

    add_native_word!(interpreter, "file.is_eof?", word_file_is_eof,
        "Is the file pointer at the end of the file?",
        "fd -- bool");


    add_native_word!(interpreter, "file.@", word_file_read,
        "Read from a given file.  (Unimplemented.)",
        " -- ");

    add_native_word!(interpreter, "file.char@", word_file_read_character,
        "Read a character from a given file.",
        "fd -- character");

    add_native_word!(interpreter, "file.string@", word_file_read_string,
        "Read a file to a string.",
        "fd -- string");

    add_native_word!(interpreter, "file.!", word_file_write,
        "Write a value as text to a file, unless it's a ByteBuffer.",
        "value fd -- ");


    add_native_word!(interpreter, "file.line@", word_file_line_read,
        "Read a full line from a file.",
        "fd -- string");

    add_native_word!(interpreter, "file.line!", word_file_line_write,
        "Write a string as a line to the file.",
        "string fd -- ");


    add_native_word!(interpreter, "file.r/o",
        |interpreter|
        {
            interpreter.push(0b0001_i64.to_value());
            Ok(())
        },
        "Constant for opening a file as read only.",
        " -- flag");

    add_native_word!(interpreter, "file.w/o",
        |interpreter|
        {
            interpreter.push(0b0010_i64.to_value());
            Ok(())
        },
        "Constant for opening a file as write only.",
        " -- flag");

    add_native_word!(interpreter, "file.r/w",
        |interpreter|
        {
            interpreter.push(0b0011_i64.to_value());
            Ok(())
        },
        "Constant for opening a file for both reading and writing.",
        " -- flag");
}

#[cfg(test)]
mod tests {
        #[cfg(unix)]
        #[test]
        fn test_raw_ipcstream_unix() {
            use std::os::unix::net::{UnixListener, UnixStream};
            use std::path::PathBuf;
            use std::fs;
            let socket_path = PathBuf::from("/tmp/test_raw_ipcstream.sock");
            let _ = fs::remove_file(&socket_path); // Clean up before test
            let listener = UnixListener::bind(&socket_path).expect("bind failed");
            let path = socket_path.clone();
            let handle = std::thread::spawn(move || {
                let (mut stream, _) = listener.accept().expect("accept failed");
                let mut buf = [0u8; 4];
                stream.read_exact(&mut buf).expect("read failed");
                assert_eq!(&buf, b"ping");
                stream.write_all(b"pong").expect("write failed");
            });
            let mut stream = RawIpcStream::Unix(UnixStream::connect(&path).expect("connect failed"));
            stream.write_all(b"ping").expect("write failed");
            let mut buf = [0u8; 4];
            stream.read_exact(&mut buf).expect("read failed");
            assert_eq!(&buf, b"pong");
            handle.join().unwrap();
            let _ = fs::remove_file(&socket_path); // Clean up after test
        }

        #[cfg(windows)]
        #[test]
        fn test_raw_ipcstream_namedpipe() {
            use named_pipe::{PipeOptions, PipeClient};
            use std::thread;
            let pipe_name = r"\\.\pipe\test_raw_ipcstream";
            let server = thread::spawn(move || {
                let mut server = PipeOptions::new(pipe_name)
                    .single()
                    .expect("create pipe failed")
                    .wait()
                    .expect("wait failed");
                let mut buf = [0u8; 4];
                server.read_exact(&mut buf).expect("read failed");
                assert_eq!(&buf, b"ping");
                server.write_all(b"pong").expect("write failed");
            });
            // Give the server a moment to start
            std::thread::sleep(std::time::Duration::from_millis(100));
            let mut client = RawIpcStream::NamedPipe(PipeClient::connect(pipe_name).expect("connect failed"));
            client.write_all(b"ping").expect("write failed");
            let mut buf = [0u8; 4];
            client.read_exact(&mut buf).expect("read failed");
            assert_eq!(&buf, b"pong");
            server.join().unwrap();
        }
    use super::RawIpcStream;
    use std::net::{TcpListener, TcpStream};
    use std::thread;
    use std::io::{Read, Write};

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