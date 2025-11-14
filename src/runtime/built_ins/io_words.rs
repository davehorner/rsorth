use crate::runtime::data_structures::value::ToValue;
#[cfg(feature = "uses_iceoryx2")]
use iceoryx2_bb_log::{set_log_level_from_env_or, LogLevel};
use crate::runtime::{
    error::{self, script_error, script_error_str, ScriptError},
    interpreter::Interpreter,
};
use std::collections::HashMap;
use std::cell::RefCell;


#[cfg(feature = "uses_iceoryx2")]
pub struct IoxSub {
    pub subscriber: iceoryx2::port::subscriber::Subscriber<IoxIpcService, [u8; 4096], ()>,
}

#[cfg(feature = "uses_iceoryx2")]
pub struct Iceoryx2ByteStream {
    pub subscriber: iceoryx2::port::subscriber::Subscriber<IoxIpcService, [u8; 4096], ()>,
    pub publisher: iceoryx2::port::publisher::Publisher<IoxIpcService, [u8; 4096], ()>,
    pub read_buf: [u8; 4096],
    pub read_pos: usize,
    pub read_len: usize,
}

#[cfg(feature = "uses_iceoryx2")]
fn word_iox_pub(interpreter: &mut dyn Interpreter) -> error::Result<()> {
    use std::collections::HashMap;
    use std::cell::RefCell;
    use iceoryx2::prelude::*;
    set_log_level_from_env_or(LogLevel::Debug);
    println!("iox.pub: called");
    thread_local! {
        static NODES: RefCell<HashMap<String, Node<ipc::Service>>> = RefCell::new(HashMap::new());
        static PUBS: RefCell<HashMap<String, Publisher<ipc::Service, [u8], ()>>> = RefCell::new(HashMap::new());
    }
    let spec = interpreter.pop_as_string()?;
    println!("iox.pub: spec = {}", spec);
    let parts: Vec<&str> = spec.split('/').collect();
    if parts.len() != 3 {
        return script_error_str(interpreter, "iox.pub expects 'Service/Instance/Event' string");
    }
    let key = spec.clone();
    println!("iox.pub: key = {}", key);
    if let Err(e) = NODES.with(|nodes: &RefCell<HashMap<String, Node<ipc::Service>>>| {
        if !nodes.borrow().contains_key(&key) {
            println!("iox.pub: creating node for key = {}", key);
            let node = match NodeBuilder::new().create::<ipc::Service>() {
                Ok(n) => n,
                Err(e) => return Err(script_error_str(interpreter, &format!("iox.pub node: {e}"))),
            };
            nodes.borrow_mut().insert(key.clone(), node);
        }
        Ok(())
    }) {
        return e;
    }
    if let Err(e) = PUBS.with(|pubs: &RefCell<HashMap<String, Publisher<ipc::Service, [u8], ()>>>| {
        if pubs.borrow().contains_key(&key) {
            println!("iox.pub: publisher already exists for key = {}", key);
            return Ok(());
        }
        let res = NODES.with(|nodes: &RefCell<HashMap<String, Node<ipc::Service>>>| {
            let binding = nodes.borrow();
            let node = binding.get(&key).unwrap();
            println!("iox.pub: creating publisher for key = {}", key);
            let service = node
                .service_builder(&spec.as_str().try_into().unwrap())
                .publish_subscribe::<[u8]>()
                .open_or_create();
            match service {
                Ok(service) => {
                    match service.publisher_builder().create() {
                        Ok(publisher) => {
                            println!("iox.pub: publisher created for key = {}", key);
                            pubs.borrow_mut().insert(key.clone(), publisher);
                            Ok(())
                        },
                        Err(e) => Err(script_error_str(interpreter, &format!("iox.pub publisher: {e}")))
                    }
                },
                Err(e) => Err(script_error_str(interpreter, &format!("iox.pub service: {e}")))
            }
        });
        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }) {
        return e;
    }
    println!("iox.pub: completed for key = {}", key);
    Ok(())
}

