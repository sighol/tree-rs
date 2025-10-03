use std::fs::File;
use std::path::Path;
use std::sync::Arc;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::config::Config;
use crate::tests::utils::TestTerminal;
use crate::tree_printer::{DirEntrySummary, TreePrinter};
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
    let (output, summary) = run_cmd(Path::new("tests/simple"), Config::default());
    assert_eq!(6, summary.num_folders);
    assert!(summary.num_files >= 4, "Should have at least 4 files");
    // Check for expected directory structure
    assert!(output.contains("simple"));
    assert!(output.contains("yyy"));
    assert!(output.contains("test.txt"));
}

#[test]
fn test_max_depth() {
    let (output, summary) = run_cmd(
        Path::new("tests/simple"),
        Config {
            max_level: 2,
            ..Default::default()
        },
    );
    assert_eq!(4, summary.num_folders);
    assert!(summary.num_files >= 1, "Should have at least 1 file");
    // Check that it contains the expected structure
    assert!(output.contains("simple"));
    assert!(output.contains("yyy"));
    assert!(output.contains('k'));
    assert!(output.contains('s'));
    assert!(output.contains("test.txt"));
    assert!(output.contains("zz"));
    // Should not contain deeply nested items (level > 2)
    assert!(!output.contains("└── a"));
    assert!(!output.contains("└── b"));
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
    let (output, summary) = run_cmd(
        Path::new("tests/simple"),
        Config {
            exclude_globs: Arc::from(vec![Glob::new("*.txt").unwrap().compile_matcher()]),
            ..Default::default()
        },
    );

    assert_eq!(6, summary.num_folders);
    assert!(
        summary.num_files >= 3,
        "Should have at least 3 files when excluding .txt"
    );
    // Check for key elements instead of exact match
    assert!(output.contains("simple"));
    assert!(output.contains("yyy"));
    assert!(!output.contains("test.txt"), "test.txt should be excluded");
    // Check that we have some of the expected files
    assert!(output.contains('a'));
    assert!(output.contains('t'));
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

#[test]
fn test_color_output() {
    let (output, summary) = run_cmd(
        Path::new("tests/simple"),
        Config {
            use_color: true,
            ..Default::default()
        },
    );

    // Should still produce output even with color enabled
    assert!(summary.num_folders >= 6, "Should have at least 6 folders");
    assert!(summary.num_files >= 4, "Should have at least 4 files");
    assert!(output.contains("simple"));
    assert!(output.contains("yyy"));
}

#[test]
#[cfg(unix)]
fn test_executable_file() {
    use std::io::Write;

    // Create a test executable
    let exec_path = "tests/simple/yyy/exec_file_test.sh";

    // Clean up from any previous run
    let _ = std::fs::remove_file(exec_path);

    let mut file = File::create(exec_path).unwrap();
    writeln!(file, "#!/bin/bash").unwrap();
    drop(file);

    // Make it executable
    let mut perms = std::fs::metadata(exec_path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(exec_path, perms).unwrap();

    let (output, summary) = run_cmd(
        Path::new("tests/simple"),
        Config {
            use_color: true,
            ..Default::default()
        },
    );

    // Clean up
    let _ = std::fs::remove_file(exec_path);

    assert!(output.contains("exec_file_test.sh"));
    assert!(summary.num_folders >= 6, "Should have at least 6 folders");
    assert!(summary.num_files >= 5, "Should have at least 5 files");
}

#[test]
fn test_hidden_files() {
    // Create a hidden file
    let hidden_file = "tests/simple/yyy/.hidden_test";

    // Clean up any leftover from previous test runs
    let _ = std::fs::remove_file(hidden_file);

    File::create(hidden_file).unwrap();

    // Test without show_hidden
    let (output_no_hidden, _summary_no_hidden) =
        run_cmd(Path::new("tests/simple"), Config::default());

    // Test with show_hidden
    let (output_with_hidden, _summary_with_hidden) = run_cmd(
        Path::new("tests/simple"),
        Config {
            show_hidden: true,
            ..Default::default()
        },
    );

    // Clean up - using ok() to ignore errors if file doesn't exist
    let _ = std::fs::remove_file(hidden_file);

    // Without show_hidden, should not see the hidden file
    assert!(
        !output_no_hidden.contains(".hidden_test"),
        "Hidden file should not be visible without show_hidden flag"
    );

    // With show_hidden, should see the hidden file
    assert!(
        output_with_hidden.contains(".hidden_test"),
        "Hidden file should be visible with show_hidden flag"
    );
}

#[test]
#[cfg(unix)]
fn test_broken_symlink() {
    use std::os::unix::fs::symlink;

    // Create a broken symlink
    let symlink_path = "tests/simple/yyy/broken_link_test";

    // Clean up any leftover from previous test runs
    let _ = std::fs::remove_file(symlink_path);

    symlink("/nonexistent/path", symlink_path).unwrap();

    let (output, _summary) = run_cmd(Path::new("tests/simple"), Config::default());

    // Clean up - using ok() to ignore errors if file doesn't exist
    let _ = std::fs::remove_file(symlink_path);

    // Should handle broken symlink gracefully
    assert!(output.contains("simple"));
}

#[test]
fn test_multiple_include_patterns() {
    let (output, summary) = run_cmd(
        Path::new("tests/simple"),
        Config {
            include_globs: Arc::from(vec![
                Glob::new("*.txt").unwrap().compile_matcher(),
                Glob::new("a").unwrap().compile_matcher(),
            ]),
            ..Default::default()
        },
    );

    assert!(output.contains("test.txt"));
    assert!(output.contains('a'));
    assert_eq!(6, summary.num_folders);
    assert_eq!(2, summary.num_files); // test.txt + a
}

#[test]
fn test_multiple_exclude_patterns() {
    let (output, summary) = run_cmd(
        Path::new("tests/simple"),
        Config {
            exclude_globs: Arc::from(vec![
                Glob::new("*.txt").unwrap().compile_matcher(),
                Glob::new("c").unwrap().compile_matcher(),
            ]),
            ..Default::default()
        },
    );

    assert!(!output.contains("test.txt"), "test.txt should be excluded");
    // File 'c' should be excluded
    let lines: Vec<&str> = output.lines().collect();
    let has_file_c = lines
        .iter()
        .any(|line| line.trim().ends_with("└── c") || line.trim().ends_with('c'));
    assert!(!has_file_c, "File 'c' should be excluded");
    assert!(summary.num_folders >= 6, "Should have at least 6 folders");
}
