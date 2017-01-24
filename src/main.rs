extern crate walkdir;
extern crate clap;
extern crate termcolor;

use std::io;
use std::io::Write;

use walkdir::{WalkDir, DirEntry, WalkDirIterator};
use termcolor::{Color, ColorChoice, ColorSpec, Stdout, WriteColor};

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
            DirSign::Vert => '|',
            DirSign::LastFile => '└',
        };
    }
}

/// ├── _config.yml
/// ├── _drafts
/// |   ├── begin-with-the-crazy-ideas.textile
/// |   └── on-simplicity-in-technology.markdown
/// ├── _includes
/// |   ├── footer.html
/// |   └── header.html
/// ├── _layouts
/// |   ├── default.html
/// |   └── post.html
/// ├── _posts
/// |   ├── 2007-10-29-why-every-programmer-should-play-nethack.textile
/// |   └── 2009-04-26-barcamp-boston-4-roundup.textile
/// ├── _data
/// |   └── members.yml
/// ├── _site
/// └── index.html

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn main() {
    let mut stdout = Stdout::new(ColorChoice::Always);

    let walker = WalkDir::new(".");
    let walker = walker.sort_by(|x, y| x.cmp(y));
    let walker = walker.into_iter();
        // .filter_entry(|e| !is_hidden(e));
        // .filter_map(|e| e.ok());



    for entry in walker {
        if entry.is_err() { continue; }

        let entry = entry.unwrap();

        let mut prefix = String::new();
        for _ in 0..entry.depth() {
            prefix.push_str("|   ")
        }

        if let Some(name) = get_name(&entry) {
            let mut color = None;
            if let Ok(metadata) = entry.path().metadata() {
                if metadata.is_dir() {
                    color = Some(Color::Magenta);
                }
            }

            prefix.push(DirSign::Cross.char());
            prefix.push(DirSign::Horz.char());
            prefix.push(DirSign::Horz.char());
            print!("{} ", prefix);
            write(&mut stdout, color, &name);
            println!("");
        }
    }
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
