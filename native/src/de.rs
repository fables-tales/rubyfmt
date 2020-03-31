use crate::ruby::{self, VALUE};
use serde::de::{self, Error as _};

pub fn from_value<T: de::DeserializeOwned>(v: VALUE) -> Result<T> {
    T::deserialize(Deserializer(v))
}

struct Deserializer(VALUE);

type Result<T> = std::result::Result<T, serde::de::value::Error>;
type Error = serde::de::value::Error;

impl<'de> serde::Deserializer<'de> for Deserializer {
    type Error = Error;

    fn deserialize_any<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        pub use ruby::ruby_value_type::*;

        match unsafe { ruby::rubyfmt_rb_type(self.0) } {
            RUBY_T_SYMBOL => visitor.visit_borrowed_str(sym_to_str(self.0)?),
            RUBY_T_STRING => visitor.visit_borrowed_str(rstring_to_str(self.0)?),
            RUBY_T_ARRAY => visitor.visit_seq(SeqAccess::new(self.0)),
            RUBY_T_NIL => visitor.visit_none(),
            RUBY_T_TRUE => visitor.visit_bool(true),
            RUBY_T_FALSE => visitor.visit_bool(false),
            RUBY_T_FIXNUM => visitor.visit_i64(unsafe { ruby::rubyfmt_rb_num2ll(self.0) }),
            other => Err(serde::de::Error::custom(format_args!(
                "Unexpected type {:?}",
                other
            ))),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct SeqAccess {
    arr: VALUE,
    idx: usize,
    len: usize,
}

impl SeqAccess {
    fn new(arr: VALUE) -> Self {
        let len = unsafe { ruby::rubyfmt_rb_ary_len(arr) } as usize;
        Self { arr, len, idx: 0 }
    }
}

impl<'de> de::SeqAccess<'de> for SeqAccess {
    type Error = Error;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>> {
        if self.idx < self.len {
            let elem = unsafe { ruby::rb_ary_entry(self.arr, self.idx as _) };
            self.idx += 1;
            seed.deserialize(Deserializer(elem)).map(Some)
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.len - self.idx)
    }
}

fn sym_to_str(v: VALUE) -> Result<&'static str> {
    use std::ffi::CStr;

    unsafe {
        let id = ruby::rb_sym2id(v);
        let c_str = CStr::from_ptr(ruby::rb_id2name(id));
        c_str.to_str().map_err(|_| invalid_utf8(c_str.to_bytes()))
    }
}

fn rstring_to_str(v: VALUE) -> Result<&'static str> {
    unsafe {
        let bytes = std::slice::from_raw_parts(
            ruby::rubyfmt_rstring_ptr(v) as _,
            ruby::rubyfmt_rstring_len(v) as _,
        );
        std::str::from_utf8(bytes).map_err(|_| invalid_utf8(bytes))
    }
}

fn invalid_utf8(bytes: &[u8]) -> Error {
    Error::invalid_value(de::Unexpected::Bytes(bytes), &"a valid UTF-8 string")
}
