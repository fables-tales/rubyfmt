use std::io::{Write, Stdout, self};
use std::fs::File;
use std::slice;

pub struct Writer<T: Write> {
    handle: T
}

type StdoutWriter = Writer<Stdout>;
type FileWriter = Writer<File>;

impl <T:Write> Writer<T> {
    pub fn new(handle: T) -> Self {
        Writer {
            handle: handle
        }
    }

    pub fn write_next_bytes(&mut self, bytes: *const u8, length: i64) -> io::Result<usize> {
        let b = unsafe { slice::from_raw_parts(bytes, length as usize) };
        let res = self.handle.write(b);
        self.handle.flush()?;
        res
    }
}

#[no_mangle]
pub extern fn writer_open_handle_or_panic(name_bytes: *mut u8, name_length: i64) -> *mut FileWriter {
    let b = unsafe { slice::from_raw_parts(name_bytes, name_length as usize) };
    let s = std::str::from_utf8(b).expect("couldn't convert filename");
    let f = File::create(s.clone()).expect(&format!("couldn't open {}", s));
    Box::into_raw(Box::new(Writer::new(f)))
}

#[no_mangle]
pub extern fn writer_open_stdout() -> *mut StdoutWriter {
    Box::into_raw(Box::new(Writer::new(io::stdout())))
}

#[no_mangle]
pub extern fn writer_file_writer_write_bytes_or_panic(writer: *mut FileWriter, bytes: *mut u8, length: i64) {
    let mw = unsafe { &mut *writer };
    mw.write_next_bytes(bytes, length).expect("couldn't write data to file");
}

#[no_mangle]
pub extern fn writer_stdout_writer_write_bytes_or_panic(writer: *mut StdoutWriter, bytes: *mut u8, length: i64) {
    let mw = unsafe { &mut *writer };
    mw.write_next_bytes(bytes, length).expect("couldn't write data to stdout");
}
