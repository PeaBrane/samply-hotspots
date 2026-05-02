mod args;
mod counts;
mod output;
mod parse;
mod profile;
mod run;
mod symbol;

use anyhow::{Context, Result};
use clap::Parser;
use std::fs::File;
use std::io::BufReader;

use crate::args::Args;
use crate::profile::Profile;

fn main() -> Result<()> {
    let args = Args::parse();
    args.validate()?;

    let file = File::open(&args.profile)
        .with_context(|| format!("failed to read profile {}", args.profile.display()))?;
    let profile: Profile = serde_json::from_reader(BufReader::new(file))
        .with_context(|| format!("failed to parse profile {}", args.profile.display()))?;

    run::run(&args, &profile)
}
