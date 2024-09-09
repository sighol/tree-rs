#![deny(clippy::pedantic)]
#![deny(clippy::all)]

mod filter;
mod pathiterator;
mod tree_printer;

#[cfg(test)]
mod tests;

use clap::Parser;
use tree_printer::{Config, TreePrinter};

use std::error::Error;
use std::io::{self, Write};
use std::path::Path;

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
    type Error = String;

    fn try_from(value: &Args) -> Result<Self, Self::Error> {
        let mut include_globs = Vec::with_capacity(value.include_pattern.len());

        for pattern in &value.include_pattern {
            let glob =
                Glob::new(pattern).map_err(|e| format!("`include_pattern` is not valid: {e}"))?;
            include_globs.push(glob.compile_matcher());
        }

        let mut exlude_globs = Vec::with_capacity(value.exclude_pattern.len());

        for pattern in &value.exclude_pattern {
            let glob =
                Glob::new(pattern).map_err(|e| format!("`exclude_pattern` is not valid: {e}"))?;
            exlude_globs.push(glob.compile_matcher());
        }

        Ok(Config {
            use_color: value.color_on || !value.color_off,
            show_hidden: value.show_all,
            show_only_dirs: value.only_dirs,
            max_level: value.max_level,
            include_globs,
            exlude_globs,
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let config = Config::try_from(&args)?;

    let path = Path::new(args.dir.as_str());

    let mut term = TerminfoTerminal::new(io::stdout())
        .ok_or(anyhow::anyhow!("Could not find colored terminal"))?;
    let summary = {
        let mut p = TreePrinter::new(config, &mut term);
        p.iterate_folders(path)
            .map_err(|e| format!("Program failed with error: {e}"))?
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
    .map_err(|e| format!("Failed to print summary: {e}"))?;

    Ok(())
}
