extern crate term;
extern crate clap;
extern crate globset;

mod pathiterator;

use globset::{Glob, GlobMatcher};

use term::color;

use std::fs::Metadata;
use std::path::Path;
use std::io;

use clap::{Arg, App};

mod dirsign {
    pub const HORZ: char = '─';
    pub const CROSS: char = '├';
    pub const VERT: char = '│';
    pub const LAST_FILE: char = '└';
    pub const BLANK: char = '\u{00A0}';
}

fn set_line_prefix(levels: &[bool], prefix: &mut String) {
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
              metadata: &Metadata,
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
}

#[cfg(not(target_os = "linux"))]
fn is_executable(metadata: &Metadata) -> bool {
    false
}

#[cfg(target_os = "linux")]
fn is_executable(metadata: &Metadata) -> bool {
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

struct TreePrinter<'a> {
    term: &'a mut TerminalType,
    config: Config,
}

impl<'a> TreePrinter<'a> {
    fn new(config: Config, term: &'a mut TerminalType) -> TreePrinter<'a> {
        TreePrinter {
            config: config,
            term: term,
        }
    }

    fn update_levels(&self, levels: &mut Vec<bool>, level: usize, is_last: bool) {
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

    fn iterate_folders(&mut self, path: &Path) -> io::Result<DirEntrySummary> {
        let mut summary = DirEntrySummary::new();

        let config = pathiterator::FileIteratorConfig {
            include_glob: self.config.include_glob.clone(),
            max_level: self.config.max_level,
            show_hidden: self.config.show_hidden,
        };

        let mut levels: Vec<bool> = Vec::new();
        let mut prefix = String::new();

        for entry in pathiterator::FileIterator::new(path, config) {
            self.update_levels(&mut levels, entry.level, entry.is_last);

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
        print!("{}", prefix);
        if let Ok(ref metadata) = entry.metadata {
            print_path(&entry.file_name, metadata, self.term, &self.config)?;
        } else {
            print!("{} [Error]", entry.file_name);
        }

        println!("");

        Ok(())
    }
}

fn to_int(v: &str) -> Result<usize, String> {
    use std::str::FromStr;

    FromStr::from_str(v).map_err(|e| format!("Could not parse '{}' as an integer: {}", &v, e))
}

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name("show_all")
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
        to_int(level).expect("Should have validated that this value was int...")
    } else {
        usize::max_value()
    };

    let config = Config {
        use_color: use_color,
        show_hidden: matches.is_present("show_all"),
        include_glob: if let Some(pattern) = matches.value_of("include_pattern") {
            Some(Glob::new(pattern).expect("include_pattern is not valid").compile_matcher())
        } else {
            None
        },
        max_level: max_level,
    };

    let path = Path::new(matches.value_of("DIR").unwrap_or("."));

    let mut term = get_terminal_printer();
    let summary = {
        let mut p = TreePrinter::new(config, &mut term);
        p.iterate_folders(path).expect("Program failed")
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
