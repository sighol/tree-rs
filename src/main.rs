extern crate term;
extern crate clap;


#[allow(dead_code)]
enum DirSign {
    Cross,
    Vert,
    Horz,
    LastFile,
}
#[allow(dead_code)]
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
        .unwrap_or("")
}

fn order_dir_entry(a: &DirEntry, b: &DirEntry) -> Ordering {
    let a_path = a.path();
    let a_name = path_to_str(&a_path);

    let b_path = b.path();
    let b_name = path_to_str(&b_path);
    let a_dir = a.path().is_dir();
    let b_dir = b.path().is_dir();
    if a_dir == b_dir {
        a_name.cmp(b_name)
    } else if a_dir {
        Ordering::Greater
    } else {
        Ordering::Less
    }
}

fn get_sorted_dir_entries(path: &Path) -> Vec<DirEntry> {
    let dir_entries = fs::read_dir(path)
        .unwrap()
        .filter_map(|e| e.ok());

    let mut dir_entries: Vec<DirEntry> = dir_entries.collect();
    dir_entries.sort_by(order_dir_entry);

    dir_entries
}

fn print_path(path: &Path, file_name: &str, levels: &mut Vec<bool>) {
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

    println!("{}{}", spaces, file_name);
}

fn iterate_folders(path: &Path, levels: &mut Vec<bool>) {
    let file_name: &str = path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(".");

    print_path(&path, file_name, levels);
    if path.is_dir() {
        if file_name.starts_with(".") && file_name.len() > 1 {
            return;
        }

        let dir_entries = get_sorted_dir_entries(path);

        levels.push(true);
        let len_entries = dir_entries.len();
        for entry in dir_entries.iter().take(if len_entries > 0 { len_entries - 1 } else { 0 }) {
            let path = entry.path();
            iterate_folders(&path, levels);
        }

        levels.pop();
        levels.push(false);
        if let Some(entry) = dir_entries.last() {
            let path = entry.path();
            iterate_folders(&path, levels);
        }
        levels.pop();
    }
}

fn main() {
    let path = Path::new(".");

    let mut vec: Vec<bool> = Vec::new();
    iterate_folders(&path, &mut vec);

    let mut t = term::stdout().unwrap();

    t.fg(color::BLUE).unwrap();
    write!(t, "hello, ").unwrap();

    t.fg(color::BRIGHT_BLUE).unwrap();
    write!(t, "world").unwrap();
    t.reset().unwrap();
    write!(t, " And reset").unwrap();
}

#[allow(dead_code)]
fn get_name(entry: &DirEntry) -> Option<String> {
    let path = entry.path();
    let file_name = path.file_name();

    match file_name {
        None => None,
        Some(file_name) => {
            match file_name.to_str() {
                None => None,
                Some(str) => Some(String::from(str)),
            }
        }
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
