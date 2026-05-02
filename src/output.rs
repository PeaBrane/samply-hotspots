use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;

use crate::counts::{Count, MixedKey, add_count, sorted_top};
use crate::symbol::{parse_hex, run_atos_batched};

pub(crate) fn print_counts(
    title: &str,
    counts: &HashMap<String, Count>,
    total: Count,
    resolved: &HashMap<String, String>,
    top: usize,
    cpu_weighted: bool,
) {
    println!("{title}");
    for (key, count) in sorted_top(counts, top) {
        let pct = percentage(count, total);
        let name = resolved.get(&key).unwrap_or(&key);
        println!("  {pct:5.1}%  {}  {name}", metric_text(count, cpu_weighted));
    }
}

pub(crate) fn print_mixed_counts(
    title: &str,
    counts: &HashMap<MixedKey, Count>,
    total: Count,
    resolved: &HashMap<MixedKey, String>,
    top: usize,
    cpu_weighted: bool,
) {
    println!("{title}");
    for (key, count) in sorted_top(counts, top) {
        let pct = percentage(count, total);
        let name = resolved.get(&key).unwrap_or(&key.raw);
        println!("  {pct:5.1}%  {}  {name}", metric_text(count, cpu_weighted));
    }
}

pub(crate) fn print_line_level(
    targets: &[String],
    addr_counts: &HashMap<String, Count>,
    binary: &Path,
    base: u64,
    resolve_limit: usize,
    cpu_weighted: bool,
) -> Result<()> {
    let mut addresses = Vec::new();
    for (raw, _) in sorted_top(addr_counts, resolve_limit) {
        let Some(addr) = parse_hex(&raw) else {
            continue;
        };
        addresses.push((raw, base + addr));
    }

    let resolved: HashMap<_, _> = run_atos_batched(binary, base, &addresses)?
        .into_iter()
        .collect();
    for target in targets {
        let mut line_counts = HashMap::new();
        for (raw, count) in addr_counts {
            let Some(symbol) = resolved.get(raw) else {
                continue;
            };
            if !symbol.contains(target) {
                continue;
            }

            let location = symbol
                .rsplit_once('(')
                .map(|(_, location)| location.trim_end_matches(')').to_string())
                .unwrap_or_else(|| symbol.clone());
            add_count(&mut line_counts, location, *count);
        }

        if line_counts.is_empty() {
            println!("=== {target}: no samples ===");
            println!();
            continue;
        }

        let total = line_counts.values().sum();
        if cpu_weighted {
            println!(
                "=== {target}: {} inclusive CPU time ===",
                metric_text(total, true).trim()
            );
        } else {
            println!("=== {target}: {total} inclusive samples ===");
        }
        for (location, count) in sorted_top(&line_counts, 20) {
            let pct = percentage(count, total);
            println!(
                "  {pct:5.1}%  {}  {location}",
                metric_text(count, cpu_weighted)
            );
        }
        println!();
    }

    Ok(())
}

pub(crate) fn metric_text(count: Count, cpu_weighted: bool) -> String {
    if cpu_weighted {
        format!("{:8.3}s", count as f64 / 1_000_000.0)
    } else {
        format!("{count:6}")
    }
}

fn percentage(count: Count, total: Count) -> f64 {
    if total == 0 {
        0.0
    } else {
        count as f64 / total as f64 * 100.0
    }
}
