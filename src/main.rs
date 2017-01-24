extern crate walkdir;
extern crate clap;
extern crate termcolor;

use std::io;
use std::io::Write;

use walkdir::{WalkDir, DirEntry};
use termcolor::{Color, ColorChoice, ColorSpec, Stdout, WriteColor};

fn main() {
    let walker = WalkDir::new(".");
    let walker = walker.sort_by(|x, y| x.cmp(y));
    let walker = walker.into_iter().filter_map(|e| e.ok());
    for entry in walker {
        let mut spaces = String::new();
        for _ in 0..entry.depth() * 2 {
            spaces.push('-')
        }

        if let Some(name) = get_name(&entry) {
            print!("{}", spaces);
            let mut color = None;
            if let Ok(metadata) = entry.path().metadata() {
                if metadata.is_dir() {
                    color = Some(Color::Blue);
                }
            }

            write(color, &name);
            println!("");
        }
    }
}

fn write(color: Option<Color>, text: &str) -> io::Result<()> {
    let mut stdout = Stdout::new(ColorChoice::Always);
    try!(stdout.set_color(ColorSpec::new().set_fg(color)));
    write!(&mut stdout, "{}", text)
}

fn get_name(entry: &DirEntry) -> Option<String> {
    let file_name = entry.path().file_name();

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
