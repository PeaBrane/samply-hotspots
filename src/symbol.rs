use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use crate::counts::MixedKey;

const RUST_NAME_REPLACEMENTS: &[(&str, &str)] = &[
    ("_$LT$", "<"),
    ("$LT$", "<"),
    ("$GT$", ">"),
    ("$u20$", " "),
    ("$u7b$$u7b$", "{{"),
    ("$u7d$$u7d$", "}}"),
    ("$RF$", "&"),
    ("$C$", ","),
    ("$LP$", "("),
    ("$RP$", ")"),
];

pub(crate) fn resolve_single(
    raw_names: &HashSet<String>,
    binary: &Path,
    base: u64,
) -> Result<HashMap<String, String>> {
    let mut resolved = HashMap::new();
    let mut addresses = Vec::new();

    for raw in raw_names {
        match parse_hex(raw) {
            Some(addr) => addresses.push((raw.clone(), base + addr)),
            None => {
                resolved.insert(raw.clone(), prettify(raw));
            }
        }
    }

    for (raw, symbol) in run_atos_batched(binary, base, &addresses)? {
        resolved.insert(raw, symbol);
    }
    Ok(resolved)
}

pub(crate) fn resolve_mixed(keys: &HashSet<MixedKey>) -> Result<HashMap<MixedKey, String>> {
    let mut resolved = HashMap::new();
    let mut by_image: HashMap<String, Vec<(String, u64)>> = HashMap::new();

    for key in keys {
        match parse_hex(&key.raw) {
            Some(addr) => {
                if let Some(image) = &key.image {
                    by_image
                        .entry(image.clone())
                        .or_default()
                        .push((key.raw.clone(), addr));
                } else {
                    resolved.insert(key.clone(), key.raw.clone());
                }
            }
            None => {
                resolved.insert(key.clone(), prettify(&key.raw));
            }
        }
    }

    for (image, addresses) in by_image {
        for (raw, symbol) in run_atos_batched(Path::new(&image), 0, &addresses)? {
            resolved.insert(
                MixedKey {
                    image: Some(image.clone()),
                    raw,
                },
                symbol,
            );
        }
    }

    Ok(resolved)
}

pub(crate) fn run_atos_batched(
    binary: &Path,
    load_base: u64,
    addresses: &[(String, u64)],
) -> Result<Vec<(String, String)>> {
    let mut out = Vec::with_capacity(addresses.len());

    for chunk in addresses.chunks(256) {
        let mut command = Command::new("atos");
        command
            .arg("-o")
            .arg(binary)
            .arg("-l")
            .arg(format_hex(load_base));
        for (_, address) in chunk {
            command.arg(format_hex(*address));
        }

        let output = command
            .output()
            .with_context(|| format!("failed to run atos for {}", binary.display()))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut lines = stdout.lines();
        for (raw, _) in chunk {
            let symbol = lines
                .next()
                .map(clean_atos_symbol)
                .filter(|symbol| !symbol.is_empty())
                .unwrap_or_else(|| raw.clone());
            out.push((raw.clone(), symbol));
        }
    }

    Ok(out)
}

pub(crate) fn parse_int_auto(value: &str) -> Result<u64> {
    let trimmed = value.trim();
    if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        return u64::from_str_radix(hex, 16).map_err(Into::into);
    }
    trimmed.parse::<u64>().map_err(Into::into)
}

pub(crate) fn parse_hex(value: &str) -> Option<u64> {
    let trimmed = value.trim();
    let hex = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    u64::from_str_radix(hex, 16).ok()
}

fn format_hex(value: u64) -> String {
    format!("0x{value:x}")
}

fn clean_atos_symbol(symbol: &str) -> String {
    let trimmed = symbol.trim();
    let without_image = trimmed
        .split_once("(in ")
        .map(|(name, _)| name.trim())
        .unwrap_or(trimmed);
    prettify(without_image)
}

fn prettify(name: &str) -> String {
    let mut pretty = trim_hash_suffix(name).to_string();
    for (old, new) in RUST_NAME_REPLACEMENTS {
        pretty = pretty.replace(old, new);
    }
    pretty
}

fn trim_hash_suffix(name: &str) -> &str {
    let Some((prefix, suffix)) = name.rsplit_once("::h") else {
        return name;
    };
    if suffix.len() != 16 || !suffix.bytes().all(|ch| ch.is_ascii_hexdigit()) {
        return name;
    }
    prefix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_rust_hash_suffix() {
        assert_eq!(prettify("foo::bar::h0123456789abcdef"), "foo::bar");
        assert_eq!(
            prettify("foo::bar::hnotreallyhash"),
            "foo::bar::hnotreallyhash"
        );
    }

    #[test]
    fn parses_hex_with_or_without_prefix() {
        assert_eq!(parse_hex("0x10"), Some(16));
        assert_eq!(parse_hex("10"), Some(16));
        assert_eq!(parse_hex("not_hex"), None);
    }
}
