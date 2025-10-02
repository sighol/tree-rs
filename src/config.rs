//! Configuration management for tree-rs
//!
//! This module handles CLI argument parsing and configuration setup,
//! including glob pattern compilation and color detection.

use anyhow::{Context, Result};
use clap::Parser;
use globset::{Glob, GlobMatcher};
use std::io::{self, IsTerminal};
use std::sync::Arc;

/// Command-line arguments for tree-rs
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Parser)]
pub struct Args {
    /// Show hidden files
    #[clap(short = 'a', long = "all")]
    pub show_all: bool,

    /// Turn colorization on always
    #[clap(short = 'C')]
    pub color_on: bool,

    /// Turn colorization off always
    #[clap(short = 'n')]
    pub color_off: bool,

    /// Directory you want to search
    #[clap(value_name = "DIR", default_value = ".")]
    pub dir: String,

    /// List only those files matching <`include_pattern`>
    #[clap(short = 'P')]
    pub include_pattern: Vec<String>,

    /// Exclude any files matching <`exclude_pattern`>
    #[clap(short = 'I')]
    pub exclude_pattern: Vec<String>,

    /// Descend only <level> directories deep
    #[clap(short = 'L', long = "level", default_value_t = usize::max_value())]
    pub max_level: usize,

    /// List directories only
    #[clap(short = 'd', default_value = "false")]
    pub only_dirs: bool,
}

/// Configuration for tree traversal and display
#[derive(Debug)]
pub struct Config {
    pub use_color: bool,
    pub show_hidden: bool,
    pub show_only_dirs: bool,
    pub max_level: usize,
    pub include_globs: Arc<[GlobMatcher]>,
    pub exclude_globs: Arc<[GlobMatcher]>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            use_color: false,
            show_hidden: false,
            show_only_dirs: false,
            max_level: usize::MAX,
            include_globs: Arc::new([]),
            exclude_globs: Arc::new([]),
        }
    }
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
