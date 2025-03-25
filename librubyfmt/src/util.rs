use ruby_prism::Location;

pub fn u8_to_string(arr: &[u8]) -> String {
    std::str::from_utf8(arr).unwrap().to_string()
}

pub fn loc_to_string(loc: Location) -> String {
    u8_to_string(loc.as_slice())
}
