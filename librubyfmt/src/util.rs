use ruby_prism::{ConstantId, Location};

pub fn u8_to_string(arr: &[u8]) -> String {
    std::str::from_utf8(arr).unwrap().to_string()
}

pub fn const_to_string(constant_id: ConstantId) -> String {
    u8_to_string(constant_id.as_slice())
}

pub fn loc_to_string(loc: Location) -> String {
    u8_to_string(loc.as_slice())
}