#[cfg(feature = "uses_iceoryx2")]
fn word_iox_sub(interpreter: &mut dyn Interpreter) -> error::Result<()> {
    use std::collections::HashMap;
    use std::cell::RefCell;
    use iceoryx2::prelude::*;
    set_log_level_from_env_or(LogLevel::Debug);
    println!("iox.sub: called");
    thread_local! {
        static NODES: RefCell<HashMap<String, Node<ipc::Service>>> = RefCell::new(HashMap::new());
        static SUBS: RefCell<HashMap<String, Subscriber<ipc::Service, [u8], ()>>> = RefCell::new(HashMap::new());
    }
    let spec = interpreter.pop_as_string()?;
    println!("iox.sub: spec = {}", spec);
    let parts: Vec<&str> = spec.split('/').collect();
    if parts.len() != 3 {
        return script_error_str(interpreter, "iox.sub expects 'Service/Instance/Event' string");
    }
    let key = spec.clone();
    println!("iox.sub: key = {}", key);
    if let Err(e) = NODES.with(|nodes: &RefCell<HashMap<String, Node<ipc::Service>>>| {
        if !nodes.borrow().contains_key(&key) {
            println!("iox.sub: creating node for key = {}", key);
            let node = match NodeBuilder::new().create::<ipc::Service>() {
                Ok(n) => n,
                Err(e) => return Err(script_error_str(interpreter, &format!("iox.sub node: {e}"))),
            };
            nodes.borrow_mut().insert(key.clone(), node);
        }
        Ok(())
    }) {
        return e;
    }
    if let Err(e) = SUBS.with(|subs: &RefCell<HashMap<String, Subscriber<ipc::Service, [u8], ()>>>| {
        if subs.borrow().contains_key(&key) {
            println!("iox.sub: subscriber already exists for key = {}", key);
            return Ok(());
        }
        let res = NODES.with(|nodes: &RefCell<HashMap<String, Node<ipc::Service>>>| {
            let binding = nodes.borrow();
            let node = binding.get(&key).unwrap();
            println!("iox.sub: creating subscriber for key = {}", key);
            let service = node
                .service_builder(&spec.as_str().try_into().unwrap())
                .publish_subscribe::<[u8]>()
                .open_or_create();
            match service {
                Ok(service) => {
                    match service.subscriber_builder().create() {
                        Ok(subscriber) => {
                            println!("iox.sub: subscriber created for key = {}", key);
                            subs.borrow_mut().insert(key.clone(), subscriber);
                            Ok(())
                        },
                        Err(e) => Err(script_error_str(interpreter, &format!("iox.sub subscriber: {e}")))
                    }
                },
                Err(e) => Err(script_error_str(interpreter, &format!("iox.sub service: {e}")))
            }
        });
        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }) {
        return e;
    }
    println!("iox.sub: completed for key = {}", key);
    Ok(())
}

#[cfg(feature = "uses_iceoryx2")]
fn word_iox_pub_send(interpreter: &mut dyn Interpreter) -> error::Result<()> {
    use std::collections::HashMap;
    use std::cell::RefCell;
    use iceoryx2::prelude::*;
    set_log_level_from_env_or(LogLevel::Debug);
    println!("iox.pub!: called");
    thread_local! {
        static PUBS: RefCell<HashMap<String, Publisher<ipc::Service, [u8], ()>>> = RefCell::new(HashMap::new());
    }
    let spec = interpreter.pop_as_string()?;
    println!("iox.pub!: spec = {}", spec);
    let msg = interpreter.pop_as_string()?;
    println!("iox.pub!: msg = {}", msg);
    let mut sent = false;
    PUBS.with(|map: &RefCell<HashMap<String, Publisher<ipc::Service, [u8], ()>>>| {
        if let Some(publisher) = map.borrow().get(&spec) {
            let len = msg.as_bytes().len();
            if let Ok(sample) = publisher.loan_slice_uninit(len) {
                let sample = sample.write_from_fn(|i| msg.as_bytes()[i]);
                if sample.send().is_ok() {
                    sent = true;
                    println!("iox.pub!: sent message for spec = {}", spec);
                }
            }
        }
    });
    if !sent {
        println!("iox.pub!: failed to send for spec = {}", spec);
        return script_error_str(interpreter, "iox.pub! failed: publisher not found or publish failed");
    }
    println!("iox.pub!: completed for spec = {}", spec);
    Ok(())
}
use std::sync::Mutex;
use std::sync::atomic::AtomicI64;
use lazy_static::lazy_static;
// Stream abstraction for true bytestreams

