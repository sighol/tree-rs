//! tree-rs: A cross-platform reimplementation of the Unix `tree` command.
//!
//! Displays directory structures in a tree-like format with colored output
//! and support for filtering, depth limiting, and pattern matching.

#![deny(clippy::pedantic)]
#![deny(clippy::all)]

mod pathiterator;
mod tree_printer;

#[cfg(test)]
mod tests;

use clap::Parser;
use tree_printer::{Config, TreePrinter};

use anyhow::{Context, Result};
use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::sync::Arc;

use globset::Glob;
use term::TerminfoTerminal;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Parser)]
struct Args {
    /// Show hidden files
    #[clap(short = 'a', long = "all")]
    show_all: bool,
    /// Turn colorization on always
    #[clap(short = 'C')]
    color_on: bool,
    /// Turn colorization off always
    #[clap(short = 'n')]
    color_off: bool,
    /// Directory you want to search
    #[clap(value_name = "DIR", default_value = ".")]
    dir: String,
    /// List only those files matching <`include_pattern`>
    #[clap(short = 'P')]
    include_pattern: Vec<String>,
    /// Exclude any files matching <`exclude_pattern`>
    #[clap(short = 'I')]
    exclude_pattern: Vec<String>,
    /// Descend only <level> directories deep
    #[clap(short = 'L', long = "level", default_value_t = usize::max_value())]
    max_level: usize,
    /// List directories only
    #[clap(short = 'd', default_value = "false")]
    only_dirs: bool,
}

impl TryFrom<&Args> for Config {
    type Error = anyhow::Error;

    fn try_from(value: &Args) -> Result<Self, Self::Error> {
        let mut include_globs = Vec::with_capacity(value.include_pattern.len());

        for pattern in &value.include_pattern {
            let glob = Glob::new(pattern).context("Invalid include_pattern")?;
            include_globs.push(glob.compile_matcher());
        }

        let mut exclude_globs = Vec::with_capacity(value.exclude_pattern.len());

        for pattern in &value.exclude_pattern {
            let glob = Glob::new(pattern).context("Invalid exclude_pattern")?;
            exclude_globs.push(glob.compile_matcher());
        }

        let use_color = if value.color_on {
            true
        } else if value.color_off {
            false
        } else {
            // Default: enable color only if stdout is a TTY
            io::stdout().is_terminal()
        };

        Ok(Config {
            use_color,
            show_hidden: value.show_all,
            show_only_dirs: value.only_dirs,
            max_level: value.max_level,
            include_globs: Arc::from(include_globs),
            exclude_globs: Arc::from(exclude_globs),
        })
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::try_from(&args)?;

    let path = Path::new(args.dir.as_str());

    let mut term = TerminfoTerminal::new(io::stdout())
        .ok_or_else(|| anyhow::anyhow!("Could not find colored terminal"))?;
    let summary = {
        let mut p = TreePrinter::new(config, &mut term);
        p.iterate_folders(path)
            .context("Failed to iterate folders")?
    };

    if args.only_dirs {
        writeln!(&mut term, "\n{} directories", summary.num_folders)
    } else {
        writeln!(
            &mut term,
            "\n{} directories, {} files",
            summary.num_folders, summary.num_files
        )
    }
    .context("Failed to print summary")?;

    Ok(())
}
