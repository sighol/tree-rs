use clap::ArgAction;
use clap::Parser;

use std::io;
use std::path::Path;
use std::{fs::Metadata, path::PathBuf};

use globset::{Glob, GlobMatcher};
use term::color;

mod filter;
mod pathiterator;

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

fn write_color(
    t: &mut TerminalType,
    config: &Config,
    color: color::Color,
    str: &str,
) -> io::Result<()> {
    if config.use_color {
        t.fg(color)?;
    }

    write!(t, "{}", str)?;

    if config.use_color {
        t.reset()?;
    }

    Ok(())
}

fn print_path(
    file_name: &str,
    metadata: &Metadata,
    t: &mut TerminalType,
    config: &Config,
) -> io::Result<()> {
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
        TreePrinter { config, term }
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

    fn get_iterator(&self, path: &Path) -> filter::FilteredIterator {
        let config = pathiterator::FileIteratorConfig {
            include_glob: self.config.include_glob.clone(),
            max_level: self.config.max_level,
            show_hidden: self.config.show_hidden,
        };

        let list = pathiterator::FileIterator::new(path, config);
        let mut list = filter::FilteredIterator::new(list);
        if self.config.include_glob.is_none() {
            list.skip_filter();
        }

        list
    }

    fn iterate_folders(&mut self, path: &Path) -> io::Result<DirEntrySummary> {
        let mut summary = DirEntrySummary::new();

        let mut levels: Vec<bool> = Vec::new();
        let mut prefix = String::new();

        for entry in self.get_iterator(path) {
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

        println!();

        Ok(())
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[arg(short, long, help = "Show hidden files")]
    show_all: bool,

    #[arg(short = 'C', help = "Turn colorization on always")]
    color_on: bool,

    #[arg(short='n', help="Turn colorization off always", action=ArgAction::SetTrue)]
    color_off: Option<bool>,

    #[arg(help = "Directory you want to search", default_value = ".")]
    dir: Option<PathBuf>,

    #[arg(short = 'P', help = "List only those files matching INCLUDE_PATTERN>")]
    include_pattern: Option<String>,

    #[arg(short, long, help = "Descend only LEVEL directories deep")]
    level: Option<usize>,
}

impl Cli {
    fn use_color(&self) -> bool {
        self.color_on || !self.color_off.is_some_and(|x| x)
    }
}

fn main() {
    let cli = Cli::parse();
    println!("cli: {cli:#?}");

    let max_level = cli.level.unwrap_or(usize::max_value());

    let config = Config {
        use_color: cli.use_color(),
        show_hidden: cli.show_all,
        include_glob: cli.include_pattern.map(|pattern| {
            Glob::new(&pattern)
                .expect("include_pattern is not valid")
                .compile_matcher()
        }),
        max_level,
    };

    let path = cli.dir.unwrap_or(PathBuf::from("."));

    let mut term = get_terminal_printer();
    let summary = {
        let mut p = TreePrinter::new(config, &mut term);
        p.iterate_folders(&path).expect("Program failed")
    };

    writeln!(
        &mut term,
        "\n{} directories, {} files",
        summary.num_folders, summary.num_files
    )
    .expect("Failed to print summary");
}

#[cfg(test)]
mod tests {

    use std::fs::create_dir_all;
    use std::process::Command;

    const PATH: &'static str = "target/release/tree-rs";

    fn run_cmd(arg: &[&str]) -> String {
        let stdout = Command::new(PATH)
            .args(arg)
            .output()
            .expect("command failed")
            .stdout;
        let stdout_str = String::from_utf8(stdout).expect("Bad parsing");
        stdout_str
    }

    #[test]
    fn test_normal() {
        create_dir_all("tests/simple/yyy/k").unwrap();
        let expected = r#"simple
└── yyy
    ├── k
    ├── s
    │   ├── a
    │   └── t
    ├── test.txt
    └── zz
        └── a
            └── b
                └── c

6 directories, 4 files
"#;

        let output = run_cmd(&["-n", "tests/simple"]);
        assert_eq!(expected, output);
    }

    #[test]
    fn test_max_depth() {
        create_dir_all("tests/simple/yyy/k").unwrap();
        let expected = r#"simple
└── yyy
    ├── k
    ├── s
    ├── test.txt
    └── zz

4 directories, 1 files
"#;

        let output = run_cmd(&["-n", "-L2", "tests/simple"]);
        assert_eq!(expected, output);
    }

    #[test]
    fn test_filter_txt_files() {
        let expected = r#"simple
└── yyy
    ├── test.txt

1 directories, 1 files
"#;

        let output = run_cmd(&["-n", "-P", "*.txt", "tests/simple"]);
        assert_eq!(expected, output);
    }
}
