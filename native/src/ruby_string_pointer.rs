use std::slice;

#[repr(C)]
pub struct RubyStringPointer {
    bytes: *const char,
    length: i64,
}

impl RubyStringPointer {
    pub fn as_buf(self) -> &'static [u8] {
        unsafe { slice::from_raw_parts(self.bytes as *const u8, self.length as usize) }
    }
}