#[cfg(unix)]
use std::os::unix::net::UnixStream;
#[cfg(windows)]
use named_pipe::PipeClient;
use std::net::TcpStream;

pub enum RawIpcStream {
    #[cfg(unix)]
    Unix(UnixStream),
    #[cfg(windows)]
    NamedPipe(PipeClient),
    Tcp(TcpStream),
}
use std::io::{self, Read, Write};
#[cfg(feature = "uses_iceoryx2")]
use iceoryx2::prelude::*;
#[cfg(feature = "uses_iceoryx2")]
use iceoryx2::port::publisher::Publisher;
#[cfg(feature = "uses_iceoryx2")]
use iceoryx2::port::subscriber::Subscriber;
#[cfg(feature = "uses_iceoryx2")]
use iceoryx2::service::ipc::Service as IoxIpcService;


use std::{
    fs::{File, OpenOptions, remove_file},
    io::{BufRead, BufReader, BufWriter, Seek},
    path::Path,
    sync::{
        atomic::Ordering,
    },
};




#[cfg(feature = "uses_iceoryx2")]
fn word_iox_sub_recv(interpreter: &mut dyn Interpreter) -> error::Result<()> {
    use std::collections::HashMap;
    use std::cell::RefCell;
    use iceoryx2::prelude::*;
    set_log_level_from_env_or(LogLevel::Debug);
    println!("iox.sub@: called");
    thread_local! {
        static SUBS: RefCell<HashMap<String, Subscriber<ipc::Service, [u8], ()>>> = RefCell::new(HashMap::new());
    }
    // Pop the spec string (Service/Instance/Event) from the stack to use as the key
    let spec = interpreter.pop_as_string()?;
    println!("iox.sub@: spec = {}", spec);
    let mut found = false;
    let mut result = Ok(());
    SUBS.with(|subs: &RefCell<HashMap<String, Subscriber<ipc::Service, [u8], ()>>>| {
        if let Some(subscriber) = subs.borrow_mut().get_mut(&spec) {
            println!("iox.sub@: receiving for spec = {}", spec);
            match subscriber.receive() {
                Ok(Some(sample)) => {
                    let payload = sample.payload();
                    let s = String::from_utf8_lossy(&payload[..]).trim_end_matches(char::from(0)).to_string();
                    use crate::runtime::data_structures::value::ToValue;
                    interpreter.push(s.to_value());
                    found = true;
                    println!("iox.sub@: received message for spec = {}: {}", spec, s);
                }
                Ok(None) => {
                    use crate::runtime::data_structures::value::ToValue;
                    interpreter.push("".to_string().to_value());
                    found = true;
                    println!("iox.sub@: no message available for spec = {}", spec);
                }
                Err(e) => {
                    result = Err(io::Error::new(io::ErrorKind::Other, format!("iceoryx2 recv error: {e}")).into());
                    found = true;
                    println!("iox.sub@: error receiving for spec = {}: {}", spec, e);
                }
            }
        }
    });
    if !found {
        println!("iox.sub@: subscriber not found for spec = {}", spec);
        return script_error_str(interpreter, "iox.sub@: subscriber not found");
    }
    println!("iox.sub@: completed for spec = {}", spec);
    result
}

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

