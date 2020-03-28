use rutie::rubysys::array::*;
use rutie::rubysys::value::*;
use serde::de::{*, Error as _};
use rutie::rubysys::fixnum::*;

pub fn from_value<T: DeserializeOwned>(v: Value) -> Result<T> {
    T::deserialize(Deserializer(v))
}

struct Deserializer(Value);

type Result<T> = std::result::Result<T, serde::de::value::Error>;
type Error = serde::de::value::Error;

impl<'de> serde::Deserializer<'de> for Deserializer {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.0.ty() {
            ValueType::Symbol => visitor.visit_borrowed_str(sym_to_str(self.0)?),
            ValueType::RString => visitor.visit_borrowed_str(rstring_to_str(self.0)?),
            ValueType::Array => visitor.visit_seq(SeqAccess::new(self.0)),
            ValueType::Nil => visitor.visit_none(),
            ValueType::True => visitor.visit_bool(true),
            ValueType::False => visitor.visit_bool(false),
            ValueType::Fixnum => visitor.visit_i64(unsafe { rb_num2ll(self.0) }),
            other => {
                Err(serde::de::Error::custom(format_args!("Unexpected type {:?}", other)))
            }
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct SeqAccess {
    arr: Value,
    idx: usize,
    len: usize,
}

impl SeqAccess {
    fn new(arr: Value) -> Self {
        let len = unsafe { rb_ary_len(arr) } as usize;
        Self { arr, len, idx: 0 }
    }
}

impl<'de> serde::de::SeqAccess<'de> for SeqAccess {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        if self.idx < self.len {
            let elem = unsafe { rb_ary_entry(self.arr, self.idx as _) };
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

fn sym_to_str(v: Value) -> Result<&'static str> {
    use rutie::rubysys::symbol::*;
    use std::ffi::CStr;

    unsafe {
        let id = rb_sym2id(v);
        let c_str = CStr::from_ptr(rb_id2name(id));
        c_str.to_str()
            .map_err(|_| invalid_utf8(c_str.to_bytes()))
    }
}

fn rstring_to_str(v: Value) -> Result<&'static str> {
    use rutie::rubysys::string::*;

    unsafe {
        let bytes = std::slice::from_raw_parts(
            rstring_ptr(v) as _,
            rstring_len(v) as _,
        );
        std::str::from_utf8(bytes)
            .map_err(|_| invalid_utf8(bytes))
    }
}

fn invalid_utf8(bytes: &[u8]) -> Error {
    Error::invalid_value(
        Unexpected::Bytes(bytes),
        &"a valid UTF-8 string",
    )
}
