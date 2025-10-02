//! Tree visualization and colored output formatting.
//!
//! Handles the visual representation of directory trees using Unicode box-drawing
//! characters and terminal colors. Supports:
//! - Colored output (directories in blue, executables in green)
//! - Unicode tree structure characters (├─└│)
//! - Hierarchical indentation
//! - Summary statistics (file/directory counts)

#![deny(clippy::pedantic)]
#![deny(clippy::all)]

use std::fs::Metadata;
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;

use term::{color, Terminal};

use crate::config::Config;
use crate::pathiterator;

mod dirsign {
    pub const HORZ: char = '─';
    pub const CROSS: char = '├';
    pub const VERT: char = '│';
    pub const LAST_FILE: char = '└';
    pub const BLANK: char = '\u{00A0}';
}

/// Calculates the indent level in a tree and prints
/// the correct sign to indicate the hierarchy
fn set_line_prefix(levels: &[bool], prefix: &mut String) {
    let len = levels.len();
    let index = len.saturating_sub(1);

    prefix.clear();

    levels.iter().take(index).for_each(|level| {
        if *level {
            prefix.push(dirsign::VERT);
            prefix.push(dirsign::BLANK);
            prefix.push(dirsign::BLANK);
        } else {
            prefix.push(' ');
            prefix.push(' ');
            prefix.push(' ');
        }

        prefix.push(' ');
    });

    if let Some(last) = levels.last() {
        if *last {
            prefix.push(dirsign::CROSS);
        } else {
            prefix.push(dirsign::LAST_FILE);
        }

        prefix.push(dirsign::HORZ);
        prefix.push(dirsign::HORZ);
        prefix.push(' ');
    }
}

fn write_color<T: Write>(
    t: &mut impl Terminal<Output = T>,
    config: &Config,
    color: color::Color,
    str: &str,
) -> io::Result<()> {
    if config.use_color {
        t.fg(color)?;
    }

    write!(t, "{str}")?;

    if config.use_color {
        t.reset()?;
    }

    Ok(())
}

fn print_path<T: Write>(
    file_name: &str,
    metadata: &Metadata,
    t: &mut impl Terminal<Output = T>,
    config: &Config,
) -> io::Result<()> {
    if metadata.is_dir() {
        write_color(t, config, color::BRIGHT_BLUE, file_name)
    } else if is_executable(metadata) {
        write_color(t, config, color::BRIGHT_GREEN, file_name)
    } else {
        write!(t, "{file_name}")
    }
}

pub struct DirEntrySummary {
    pub num_folders: usize,
    pub num_files: usize,
}

impl DirEntrySummary {
    fn new() -> DirEntrySummary {
        DirEntrySummary {
            num_folders: 0,
            num_files: 0,
        }
    }
}

#[cfg(not(unix))]
fn is_executable(_metadata: &Metadata) -> bool {
    false
}

#[cfg(unix)]
fn is_executable(metadata: &Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode();
    (mode & 0o100) != 0
}

pub struct TreePrinter<'a, T, W>
where
    W: Write,
    T: Terminal<Output = W>,
{
    term: &'a mut T,
    config: Config,
}

impl<'a, T: Terminal<Output = W>, W: std::io::Write> TreePrinter<'a, T, W> {
    pub fn new(config: Config, term: &'a mut T) -> TreePrinter<'a, T, W> {
        TreePrinter { term, config }
    }

    fn update_levels(levels: &mut Vec<bool>, level: usize, is_last: bool) {
        while levels.len() > level {
            levels.pop();
        }

        if level > levels.len() {
            levels.push(!is_last);
        }

        let levels_len = levels.len();
        if levels_len > 0 {
            levels[levels_len.saturating_sub(1)] = !is_last;
        }
    }

    fn get_iterator(&self, path: &Path) -> pathiterator::FileIterator {
        let config = pathiterator::FileIteratorConfig {
            include_globs: Arc::clone(&self.config.include_globs),
            exclude_globs: Arc::clone(&self.config.exclude_globs),
            max_level: self.config.max_level,
            show_hidden: self.config.show_hidden,
            show_only_dirs: self.config.show_only_dirs,
        };

        pathiterator::FileIterator::new(path, config)
    }

    /// # Errors
    ///
    /// Will return an error if printing to the terminal fails.
    pub fn iterate_folders(&mut self, path: &Path) -> io::Result<DirEntrySummary> {
        let mut summary = DirEntrySummary::new();

        let mut levels: Vec<bool> = Vec::new();
        let mut prefix = String::new();

        for entry in self.get_iterator(path) {
            Self::update_levels(&mut levels, entry.level, entry.is_last);

            // Don't count the root directory (level 0)
            if entry.level > 0 {
                if entry.is_dir() {
                    summary.num_folders += 1;
                } else {
                    summary.num_files += 1;
                }
            }

            set_line_prefix(&levels, &mut prefix);
            self.print_line(&entry, &prefix)?;
        }

        Ok(summary)
    }

    fn print_line(&mut self, entry: &pathiterator::IteratorItem, prefix: &str) -> io::Result<()> {
        write!(self.term, "{prefix}")?;
        if let Ok(ref metadata) = entry.metadata {
            print_path(&entry.file_name, metadata, self.term, &self.config)?;
        } else if let Err(ref e) = entry.metadata {
            eprintln!("{} [Error: {}]", entry.file_name, e);
        }

        writeln!(self.term)?;

        Ok(())
    }
}