#[cfg(feature = "uses_iceoryx2")]
impl Iceoryx2ByteStream {
    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.read_pos >= self.read_len {
            match self.subscriber.receive() {
                Ok(Some(sample)) => {
                    let payload: &[u8; 4096] = sample.payload();
                    self.read_buf.copy_from_slice(payload);
                    self.read_len = 4096;
                    self.read_pos = 0;
                }
                Ok(None) => return Ok(0),
                Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("iceoryx2 receive error: {e}"))),
            }
        }
        let available = &self.read_buf[self.read_pos..self.read_len];
        let n = available.len().min(buf.len());
        buf[..n].copy_from_slice(&available[..n]);
        self.read_pos += n;
        Ok(n)
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

#[cfg(feature = "uses_iceoryx2")]
impl Iceoryx2ByteStream {
    pub fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.len() > 4096 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "iceoryx2 bytestream max 4096 bytes per message"));
        }
        let mut arr = [0u8; 4096];
        arr[..buf.len()].copy_from_slice(buf);
        self.publisher.send_copy(arr).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("iceoryx2 send error: {e}")))?;
        Ok(buf.len())
    }
    pub fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

enum FileObject {
    File(File),
    Stream(RawIpcStream), // Never contains Iceoryx2 variant
}


lazy_static! {
    // The counter for generating new IDs.
    static ref FD_COUNTER: AtomicI64 = AtomicI64::new(4);
    // Keep a table to map generated FDs to file structs (excluding iceoryx2 streams).
    static ref FILE_TABLE: Mutex<HashMap<i64, FileObject>> = Mutex::new(HashMap::new());
}

#[cfg(feature = "uses_iceoryx2")]
thread_local! {
    static ICEORYX2_STREAM_TABLE: RefCell<HashMap<i64, Iceoryx2ByteStream>> = RefCell::new(HashMap::new());
}

fn generate_fd() -> i64 {
    FD_COUNTER.fetch_add(1, Ordering::SeqCst)
}

fn add_file(fd: i64, file: File) {
    FILE_TABLE
        .lock()
        .unwrap()
        .insert(fd, FileObject::File(file));
}

fn add_stream(fd: i64, stream: RawIpcStream) {
    FILE_TABLE
        .lock()
        .unwrap()
        .insert(fd, FileObject::Stream(stream));
}

#[cfg(feature = "uses_iceoryx2")]
fn add_iceoryx2_stream(fd: i64, stream: Iceoryx2ByteStream) {
    ICEORYX2_STREAM_TABLE.with(|table| {
        table.borrow_mut().insert(fd, stream);
    });
}

fn get_file(interpreter: &mut dyn Interpreter, fd: i64) -> error::Result<FileObject> {
    #[cfg(feature = "uses_iceoryx2")]
    {
        if ICEORYX2_STREAM_TABLE.with(|table| table.borrow().contains_key(&fd)) {
            // Cloning not supported for iceoryx2 streams
            return Err(std::io::Error::other("Cloning iceoryx2 streams is not supported").into());
        }
    }
    let table = FILE_TABLE.lock().unwrap();
    let file = table.get(&fd);

    match file {
        Some(file) => {
            match file {
                FileObject::File(file) => Ok(FileObject::File(file.try_clone()?)),
                // Cloning streams is not supported for all types; return an error for now
                FileObject::Stream(_) => {
                    Err(std::io::Error::other("Cloning streams is not supported").into())
                }
            }
        }

        None => script_error(interpreter, format!("File struct for fd {} not found.", fd)),
    }
}

fn unregister_file(interpreter: &mut dyn Interpreter, fd: i64) -> error::Result<()> {
    #[cfg(feature = "uses_iceoryx2")]
    {
        let removed = ICEORYX2_STREAM_TABLE.with(|table| table.borrow_mut().remove(&fd));
        if removed.is_some() {
            return Ok(());
        }
    }
    let mut table = FILE_TABLE.lock().unwrap();

    if !table.contains_key(&fd) {
        script_error(interpreter, format!("File struct not found for fd {}.", fd))?;
    }

    table.remove(&fd);

    Ok(())
}

