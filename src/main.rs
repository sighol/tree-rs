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
    include_pattern: Option<String>,
    /// Descend only <level> directories deep
    #[clap(short = 'L', long = "level", default_value_t = usize::max_value())]
    max_level: usize,
}

impl TryFrom<&Args> for Config {
    type Error = String;

    fn try_from(value: &Args) -> Result<Self, Self::Error> {
        let include_glob = value
            .include_pattern
            .as_deref()
            .map(Glob::new)
            .transpose()
            .map_err(|e| format!("`include_pattern` is not valid: {e}"))?
            .map(|glob| glob.compile_matcher());

        Ok(Config {
            use_color: value.color_on || !value.color_off,
            show_hidden: value.show_all,
            max_level: value.max_level,
            include_glob,
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

    writeln!(
        &mut term,
        "\n{} directories, {} files",
        summary.num_folders, summary.num_files
    )
    .map_err(|e| format!("Failed to print summary: {e}"))?;

    Ok(())
}
