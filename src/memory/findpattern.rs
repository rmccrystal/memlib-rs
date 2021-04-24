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
    /// If you need to add base
    pub rip_relative: bool
}

/// Brute forces offsets and indices to find an OffsetDefinition from a sig and value
/// If base is some rip_relative is true, base should be game base addr.
pub fn offset_definition_from_sig(data: &[u8], sig: &str, value: u64, name: &str, dword: bool, max_offset: usize, neg_offset: usize, base: Option<u64>) -> Result<OffsetDefinition> {
    let matches = find_pattern(&data, &sig).ok_or_else(|| anyhow!("Could not find any matches for sig {} ({})", sig, name))?;

    let add = base.unwrap_or(0);
    let byte_value = if dword {
        u32::to_le_bytes(value as u32 + add as u32).to_vec()
    } else {
        u64::to_le_bytes(value + add as u64).to_vec()
    };

    for (i, data_index) in matches.iter().enumerate() {
        let data_index = *data_index as usize;

        let offset = find_subsequence(&data[data_index-neg_offset..data_index + max_offset], &byte_value);
        if let Some(offset) = offset {
            return Ok(OffsetDefinition {
                sig: sig.to_string(),
                offset,
                dword,
                name: name.to_string(),
                index: i,
                rip_relative: base.is_some()
            });
        }
    }

    Err(anyhow!("No signatures matches for {}", name))
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

pub fn find_offset(data: &[u8], info: &OffsetDefinition) -> Result<u64> {
    let addrs = find_pattern(&data, &info.sig).ok_or_else(|| anyhow!("Could not find pattern {}", info.sig))?;
    let base = *addrs.get(info.index).ok_or_else(|| anyhow!("Could not get index {}, len = {}", info.index, addrs.len()))? as usize;
    let addr = base + info.offset;
    if !info.dword {
        Ok(u64::from_le_bytes(data[addr..addr + 8].try_into().unwrap()))
    } else {
        Ok(u32::from_le_bytes(data[addr..addr + 4].try_into().unwrap()) as _)
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
