extern crate term;
extern crate clap;


enum DirSign {
    Cross,
    Vert,
    Horz,
    LastFile,
}

impl DirSign {
    fn char(self) -> char {
        return match self {
            DirSign::Horz => '─',
            DirSign::Cross => '├',
            DirSign::Vert => '│',
            DirSign::LastFile => '└',
        };
    }
}

use term::color;

use std::fs::{self, DirEntry};
use std::path::Path;

use std::cmp::Ordering;

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

fn get_sorted_dir_entries(path: &Path) -> Vec<DirEntry> {
    let dir_entries = fs::read_dir(path)
        .unwrap()
        .filter_map(|e| e.ok());

    let mut dir_entries: Vec<DirEntry> = dir_entries.collect();
    dir_entries.sort_by(order_dir_entry);

    dir_entries
}

fn print_path(path: &Path, file_name: &str, levels: &mut Vec<bool>, t: &mut Box<term::StdoutTerminal>) {
    let mut spaces = String::new();
    let len = levels.len();
    let index = if len > 0 { len - 1 } else { 0 };
    for level in levels.iter().take(index) {
        if *level {
            spaces.push(DirSign::Vert.char());
        } else {
            spaces.push(' ')
        }
        for _ in 0..3 {
            spaces.push(' ')
        }
    }

    if let Some(last) = levels.last() {
        if *last {
            spaces.push(DirSign::Cross.char());
        } else {
            spaces.push(DirSign::LastFile.char());
        }

        spaces.push(DirSign::Horz.char());
        spaces.push(DirSign::Horz.char());
        spaces.push(' ')
    }

    write!(t, "{}", spaces).unwrap();
    if path.is_dir() {
        t.fg(color::BRIGHT_BLUE).unwrap();
    }
    write!(t, "{}", file_name).unwrap();
    t.reset().unwrap();
    println!("");
}

fn is_hidden(file_name: &str) -> bool {
    file_name.starts_with(".") && file_name.len() > 1
}

fn iterate_folders(path: &Path, levels: &mut Vec<bool>, t: &mut Box<term::StdoutTerminal>) {
    let file_name = path_to_str(path);
    print_path(&path, file_name, levels, t);
    if path.is_dir() {
        if is_hidden(file_name) {
            return;
        }

        let dir_entries = get_sorted_dir_entries(path);

        levels.push(true);
        let len_entries = dir_entries.len();
        for entry in dir_entries.iter().take(if len_entries > 0 { len_entries - 1 } else { 0 }) {
            let path = entry.path();
            iterate_folders(&path, levels, t);
        }

        levels.pop();
        levels.push(false);
        if let Some(entry) = dir_entries.last() {
            let path = entry.path();
            iterate_folders(&path, levels, t);
        }
        levels.pop();
    }
}

fn main() {
    let path = Path::new(".");

    let mut vec: Vec<bool> = Vec::new();

    let mut t = term::stdout().unwrap();
    iterate_folders(&path, &mut vec, &mut t);
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
