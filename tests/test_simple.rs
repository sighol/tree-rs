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
