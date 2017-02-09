extern crate term;
extern crate clap;
extern crate globset;

mod pathiterator;

use globset::{Glob, GlobMatcher};

use term::color;

use std::collections::VecDeque;

use std::fs::{self, DirEntry, Metadata};
use std::path::{Path, PathBuf};
use std::io;

use std::cmp::Ordering;

use clap::{Arg, App};

mod dirsign {
    pub const HORZ: char = '─';
    pub const CROSS: char = '├';
    pub const VERT: char = '│';
    pub const LAST_FILE: char = '└';
    pub const BLANK: char = '\u{00A0}';
}

fn path_to_str(dir: &Path) -> &str {
    dir.file_name()
        .and_then(|x| x.to_str())
        .or_else(|| dir.to_str())
        .unwrap_or("")
}

fn order_dir_entry(a: &DirEntry, b: &DirEntry) -> Ordering {
    a.file_name().cmp(&b.file_name())
}

fn get_sorted_dir_entries(path: &Path) -> io::Result<Vec<DirEntry>> {
    fs::read_dir(path).map(|entries| {
        let mut dir_entries: Vec<DirEntry> = entries.filter_map(Result::ok).collect();
        dir_entries.sort_by(order_dir_entry);
        dir_entries
    })
}

fn set_line_prefix(levels: &Vec<bool>, prefix: &mut String) {
    let len = levels.len();
    let index = len.saturating_sub(1);

    prefix.clear();
    for level in levels.iter().take(index) {
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
    }

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

fn write_color(t: &mut TerminalType,
               config: &Config,
               color: color::Color,
               str: &str)
               -> io::Result<()> {
    if config.use_color {
        t.fg(color)?;
    }

    write!(t, "{}", str)?;

    if config.use_color {
        t.reset()?;
    }

    Ok(())
}

fn print_path(file_name: &str,
              metadata: &fs::Metadata,
              t: &mut TerminalType,
              config: &Config)
              -> io::Result<()> {
    if metadata.is_dir() {
        write_color(t, config, color::BRIGHT_BLUE, file_name)
    } else if is_executable(metadata) {
        write_color(t, config, color::BRIGHT_GREEN, file_name)
    } else {
        write!(t, "{}", file_name)
    }
}

fn is_hidden(file_name: &str) -> bool {
    file_name.starts_with(".")
}

struct DirEntrySummary {
    num_folders: usize,
    num_files: usize,
}

impl DirEntrySummary {
    fn new() -> DirEntrySummary {
        DirEntrySummary {
            num_folders: 0,
            num_files: 0,
        }
    }

    fn add(&mut self, other: &DirEntrySummary) {
        self.num_files += other.num_files;
        self.num_folders += other.num_folders;
    }
}

#[cfg(not(target_os = "linux"))]
fn is_executable(metadata: &fs::Metadata) -> bool {
    false
}

#[cfg(target_os = "linux")]
fn is_executable(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode();
    (mode & 0o100) != 0
}

struct Config {
    use_color: bool,
    show_hidden: bool,
    max_level: usize,
    include_glob: Option<GlobMatcher>,
}

type TerminalType = Box<term::StdoutTerminal>;

fn get_terminal_printer() -> TerminalType {
    term::stdout().expect("Could not unwrap term::stdout.")
}

struct TreePrinterCacheItem {
    metadata: Metadata,
    levels: Vec<bool>,
    path: PathBuf,
}

impl TreePrinterCacheItem {
    fn new(path: PathBuf, metadata: Metadata, levels: Vec<bool>) -> TreePrinterCacheItem {
        TreePrinterCacheItem {
            path: path,
            metadata: metadata,
            levels: levels,
        }
    }
}

struct TreePrinterCache {
    items: VecDeque<TreePrinterCacheItem>,
}

impl TreePrinterCache {
    fn new() -> TreePrinterCache {
        TreePrinterCache { items: VecDeque::new() }
    }
}

struct TreePrinter<'a> {
    term: &'a mut TerminalType,
    config: Config,
    prefix_buffer: String,
    levels: Vec<bool>,
    cache: TreePrinterCache,
}

