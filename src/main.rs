extern crate term;
extern crate clap;

use term::color;

use std::fs::{self, DirEntry};
use std::path::Path;
use std::io;
use std::io::{Stdout, BufWriter};

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

fn line_prefix<'a>(levels: &mut Vec<bool>, prefix_buffer: &'a mut String) -> &'a mut String {
    let len = levels.len();
    let index = len.saturating_sub(1);
    // factor = 4, because in each iteration pushes at least 3 chars in if/else plus one in the
    // for{} block, plus 4 in the last if{} in this function
    let mut prefix = prefix_buffer;
    // we are reusing the prefix buffer from a previous iteration, so clear it
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

    prefix
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

fn iterate_folders(path: &Path,
                   levels: &mut Vec<bool>,
                   t: &mut TerminalType,
                   config: &Config,
                   prefix_buffer: &mut String)
                   -> io::Result<DirEntrySummary> {
    let mut summary = DirEntrySummary::new();
    let file_name = path_to_str(path);
    if !config.show_hidden && levels.len() > 0 && is_hidden(file_name) {
        return Ok(summary);
    }

    // store path metadata to avoid many syscalls
    let path_metadata = path.symlink_metadata()?;

    let is_dir = path_metadata.is_dir();

    // Do not count root folder in summary
    if levels.len() > 0 {
        if is_dir {
            summary.num_folders += 1;
        } else {
            summary.num_files += 1;
        }
    }

    let mut prefix_buffer = line_prefix(levels, prefix_buffer);

    if path_metadata.file_type().is_symlink() {
        if let Ok(link_path) = fs::read_link(path) {
            write!(t, "{}", &prefix_buffer)?;
            write_color(t, config, color::BRIGHT_CYAN, file_name)?;
            write!(t, " -> ")?;
            let link_file_name = format!("{}", link_path.display());
            
            // BUG: Currently prints all symlinks as executable, since the
            // metadata is for the symlink itself. Need to find a way to get new
            // metadata from the symlink. path.metadata()? will sometimes return
            // Err, as may calling link_path.metadata()?;
            print_path(&link_file_name, &path_metadata, t, config)?;
            writeln!(t, "")?;

            return Ok(summary);
        }
    }

    write!(t, "{}", &prefix_buffer)?;
    print_path(file_name, &path_metadata, t, config)?;
    writeln!(t, "")?;

    if levels.len() >= config.max_level {
        return Ok(summary);
    }

    if is_dir {
        let dir_entries = get_sorted_dir_entries(path);
        if let Err(err) = dir_entries {
            let error_msg = format!("Could not read directory '{}': {}\n", path.display(), err);
            write_color(t, config, color::RED, &error_msg)?;
            return Ok(summary);
        }

        let dir_entries = dir_entries.unwrap();

        levels.push(true);
        let len_entries = dir_entries.len();
        for entry in dir_entries.iter().take(len_entries.saturating_sub(1)) {
            let sub_summary =
                iterate_folders(&entry.path(), levels, t, config, &mut prefix_buffer)?;
            summary.add(&sub_summary);
        }

        levels.pop();
        levels.push(false);
        if let Some(entry) = dir_entries.last() {
            let sub_summary =
                iterate_folders(&entry.path(), levels, t, config, &mut prefix_buffer)?;
            summary.add(&sub_summary);
        }
        levels.pop();
    }

    Ok(summary)
}

struct Config {
    use_color: bool,
    show_hidden: bool,
    max_level: usize,
}

type TerminalType = term::Terminal<Output = BufWriter<Stdout>>;

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
        .arg(Arg::with_name("level")
            .short("L")
            .long("level")
            .takes_value(true)
            .validator(|s| to_int(&s).map(|_| ()))
            .help("Descend only level directories deep"))
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
        max_level: max_level,
    };

    let path = Path::new(matches.value_of("DIR").unwrap_or("."));

    let mut vec: Vec<bool> = Vec::new();
    let stdout_writer = BufWriter::new(io::stdout());
    let mut t = term::terminfo::TerminfoTerminal::new(stdout_writer).unwrap();
    let mut prefix_buffer = String::with_capacity(10);
    let summary = iterate_folders(&path, &mut vec, &mut t, &config, &mut prefix_buffer)
        .expect("Program failed");

    let my_term: &mut TerminalType = &mut t;
    writeln!(my_term,
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
