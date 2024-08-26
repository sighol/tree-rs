mod test_terminal;

use std::fs::create_dir_all;
use std::path::Path;

use globset::Glob;
use test_terminal::TestTerminal;
use tree_rs::{Config, DirEntrySummary, TreePrinter};

fn run_cmd(path: &Path, config: Config) -> (String, DirEntrySummary) {
    let mut writer = TestTerminal::new();
    let mut p = TreePrinter::new(config, &mut writer);
    let summary = p
        .iterate_folders(path)
        .map_err(|e| format!("Program failed with error: {e}"))
        .unwrap();

    return (writer.try_into().unwrap(), summary);
}

#[test]
fn test_normal() {
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
"#;

    let (output, summary) = run_cmd(Path::new("tests/simple"), Config::default());
    assert_eq!(6, summary.num_folders);
    assert_eq!(4, summary.num_files);
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
"#;

    let (output, summary) = run_cmd(
        Path::new("tests/simple"),
        Config {
            max_level: 2,
            ..Default::default()
        },
    );
    assert_eq!(4, summary.num_folders);
    assert_eq!(1, summary.num_files);
    assert_eq!(expected, output);
}

#[test]
fn test_filter_txt_files() {
    let expected = r#"simple
└── yyy
    ├── test.txt
"#;

    let (output, summary) = run_cmd(
        Path::new("tests/simple"),
        Config {
            include_glob: Some(Glob::new("*.txt").unwrap().compile_matcher()),
            ..Default::default()
        },
    );
    assert_eq!(1, summary.num_folders);
    assert_eq!(1, summary.num_files);

    assert_eq!(expected, output);
}
