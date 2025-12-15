use ruby_prism::{ConstantId, Location};

pub fn u8_to_str(arr: &[u8]) -> &str {
    std::str::from_utf8(arr).unwrap()
}

pub fn u8_to_string(arr: &[u8]) -> String {
    u8_to_str(arr).to_string()
}

pub fn const_to_string(constant_id: ConstantId) -> String {
    u8_to_string(constant_id.as_slice())
}

pub fn const_to_str(constant_id: ConstantId<'_>) -> &str {
    u8_to_str(constant_id.as_slice())
}

pub fn loc_to_str(loc: Location<'_>) -> &str {
    u8_to_str(loc.as_slice())
}

pub fn loc_to_string(loc: Location) -> String {
    u8_to_string(loc.as_slice())
}
