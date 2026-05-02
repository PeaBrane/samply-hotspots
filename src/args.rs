use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "samply-hotspots",
    about = "Summarize macOS samply JSON profiles with atos symbolication."
)]
pub(crate) struct Args {
    /// Path to samply JSON profile
    #[arg(long)]
    pub(crate) profile: PathBuf,

    /// Single-image binary path for Rust-only profiles
    #[arg(long)]
    pub(crate) binary: Option<PathBuf>,

    /// Mach-O text segment base for single-image profiles
    #[arg(long, default_value = "0x100000000")]
    pub(crate) base: String,

    /// Resolve each frame against its owning image for Python/Rust mixed profiles
    #[arg(long)]
    pub(crate) mixed: bool,

    /// Weight samples by threadCPUDelta when present instead of raw sample rows
    #[arg(long)]
    pub(crate) cpu_weighted: bool,

    /// Number of functions to print in each section
    #[arg(long, default_value_t = 25)]
    pub(crate) top: usize,

    /// Number of top leaf + total entries to symbolicate for the summary
    #[arg(long, default_value_t = 40)]
    pub(crate) resolve_limit: usize,

    /// Optional function substrings for line-level inclusive breakdowns
    #[arg(long, num_args = 0..)]
    pub(crate) targets: Vec<String>,

    /// Number of raw addresses to symbolicate for line-level breakdowns
    #[arg(long, default_value_t = 200)]
    pub(crate) line_resolve_limit: usize,
}

impl Args {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.mixed && !self.targets.is_empty() {
            bail!("--targets is only supported for single-image profiles");
        }
        Ok(())
    }
}