impl<'a> TreePrinter<'a> {
    fn new(config: Config, term: &'a mut TerminalType) -> TreePrinter<'a> {
        TreePrinter {
            config: config,
            term: term,
            prefix_buffer: String::new(),
            levels: Vec::new(),
            cache: TreePrinterCache::new(),
        }
    }

    fn iterate_folders(&mut self, path: &Path) -> io::Result<DirEntrySummary> {
        let mut summary = DirEntrySummary::new();
        let file_name = path_to_str(path);
        if !self.config.show_hidden && self.levels.len() > 0 && is_hidden(file_name) {
            return Ok(summary);
        }

        // store path metadata to avoid many syscalls
        let path_metadata = path.symlink_metadata()?;

        let is_dir = path_metadata.is_dir();

        set_line_prefix(&self.levels, &mut self.prefix_buffer);

        if self.levels.len() >= self.config.max_level {
            return Ok(summary);
        }

        loop {
            let should_pop_front = if let Some(last_cache) = self.cache.items.back() {
                last_cache.levels.len() >= self.levels.len()
            } else {
                false
            };

            if should_pop_front {
                self.cache.items.pop_back();
            } else {
                break;
            }
        }

        if !is_dir {
            let file_is_included = if let Some(ref include_glob) = self.config.include_glob {
                include_glob.is_match(file_name)
            } else {
                true
            };

            if file_is_included {
                while let Some(item) = self.cache.items.pop_front() {
                    let mut prefix = String::new();
                    set_line_prefix(&item.levels, &mut prefix);
                    write!(self.term, "{}", &prefix)?;
                    let file_name = path_to_str(&item.path);
                    print_path(file_name, &item.metadata, self.term, &self.config)?;
                    writeln!(self.term, "")?;
                    // Do not count root folder in summary
                    if item.levels.len() > 0 {
                        summary.num_folders += 1;
                    }
                }

                summary.num_files += 1;

                if path_metadata.file_type().is_symlink() {
                    if let Ok(link_path) = fs::read_link(path) {
                        write!(self.term, "{}", &self.prefix_buffer)?;
                        write_color(self.term, &self.config, color::BRIGHT_CYAN, file_name)?;
                        write!(self.term, " -> ")?;
                        let link_file_name = format!("{}", link_path.display());

                        // BUG: Currently prints all symlinks as executable, since the
                        // metadata is for the symlink itself. Need to find a way to get new
                        // metadata from the symlink. path.metadata()? will sometimes return
                        // Err, as may calling link_path.metadata()?;
                        print_path(&link_file_name, &path_metadata, self.term, &self.config)?;
                        writeln!(self.term, "")?;

                        return Ok(summary);
                    }
                }

                write!(self.term, "{}", &self.prefix_buffer)?;
                print_path(file_name, &path_metadata, self.term, &self.config)?;
                writeln!(self.term, "")?;
            }
        } else {
            if self.config.include_glob.is_some() {
                let item =
                    TreePrinterCacheItem::new(path.to_owned(), path_metadata, self.levels.clone());
                self.cache.items.push_back(item);
            } else {
                write!(self.term, "{}", &self.prefix_buffer)?;
                print_path(file_name, &path_metadata, self.term, &self.config)?;
                writeln!(self.term, "")?;

                if self.levels.len() > 0 {
                    summary.num_folders += 1;
                }
            }

            let dir_entries = get_sorted_dir_entries(path);
            if let Err(err) = dir_entries {
                let error_msg = format!("Could not read directory '{}': {}\n", path.display(), err);
                write_color(self.term, &self.config, color::RED, &error_msg)?;
                return Ok(summary);
            }

            let dir_entries = dir_entries.unwrap();

            self.levels.push(true);
            let len_entries = dir_entries.len();
            for entry in dir_entries.iter().take(len_entries.saturating_sub(1)) {
                let sub_summary = self.iterate_folders(&entry.path())?;
                summary.add(&sub_summary);
            }

            self.levels.pop();
            self.levels.push(false);
            if let Some(entry) = dir_entries.last() {
                let sub_summary = self.iterate_folders(&entry.path())?;
                summary.add(&sub_summary);
            }
            self.levels.pop();

        }

        Ok(summary)
    }
}

fn to_int(v: &str) -> Result<usize, String> {
    use std::str::FromStr;

    FromStr::from_str(v).map_err(|e| format!("Could not parse '{}' as an integer: {}", &v, e))
}

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name("a")
            .short("a")
            .long("all")
            .help("Show hidden files"))
        .arg(Arg::with_name("color_on")
            .short("C")
            .help("Turn colorization on always"))
        .arg(Arg::with_name("color_off")
            .short("n")
            .help("Turn colorization off always"))
        .arg(Arg::with_name("DIR")
            .index(1)
            .help("Directory you want to search"))
        .arg(Arg::with_name("include_pattern")
            .short("P")
            .takes_value(true)
            .help("List only those files matching <include_pattern>"))
        .arg(Arg::with_name("level")
            .short("L")
            .long("level")
            .takes_value(true)
            .validator(|s| to_int(&s).map(|_| ()))
            .help("Descend only <level> directories deep"))
        .get_matches();

    let use_color = matches.is_present("color_on") || !matches.is_present("color_off");

    let max_level = if let Some(level) = matches.value_of("level") {
        to_int(&level).expect("Should have validated that this value was int...")
    } else {
        usize::max_value()
    };



    let config = Config {
        use_color: use_color,
        show_hidden: matches.is_present("a"),
        include_glob: if let Some(pattern) = matches.value_of("include_pattern") {
            Some(Glob::new(pattern).expect("include_pattern is not valid").compile_matcher())
        } else {
            None
        },
        max_level: max_level,
    };

    let path = Path::new(matches.value_of("DIR").unwrap_or("."));

    let c = pathiterator::FileIteratorConfig {
        show_hidden: matches.is_present("a"),
        include_glob: if let Some(pattern) = matches.value_of("include_pattern") {
            Some(Glob::new(pattern).expect("include_pattern is not valid").compile_matcher())
        } else {
            None
        },
        max_level: max_level,
    };

    let itr = pathiterator::iterate(&path, c);

    for item in itr {
        println!("{}", item.to_string());
    }
    return;

    let mut term = get_terminal_printer();
    let summary = {
        let mut p = TreePrinter::new(config, &mut term);
        p.iterate_folders(&path).expect("Program failed")
    };

    writeln!(&mut term,
             "\n{} directories, {} files",
             summary.num_folders,
             summary.num_files)
        .expect("Failed to print summary");
}


#[cfg(test)]
mod tests {
    use std::path::Path;
    use super::*;

    #[test]
    fn test_is_hidden() {
        assert!(true == is_hidden(".git"));
        assert!(false == is_hidden("file"));
    }

    #[test]
    fn path_is_file_is_dir() {
        let path = Path::new(".");
        assert!(path.is_file() == false);
        assert!(path.is_dir() == true);
    }
}