fn flags_to_options(flags: i64) -> OpenOptions {
    let mut options = OpenOptions::new();

    if flags & 0b0001 != 0 {
        options.read(true);
    }

    if flags & 0b0010 != 0 {
        options.write(true);
    }

    options
}

fn word_file_open(interpreter: &mut dyn Interpreter) -> error::Result<()> {
    let flags = interpreter.pop_as_int()?;
    let path = interpreter.pop_as_string()?;

    let options = flags_to_options(flags);

    match options.open(path.clone()) {
        Ok(file) => {
            let fd = generate_fd();

            add_file(fd, file);
            interpreter.push(fd.to_value());
        }

        Err(error) => {
            script_error(
                interpreter,
                format!("Could not open file {}: {}", path, error),
            )?;
        }
    }

    Ok(())
}

fn word_file_create(interpreter: &mut dyn Interpreter) -> error::Result<()> {
    let flags = interpreter.pop_as_int()?;
    let path = interpreter.pop_as_string()?;

    let mut options = flags_to_options(flags);

    options.create(true);
    options.truncate(true);

    match options.open(path.clone()) {
        Ok(file) => {
            let fd = generate_fd();

            add_file(fd, file);
            interpreter.push(fd.to_value());
        }

        Err(error) => {
            script_error(
                interpreter,
                format!("Could not open file {}: {}", path, error),
            )?;
        }
    }

    Ok(())
}

fn word_file_create_temp_file(interpreter: &mut dyn Interpreter) -> error::Result<()> {
    script_error_str(interpreter, "Create temp file unimplemented.")
}

fn word_file_close(interpreter: &mut dyn Interpreter) -> error::Result<()> {
    let fd = interpreter.pop_as_int()?;

    unregister_file(interpreter, fd)?;

    Ok(())
}

fn word_file_delete(interpreter: &mut dyn Interpreter) -> error::Result<()> {
    let path = interpreter.pop_as_string()?;

    remove_file(&path)?;

    Ok(())
}

