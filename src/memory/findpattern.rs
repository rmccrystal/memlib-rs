extern crate regex;
use self::regex::bytes::Regex;
use crate::memory::Address;
use log::*;

/// Enables the user to generate a byte regex out of the normal signature
/// format.
pub fn generate_regex(raw: &str) -> Option<Regex> {
    let mut res = raw
        .to_string()
        .split_whitespace()
        .map(|x| match &x {
            &"?" => ".".to_string(),
            x => format!("\\x{}", x),
        })
        .collect::<Vec<_>>()
        .join("");
    res.insert_str(0, "(?s-u)");
    Regex::new(&res).ok()
}

/// Find pattern.
pub fn find_pattern(data: &[u8], pattern: &str) -> Option<Address> {
    let result = generate_regex(pattern)
        .and_then(|r| r.find(data))
        .map(|m| m.start() as u64);
    if result.is_some() {
        let result = result.unwrap();
        debug!("Found pattern {} at offset 0x{:X}", pattern, result);
        return Some(result as Address);
    }
    None
}
