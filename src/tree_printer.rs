#![deny(clippy::pedantic)]
#![deny(clippy::all)]

use std::fs::Metadata;
use std::io::{self, Write};
use std::path::Path;

use globset::GlobMatcher;
use term::{color, Terminal};

use crate::{filter, pathiterator};

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

#[cfg(not(target_os = "linux"))]
fn is_executable(_metadata: &Metadata) -> bool {
    false
}

#[cfg(target_os = "linux")]
fn is_executable(metadata: &Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode();
    (mode & 0o100) != 0
}

pub struct Config {
    pub use_color: bool,
    pub show_hidden: bool,
    pub show_only_dirs: bool,
    pub max_level: usize,
    pub include_globs: Vec<GlobMatcher>,
    pub exlude_globs: Vec<GlobMatcher>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            use_color: false,
            show_hidden: false,
            show_only_dirs: false,
            max_level: usize::MAX,
            include_globs: Vec::new(),
            exlude_globs: Vec::new(),
        }
    }
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

    fn get_iterator(&self, path: &Path) -> filter::FilteredIterator {
        let config = pathiterator::FileIteratorConfig {
            include_globs: self.config.include_globs.clone(),
            exlude_globs: self.config.exlude_globs.clone(),
            max_level: self.config.max_level,
            show_hidden: self.config.show_hidden,
            show_only_dirs: self.config.show_only_dirs,
        };

        let iterator = pathiterator::FileIterator::new(path, config);
        filter::FilteredIterator::new(iterator)
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

            if entry.is_dir() {
                summary.num_folders += 1;
            } else {
                summary.num_files += 1;
            }

            set_line_prefix(&levels, &mut prefix);
            self.print_line(&entry, &prefix)?;
        }

        summary.num_folders = summary.num_folders.saturating_sub(1);

        Ok(summary)
    }

    fn print_line(&mut self, entry: &pathiterator::IteratorItem, prefix: &str) -> io::Result<()> {
        write!(self.term, "{prefix}")?;
        if let Ok(ref metadata) = entry.metadata {
            print_path(&entry.file_name, metadata, self.term, &self.config)?;
        } else {
            eprint!("{} [Error]", entry.file_name);
        }

        writeln!(self.term)?;

        Ok(())
    }
}
