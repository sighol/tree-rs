//! tree-rs: A cross-platform reimplementation of the Unix `tree` command.
//!
//! Displays directory structures in a tree-like format with colored output
//! and support for filtering, depth limiting, and pattern matching.

#![deny(clippy::pedantic)]
#![deny(clippy::all)]

mod config;
mod pathiterator;
mod tree_printer;

#[cfg(test)]
mod tests;

use clap::Parser;
use config::{Args, Config};
use tree_printer::TreePrinter;

use anyhow::{Context, Result};
use std::io::{self, Write};
use std::path::Path;

use term::{Terminal, TerminfoTerminal};
use tree_printer::DirEntrySummary;

/// Main application logic - extracted for testing
///
/// # Errors
///
/// Returns an error if:
/// - The directory cannot be read or iterated
/// - Writing output to the terminal fails
pub fn run<W: Write>(
    config: Config,
    path: &Path,
    only_dirs: bool,
    term: &mut impl Terminal<Output = W>,
) -> Result<DirEntrySummary> {
    let summary = {
        let mut p = TreePrinter::new(config, term);
        p.iterate_folders(path)
            .context("Failed to iterate folders")?
    };

    if only_dirs {
        writeln!(term, "\n{} directories", summary.num_folders)
    } else {
        writeln!(
            term,
            "\n{} directories, {} files",
            summary.num_folders, summary.num_files
        )
    }
    .context("Failed to print summary")?;

    Ok(summary)
}

fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::try_from(&args)?;
    let path = Path::new(args.dir.as_str());

    let mut term = TerminfoTerminal::new(io::stdout())
        .ok_or_else(|| anyhow::anyhow!("Could not find colored terminal"))?;

    run(config, path, args.only_dirs, &mut term)?;

    Ok(())
}
