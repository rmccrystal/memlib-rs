extern crate regex;

use self::regex::bytes::Regex;
use crate::memory::Address;
use anyhow::*;
use log::*;
use std::convert::TryInto;

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
pub fn find_pattern(data: &[u8], pattern: &str) -> Option<Vec<Address>> {
    let r = generate_regex(pattern)?;
    let matches: Vec<_> = r.find_iter(&data).map(|n| n.start() as _).collect();
    if matches.is_empty() {
        None
    } else {
        Some(matches)
    }
}

/// The data required to find an offset
#[derive(Clone, Default, Debug)]
pub struct OffsetDefinition {
    pub sig: String,
    pub name: String,
    pub index: usize,
    /// The bytes to add to the pattern base
    pub offset: usize,
    /// True if the offset should only be 4 bytes instead of 8
    pub dword: bool,
}

pub fn find_offset(data: &[u8], info: &OffsetDefinition) -> Result<u64> {
    let addrs = find_pattern(&data, &info.sig).ok_or_else(|| anyhow!("Could not find pattern {}", info.sig))?;
    let base = *addrs.get(info.index).ok_or_else(|| anyhow!("Could not get index {}, len = {}", info.index, addrs.len()))? as usize;
    let addr = base + info.offset;
    if !info.dword {
        Ok(u64::from_le_bytes(data[addr..addr+8].try_into().unwrap()))
    } else {
        Ok(u32::from_le_bytes(data[addr..addr+4].try_into().unwrap()) as _)
    }
}

/// Dumps offsets into Rust code as a string
pub fn dump_offsets(data: &[u8], defs: &[OffsetDefinition]) -> String {
    let mut buf = String::new();

    for def in defs {
        let offset = find_offset(&data, def).unwrap_or_else(|e| {
            error!("Error finding OffsetDefinition {:?}: {:?}", def, e);
            0
        });

        buf.push_str(&format!("pub const {}: Address = 0x{:X}\n", def.name, offset));
    }

    buf
}
