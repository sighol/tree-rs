extern crate term;
extern crate clap;

use term::color;

use std::fs::{self, DirEntry};
use std::path::Path;
use std::io;

use std::cmp::Ordering;

use clap::{Arg, App};

enum DirSign {
    Cross,
    Vert,
    Horz,
    LastFile,
    Blank,
}

impl DirSign {
    fn char(self) -> char {
        return match self {
            DirSign::Horz => '─',
            DirSign::Cross => '├',
            DirSign::Vert => '│',
            DirSign::LastFile => '└',
            DirSign::Blank => '\u{00A0}',
        };
    }
}

fn path_to_str(dir: &Path) -> &str {
    dir.file_name()
        .and_then(|x| x.to_str())
        .or_else(|| dir.to_str())
        .unwrap_or("")
}

fn order_dir_entry(a: &DirEntry, b: &DirEntry) -> Ordering {
    let a_path = a.path();
    let a_name = path_to_str(&a_path);

    let b_path = b.path();
    let b_name = path_to_str(&b_path);

    a_name.cmp(b_name)
}

fn get_sorted_dir_entries(path: &Path) -> io::Result<Vec<DirEntry>> {
    let dir_entries = fs::read_dir(path);
    match dir_entries {
        Ok(entries) => {
            let dir_entries = entries.filter_map(|e| e.ok());
            let mut dir_entries: Vec<DirEntry> = dir_entries.collect();
            dir_entries.sort_by(order_dir_entry);
            Ok(dir_entries)
        }
        Err(err) => Err(err),
    }
}

fn line_prefix(levels: &mut Vec<bool>) -> String {
    let mut prefix = String::new();
    let len = levels.len();
    let index = if len > 0 { len - 1 } else { 0 };
    for level in levels.iter().take(index) {
        if *level {
            prefix.push(DirSign::Vert.char());
            for _ in 0..2 {
                prefix.push(DirSign::Blank.char())
            }
        } else {
            for _ in 0..3 {
                prefix.push(' ');
            }
        }

        prefix.push(' ');
    }

    if let Some(last) = levels.last() {
        if *last {
            prefix.push(DirSign::Cross.char());
        } else {
            prefix.push(DirSign::LastFile.char());
        }

        prefix.push(DirSign::Horz.char());
        prefix.push(DirSign::Horz.char());
        prefix.push(' ');
    }

    prefix
}

fn writeln_color(t: &mut Box<term::StdoutTerminal>, config: &Config, color: color::Color, str: &str) -> io::Result<()> {
    if config.use_color {
        t.fg(color)?;
    }

    writeln!(t, "{}", str)?;

    if config.use_color {
        t.reset()?;
    }

    Ok(())
}

fn print_path(path: &Path,
              file_name: &str,
              levels: &mut Vec<bool>,
              t: &mut Box<term::StdoutTerminal>,
              config: &Config) -> io::Result<()> {

    let prefix = line_prefix(levels);

    write!(t, "{}", prefix)?;
    if path.is_dir() {
        writeln_color(t, config, color::BRIGHT_BLUE, file_name)
    } else {
        writeln!(t, "{}", file_name)
    }
}

fn is_hidden(file_name: &str) -> bool {
    file_name.starts_with(".") && file_name != ".." && file_name != "."
}

fn iterate_folders(path: &Path,
                   levels: &mut Vec<bool>,
                   t: &mut Box<term::StdoutTerminal>,
                   config: &Config) -> io::Result<()> {
    let file_name = path_to_str(path);
    if !config.show_hidden && is_hidden(file_name) {
        return Ok(());
    }

    print_path(&path, file_name, levels, t, config)?;
    if path.is_dir() {
        let dir_entries = get_sorted_dir_entries(path);
        if let Err(err) = dir_entries {
            let error_msg = format!("Could not read directory: {}", err);
            writeln_color(t, config, color::RED, &error_msg)?;
            return Ok(());
        }

        let dir_entries = dir_entries.unwrap();

        levels.push(true);
        let len_entries = dir_entries.len();
        for entry in dir_entries.iter().take(if len_entries > 0 { len_entries - 1 } else { 0 }) {
            let path = entry.path();
            iterate_folders(&path, levels, t, config)?;
        }

        levels.pop();
        levels.push(false);
        if let Some(entry) = dir_entries.last() {
            let path = entry.path();
            iterate_folders(&path, levels, t, config)?;
        }
        levels.pop();
    }

    Ok(())
}

struct Config {
    use_color: bool,
    show_hidden: bool,
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
        .get_matches();

    let use_color = matches.is_present("color_on") || !matches.is_present("color_off");
    let config = Config {
        use_color: use_color,
        show_hidden: matches.is_present("a"),
    };

    let path = matches.value_of("DIR").unwrap_or(".");
    let path = Path::new(path);


    let mut vec: Vec<bool> = Vec::new();

    let mut t = term::stdout().unwrap();
    match iterate_folders(&path, &mut vec, &mut t, &config) {
        Ok(_) => (),
        Err(err) => writeln!(t, "Program failed: {}", err).unwrap_or(()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn path_is_file_is_dir() {
        let path = Path::new(".");
        assert!(path.is_file() == false);
        assert!(path.is_dir() == true);
    }
}
