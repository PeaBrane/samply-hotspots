use std::collections::HashSet;

use anyhow::{Context, Result};

use crate::args::Args;
use crate::counts::sorted_top;
use crate::output::{metric_text, print_counts, print_line_level, print_mixed_counts};
use crate::parse::{parse_mixed_image_profile, parse_single_image_profile};
use crate::profile::Profile;
use crate::symbol::{parse_int_auto, resolve_mixed, resolve_single};

pub(crate) fn run(args: &Args, profile: &Profile) -> Result<()> {
    if args.mixed {
        run_mixed(args, profile)
    } else {
        let binary = args
            .binary
            .as_ref()
            .context("--binary is required unless --mixed is set")?;
        let base = parse_int_auto(&args.base)
            .with_context(|| format!("invalid Mach-O base address {:?}", args.base))?;
        run_single(args, profile, binary, base)
    }
}

fn run_single(args: &Args, profile: &Profile, binary: &std::path::Path, base: u64) -> Result<()> {
    let counts = parse_single_image_profile(profile, binary, args.cpu_weighted)?;
    let mut names_to_resolve = HashSet::new();
    for (name, _) in sorted_top(&counts.leaf, args.resolve_limit) {
        names_to_resolve.insert(name);
    }
    for (name, _) in sorted_top(&counts.all, args.resolve_limit) {
        names_to_resolve.insert(name);
    }

    let resolved = resolve_single(&names_to_resolve, binary, base)?;
    let total = counts.leaf.values().sum();
    println!(
        "Total target-binary {}: {}",
        if args.cpu_weighted {
            "CPU seconds"
        } else {
            "samples"
        },
        metric_text(total, args.cpu_weighted)
    );
    println!();
    print_counts(
        "=== Top by SELF time (leaf/nearest target frame) ===",
        &counts.leaf,
        total,
        &resolved,
        args.top,
        args.cpu_weighted,
    );
    println!();
    print_counts(
        "=== Top by TOTAL time (inclusive) ===",
        &counts.all,
        total,
        &resolved,
        args.top,
        args.cpu_weighted,
    );

    if !args.targets.is_empty() {
        println!();
        print_line_level(
            &args.targets,
            &counts.addr,
            binary,
            base,
            args.line_resolve_limit,
            args.cpu_weighted,
        )?;
    }

    Ok(())
}

fn run_mixed(args: &Args, profile: &Profile) -> Result<()> {
    let counts = parse_mixed_image_profile(profile, args.cpu_weighted)?;
    let mut keys_to_resolve = HashSet::new();
    for (key, _) in sorted_top(&counts.leaf, args.resolve_limit) {
        keys_to_resolve.insert(key);
    }
    for (key, _) in sorted_top(&counts.all, args.resolve_limit) {
        keys_to_resolve.insert(key);
    }

    let resolved = resolve_mixed(&keys_to_resolve)?;
    let total = counts.leaf.values().sum();
    println!(
        "Total {}: {}",
        if args.cpu_weighted {
            "CPU seconds"
        } else {
            "samples"
        },
        metric_text(total, args.cpu_weighted)
    );
    println!();
    print_mixed_counts(
        "=== Top by SELF time (leaf) ===",
        &counts.leaf,
        total,
        &resolved,
        args.top,
        args.cpu_weighted,
    );
    println!();
    print_mixed_counts(
        "=== Top by TOTAL time (inclusive) ===",
        &counts.all,
        total,
        &resolved,
        args.top,
        args.cpu_weighted,
    );

    Ok(())
}