fn word_socket_connect(interpreter: &mut dyn Interpreter) -> error::Result<()> {
    let path = interpreter.pop_as_string()?;
#[cfg(feature = "uses_iceoryx2")]
fn is_iceoryx2_fd(fd: i64) -> bool {
    ICEORYX2_STREAM_TABLE.with(|table| table.borrow().contains_key(&fd))
}

#[cfg(feature = "uses_iceoryx2")]
fn with_iceoryx2_stream<T, F: FnOnce(&mut Iceoryx2ByteStream) -> T>(fd: i64, f: F) -> Option<T> {
    ICEORYX2_STREAM_TABLE.with(|table| {
        let mut table = table.borrow_mut();
        table.get_mut(&fd).map(f)
    })
}

    #[cfg(unix)]
    {
        // Try Unix domain socket first
        match UnixStream::connect(&path) {
            Ok(stream) => {
                let fd = generate_fd();
                add_stream(fd, RawIpcStream::Unix(stream));
                interpreter.push(fd.to_value());
                return Ok(());
            }
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

    script_error(
        interpreter,
        format!("Failed to connect to any supported socket/pipe: {}", path),
    )?
}

fn word_file_size_read(interpreter: &mut dyn Interpreter) -> error::Result<()> {
    let fd = interpreter.pop_as_int()?;
    let file = get_file(interpreter, fd)?;

    match file {
        FileObject::File(file) => {
            let metadata = file.metadata()?;
            let size = metadata.len();

            interpreter.push(size.to_value());
        }

        FileObject::Stream(_) => {
            script_error_str(interpreter, "Can not read size of a socket.")?;
        }
    }

    Ok(())
}

fn word_file_exists(interpreter: &mut dyn Interpreter) -> error::Result<()> {
    let path = interpreter.pop_as_string()?;

    interpreter.push(Path::new(&path).exists().to_value());
    Ok(())
}

fn word_file_is_open(interpreter: &mut dyn Interpreter) -> error::Result<()> {
    let fd = interpreter.pop_as_int()?;
    let file = get_file(interpreter, fd);

    interpreter.push(file.is_ok().to_value());

    Ok(())
}

fn word_file_is_eof(interpreter: &mut dyn Interpreter) -> error::Result<()> {
    let fd = interpreter.pop_as_int()?;
    let file = get_file(interpreter, fd)?;

    match file {
        FileObject::File(mut file) => {
            let current_pos = file.stream_position()?;
            let total_size = file.metadata()?.len();

            interpreter.push((current_pos == total_size).to_value());
        }

        FileObject::Stream(_) => {
            script_error_str(interpreter, "Can not eof status of a socket.")?;
        }
    }

    Ok(())
}

fn word_file_read(interpreter: &mut dyn Interpreter) -> error::Result<()> {
    script_error_str(interpreter, "Unimplemented.")
}

fn word_file_read_character(interpreter: &mut dyn Interpreter) -> error::Result<()> {
    fn read<T>(interpreter: &mut dyn Interpreter, reader: &mut BufReader<T>) -> error::Result<()>
    where
        T: Read,
    {
        let mut buffer = [0; 1];

        match reader.read(&mut buffer) {
            Ok(0) => {
                interpreter.push("".to_string().to_value());
            }

            Ok(_) => {
                interpreter.push(buffer[0].to_string().to_value());
            }

            Err(error) => {
                return script_error(interpreter, format!("Could not read from file: {}.", error));
            }
        }

        Ok(())
    }

    let fd = interpreter.pop_as_int()?;
    let file = get_file(interpreter, fd)?;

    match file {
        FileObject::File(file) => read(interpreter, &mut BufReader::new(file)),
        FileObject::Stream(stream) => read(interpreter, &mut BufReader::new(stream)),
    }
}

fn word_file_read_string(interpreter: &mut dyn Interpreter) -> error::Result<()> {
    fn read<T>(interpreter: &mut dyn Interpreter, reader: &mut BufReader<T>) -> error::Result<()>
    where
        T: Read,
    {
        let mut string = String::new();

        match reader.read_to_string(&mut string) {
            Ok(0) => {
                interpreter.push("".to_string().to_value());
            }

            Ok(_) => {
                interpreter.push(string.to_value());
            }

            Err(error) => {
                return script_error(interpreter, format!("Could not read from file: {}.", error));
            }
        }

        Ok(())
    }

    let fd = interpreter.pop_as_int()?;
    let file = get_file(interpreter, fd)?;

    match file {
        FileObject::File(file) => read(interpreter, &mut BufReader::new(file)),
        FileObject::Stream(stream) => read(interpreter, &mut BufReader::new(stream)),
    }
}

fn word_file_write(interpreter: &mut dyn Interpreter) -> error::Result<()> {
    fn write<T>(
        interpreter: &mut dyn Interpreter,
        string: String,
        writer: &mut BufWriter<T>,
    ) -> error::Result<()>
    where
        T: Write,
    {
        let bytes = string.into_bytes();

        match writer.write_all(bytes.as_slice()) {
            // TODO: Handle partial writes.
            Ok(_) => Ok(()),

            Err(error) => {
                script_error(interpreter, format!("Could not read from file: {}.", error))
            }
        }
    }

    // TODO: Implement ByteBuffer and better string conversion.
    let fd = interpreter.pop_as_int()?;
    let string = interpreter.pop_as_string()?;
    let file = get_file(interpreter, fd)?;

    match file {
        FileObject::File(file) => write(interpreter, string, &mut BufWriter::new(file)),
        FileObject::Stream(stream) => write(interpreter, string, &mut BufWriter::new(stream)),
    }
}

fn word_file_line_read(interpreter: &mut dyn Interpreter) -> error::Result<()> {
    fn read<T>(interpreter: &mut dyn Interpreter, buffer: &mut BufReader<T>) -> error::Result<()>
    where
        T: Read,
    {
        let mut line = String::new();

        match buffer.read_line(&mut line) {
            Ok(0) => {
                interpreter.push("".to_string().to_value());
            }

            Ok(_) => {
                let line = line.trim_end_matches(&['\n', '\r'][..]).to_string();
                interpreter.push(line.to_value());
            }

            Err(error) => {
                return script_error(interpreter, format!("Could not read from file: {}.", error));
            }
        }

        Ok(())
    }

    let fd = interpreter.pop_as_int()?;
    let file = get_file(interpreter, fd)?;

    match file {
        FileObject::File(file) => read(interpreter, &mut BufReader::new(file)),
        FileObject::Stream(stream) => read(interpreter, &mut BufReader::new(stream)),
    }
}

fn word_file_line_write(interpreter: &mut dyn Interpreter) -> error::Result<()> {
    fn write<T>(
        interpreter: &mut dyn Interpreter,
        string: String,
        writer: &mut BufWriter<T>,
    ) -> error::Result<()>
    where
        T: Write,
    {
        let bytes = (string + "\n").into_bytes();

        match writer.write_all(bytes.as_slice()) {
            // TODO: Handle partial writes.
            Ok(_) => Ok(()),

            Err(error) => {
                script_error(interpreter, format!("Could not read from file: {}.", error))
            }
        }
    }

    // TODO: Implement better string conversion.
    let fd = interpreter.pop_as_int()?;
    let string = interpreter.pop_as_string()?;
    let file = get_file(interpreter, fd)?;

    match file {
        FileObject::File(file) => write(interpreter, string, &mut BufWriter::new(file)),
        FileObject::Stream(stream) => write(interpreter, string, &mut BufWriter::new(stream)),
    }
}

pub fn register_io_words(interpreter: &mut dyn Interpreter) {
            crate::add_native_word!(
                interpreter,
                "process.spawn",
                word_process_spawn,
                "Spawn a new process to run a script. Usage: 'script_path process.spawn'. Returns exit code.",
                "script_path -- exit_code"
            );
        // Native word to spawn a new process running the interpreter with a given script
        fn word_process_spawn(interpreter: &mut dyn Interpreter) -> error::Result<()> {
            use std::process::Command;
            let script_path = interpreter.pop_as_string()?;
            // Try to find the current executable
            let exe = match std::env::current_exe() {
                Ok(e) => e,
                Err(e) => return script_error_str(interpreter, &format!("process.spawn: could not get current exe: {e}")),
            };
            let output = match Command::new(exe).arg(&script_path).output() {
                Ok(o) => o,
                Err(e) => return script_error_str(interpreter, &format!("process.spawn: failed to launch: {e}")),
            };
            let exit_code = output.status.code().unwrap_or(-1);
            use crate::runtime::data_structures::value::ToValue;
            interpreter.push((exit_code as i64).to_value());
            Ok(())
        }
        #[cfg(feature = "uses_iceoryx2")]
        crate::add_native_word!(
            interpreter,
            "iox.pub",
            word_iox_pub,
            "Create an iceoryx2 publisher for a service.",
            "service -- pub"
        );
        #[cfg(feature = "uses_iceoryx2")]
        crate::add_native_word!(
            interpreter,
            "iox.sub",
            word_iox_sub,
            "Create an iceoryx2 subscriber for a service.",
            "service -- sub"
        );
        #[cfg(feature = "uses_iceoryx2")]
        crate::add_native_word!(
            interpreter,
            "iox.pub!",
            word_iox_pub_send,
            "Send a message using an iceoryx2 publisher.",
            "string pub -- "
        );
        #[cfg(feature = "uses_iceoryx2")]
        crate::add_native_word!(
            interpreter,
            "iox.sub@",
            word_iox_sub_recv,
            "Receive a message using an iceoryx2 subscriber.",
            "sub -- string"
        );
    crate::add_native_word!(
        interpreter,
        "file.open",
        word_file_open,
        "Open an existing file and return a fd.",
        "path flags -- fd"
    );

    crate::add_native_word!(
        interpreter,
        "file.create",
        word_file_create,
        "Create/open a file and return a fd.",
        "path flags -- fd"
    );

    crate::add_native_word!(
        interpreter,
        "file.create.tempfile",
        word_file_create_temp_file,
        "Create/open an unique temporary file and return it's fd.",
        "flags -- path fd"
    );

    crate::add_native_word!(
        interpreter,
        "file.close",
        word_file_close,
        "Take a fd and close it.",
        "fd -- "
    );

    crate::add_native_word!(
        interpreter,
        "file.delete",
        word_file_delete,
        "Delete the specified file.",
        "file_path -- "
    );

    crate::add_native_word!(
        interpreter,
        "socket.connect",
        word_socket_connect,
        "Connect to Unix domain socket at the given path.",
        "path -- fd"
    );

    crate::add_native_word!(
        interpreter,
        "file.size@",
        word_file_size_read,
        "Return the size of a file represented by a fd.",
        "fd -- size"
    );

    crate::add_native_word!(
        interpreter,
        "file.exists?",
        word_file_exists,
        "Does the file at the given path exist?",
        "path -- bool"
    );

    crate::add_native_word!(
        interpreter,
        "file.is_open?",
        word_file_is_open,
        "Is the fd currently valid?",
        "fd -- bool"
    );

    crate::add_native_word!(
        interpreter,
        "file.is_eof?",
        word_file_is_eof,
        "Is the file pointer at the end of the file?",
        "fd -- bool"
    );

    crate::add_native_word!(
        interpreter,
        "file.@",
        word_file_read,
        "Read from a given file.  (Unimplemented.)",
        " -- "
    );

    crate::add_native_word!(
        interpreter,
        "file.char@",
        word_file_read_character,
        "Read a character from a given file.",
        "fd -- character"
    );

    crate::add_native_word!(
        interpreter,
        "file.string@",
        word_file_read_string,
        "Read a file to a string.",
        "fd -- string"
    );

    crate::add_native_word!(
        interpreter,
        "file.!",
        word_file_write,
        "Write a value as text to a file, unless it's a ByteBuffer.",
        "value fd -- "
    );

    crate::add_native_word!(
        interpreter,
        "file.line@",
        word_file_line_read,
        "Read a full line from a file.",
        "fd -- string"
    );

    crate::add_native_word!(
        interpreter,
        "file.line!",
        word_file_line_write,
        "Write a string as a line to the file.",
        "string fd -- "
    );

    crate::add_native_word!(
        interpreter,
        "file.r/o",
        |interpreter| {
            interpreter.push(0b0001_i64.to_value());
            Ok(())
        },
        "Constant for opening a file as read only.",
        " -- flag"
    );

    crate::add_native_word!(
        interpreter,
        "file.w/o",
        |interpreter| {
            interpreter.push(0b0010_i64.to_value());
            Ok(())
        },
        "Constant for opening a file as write only.",
        " -- flag"
    );

    crate::add_native_word!(
        interpreter,
        "file.r/w",
        |interpreter| {
            interpreter.push(0b0011_i64.to_value());
            Ok(())
        },
        "Constant for opening a file for both reading and writing.",
        " -- flag"
    );
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    #[test]
    fn test_raw_ipcstream_unix() {
        use std::fs;
        use std::os::unix::net::{UnixListener, UnixStream};
        use std::path::PathBuf;
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
        use named_pipe::{PipeClient, PipeOptions};
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
        let mut client =
            RawIpcStream::NamedPipe(PipeClient::connect(pipe_name).expect("connect failed"));
        client.write_all(b"ping").expect("write failed");
        let mut buf = [0u8; 4];
        client.read_exact(&mut buf).expect("read failed");
        assert_eq!(&buf, b"pong");
        server.join().unwrap();
    }
    use super::RawIpcStream;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread;

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
