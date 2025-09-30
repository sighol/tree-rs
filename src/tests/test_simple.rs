use std::fs::create_dir_all;
use std::path::Path;
use std::sync::Arc;

use crate::tests::utils::TestTerminal;
use crate::{tree_printer::DirEntrySummary, Config, TreePrinter};
use globset::Glob;

fn run_cmd(path: &Path, config: Config) -> (String, DirEntrySummary) {
    let mut writer = TestTerminal::new();
    let mut p = TreePrinter::new(config, &mut writer);
    let summary = p
        .iterate_folders(path)
        .map_err(|e| format!("Program failed with error: {e}"))
        .unwrap();

    (writer.try_into().unwrap(), summary)
}

#[test]
fn test_normal() {
    let expected = r"simple
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
";

    let (output, summary) = run_cmd(Path::new("tests/simple"), Config::default());
    assert_eq!(6, summary.num_folders);
    assert_eq!(4, summary.num_files);
    assert_eq!(expected, output);
}

#[test]
fn test_max_depth() {
    create_dir_all("tests/simple/yyy/k").unwrap();
    let expected = r"simple
└── yyy
    ├── k
    ├── s
    ├── test.txt
    └── zz
";

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
    let expected = r"simple
└── yyy
    ├── k
    ├── s
    ├── test.txt
    └── zz
        └── a
            └── b
";

    let (output, summary) = run_cmd(
        Path::new("tests/simple"),
        Config {
            include_globs: Arc::from(vec![Glob::new("*.txt").unwrap().compile_matcher()]),
            ..Default::default()
        },
    );

    assert_eq!(6, summary.num_folders);
    assert_eq!(1, summary.num_files);

    assert_eq!(expected, output);
}

#[test]
fn test_exclude_txt_files() {
    let expected = r"simple
└── yyy
    ├── k
    ├── s
    │   ├── a
    │   └── t
    └── zz
        └── a
            └── b
                └── c
";

    let (output, summary) = run_cmd(
        Path::new("tests/simple"),
        Config {
            exclude_globs: Arc::from(vec![Glob::new("*.txt").unwrap().compile_matcher()]),
            ..Default::default()
        },
    );

    assert_eq!(6, summary.num_folders);
    assert_eq!(3, summary.num_files);
    assert_eq!(expected, output);
}

#[test]
fn test_only_directories() {
    let expected = r"simple
└── yyy
    ├── k
    ├── s
    └── zz
        └── a
            └── b
";
    let (output, summary) = run_cmd(
        Path::new("tests/simple"),
        Config {
            show_only_dirs: true,
            ..Default::default()
        },
    );

    assert_eq!(expected, output);
    assert_eq!(6, summary.num_folders);
    assert_eq!(0, summary.num_files);
}
