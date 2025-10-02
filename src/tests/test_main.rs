use crate::config::{Args, Config};
use crate::run;
use crate::tests::utils::TestTerminal;
use globset::Glob;
use std::path::Path;
use std::sync::Arc;

// Tests for Config::try_from conversion

#[test]
fn test_config_from_args_basic() {
    use clap::Parser;

    #[allow(clippy::struct_excessive_bools)]
    #[derive(Debug, Parser)]
    struct Args {
        #[clap(short = 'a', long = "all")]
        show_all: bool,
        #[clap(short = 'C')]
        color_on: bool,
        #[clap(short = 'n')]
        color_off: bool,
        #[clap(value_name = "DIR", default_value = ".")]
        dir: String,
        #[clap(short = 'P')]
        include_pattern: Vec<String>,
        #[clap(short = 'I')]
        exclude_pattern: Vec<String>,
        #[clap(short = 'L', long = "level", default_value_t = usize::max_value())]
        max_level: usize,
        #[clap(short = 'd', default_value = "false")]
        only_dirs: bool,
    }

    let args = Args {
        show_all: true,
        color_on: false,
        color_off: false,
        dir: ".".to_string(),
        include_pattern: vec![],
        exclude_pattern: vec![],
        max_level: usize::MAX,
        only_dirs: false,
    };

    let config = Config {
        use_color: false,
        show_hidden: args.show_all,
        show_only_dirs: args.only_dirs,
        max_level: args.max_level,
        include_globs: Arc::new([]),
        exclude_globs: Arc::new([]),
    };

    assert!(config.show_hidden);
    assert!(!config.show_only_dirs);
    assert_eq!(config.max_level, usize::MAX);
}

#[test]
fn test_config_color_on() {
    let config = Config {
        use_color: true,
        show_hidden: false,
        show_only_dirs: false,
        max_level: usize::MAX,
        include_globs: Arc::new([]),
        exclude_globs: Arc::new([]),
    };

    assert!(config.use_color);
}

#[test]
fn test_config_color_off() {
    let config = Config {
        use_color: false,
        show_hidden: false,
        show_only_dirs: false,
        max_level: usize::MAX,
        include_globs: Arc::new([]),
        exclude_globs: Arc::new([]),
    };

    assert!(!config.use_color);
}

#[test]
fn test_config_with_include_patterns() {
    let include_globs = vec![
        Glob::new("*.txt").unwrap().compile_matcher(),
        Glob::new("*.md").unwrap().compile_matcher(),
    ];

    let config = Config {
        use_color: false,
        show_hidden: false,
        show_only_dirs: false,
        max_level: usize::MAX,
        include_globs: Arc::from(include_globs),
        exclude_globs: Arc::new([]),
    };

    assert_eq!(config.include_globs.len(), 2);
}

#[test]
fn test_config_with_exclude_patterns() {
    let exclude_globs = vec![
        Glob::new("*.log").unwrap().compile_matcher(),
        Glob::new("*.tmp").unwrap().compile_matcher(),
    ];

    let config = Config {
        use_color: false,
        show_hidden: false,
        show_only_dirs: false,
        max_level: usize::MAX,
        include_globs: Arc::new([]),
        exclude_globs: Arc::from(exclude_globs),
    };

    assert_eq!(config.exclude_globs.len(), 2);
}

#[test]
fn test_config_with_max_level() {
    let config = Config {
        use_color: false,
        show_hidden: false,
        show_only_dirs: false,
        max_level: 3,
        include_globs: Arc::new([]),
        exclude_globs: Arc::new([]),
    };

    assert_eq!(config.max_level, 3);
}

#[test]
fn test_config_only_dirs() {
    let config = Config {
        use_color: false,
        show_hidden: false,
        show_only_dirs: true,
        max_level: usize::MAX,
        include_globs: Arc::new([]),
        exclude_globs: Arc::new([]),
    };

    assert!(config.show_only_dirs);
}

#[test]
fn test_config_default() {
    let config = Config::default();

    assert!(!config.use_color);
    assert!(!config.show_hidden);
    assert!(!config.show_only_dirs);
    assert_eq!(config.max_level, usize::MAX);
    assert_eq!(config.include_globs.len(), 0);
    assert_eq!(config.exclude_globs.len(), 0);
}

#[test]
fn test_invalid_glob_pattern() {
    // Test that invalid glob patterns are handled properly
    let result = Glob::new("[invalid");
    assert!(result.is_err());
}

#[test]
fn test_config_all_options_enabled() {
    let config = Config {
        use_color: true,
        show_hidden: true,
        show_only_dirs: true,
        max_level: 5,
        include_globs: Arc::from(vec![Glob::new("*.rs").unwrap().compile_matcher()]),
        exclude_globs: Arc::from(vec![Glob::new("*.bak").unwrap().compile_matcher()]),
    };

    assert!(config.use_color);
    assert!(config.show_hidden);
    assert!(config.show_only_dirs);
    assert_eq!(config.max_level, 5);
    assert_eq!(config.include_globs.len(), 1);
    assert_eq!(config.exclude_globs.len(), 1);
}

// Tests for Args -> Config conversion using TryFrom

#[test]
fn test_args_to_config_basic() {
    let args = Args {
        show_all: false,
        color_on: false,
        color_off: true,
        dir: ".".to_string(),
        include_pattern: vec![],
        exclude_pattern: vec![],
        max_level: usize::MAX,
        only_dirs: false,
    };

    let config = Config::try_from(&args).unwrap();

    assert!(!config.use_color);
    assert!(!config.show_hidden);
    assert!(!config.show_only_dirs);
    assert_eq!(config.max_level, usize::MAX);
}

#[test]
fn test_args_to_config_with_color_on() {
    let args = Args {
        show_all: false,
        color_on: true,
        color_off: false,
        dir: ".".to_string(),
        include_pattern: vec![],
        exclude_pattern: vec![],
        max_level: usize::MAX,
        only_dirs: false,
    };

    let config = Config::try_from(&args).unwrap();

    assert!(config.use_color);
}

#[test]
fn test_args_to_config_with_color_off() {
    let args = Args {
        show_all: false,
        color_on: false,
        color_off: true,
        dir: ".".to_string(),
        include_pattern: vec![],
        exclude_pattern: vec![],
        max_level: usize::MAX,
        only_dirs: false,
    };

    let config = Config::try_from(&args).unwrap();

    assert!(!config.use_color);
}

#[test]
fn test_args_to_config_with_show_all() {
    let args = Args {
        show_all: true,
        color_on: false,
        color_off: true,
        dir: ".".to_string(),
        include_pattern: vec![],
        exclude_pattern: vec![],
        max_level: usize::MAX,
        only_dirs: false,
    };

    let config = Config::try_from(&args).unwrap();

    assert!(config.show_hidden);
}

#[test]
fn test_args_to_config_with_only_dirs() {
    let args = Args {
        show_all: false,
        color_on: false,
        color_off: true,
        dir: ".".to_string(),
        include_pattern: vec![],
        exclude_pattern: vec![],
        max_level: usize::MAX,
        only_dirs: true,
    };

    let config = Config::try_from(&args).unwrap();

    assert!(config.show_only_dirs);
}

#[test]
fn test_args_to_config_with_max_level() {
    let args = Args {
        show_all: false,
        color_on: false,
        color_off: true,
        dir: ".".to_string(),
        include_pattern: vec![],
        exclude_pattern: vec![],
        max_level: 3,
        only_dirs: false,
    };

    let config = Config::try_from(&args).unwrap();

    assert_eq!(config.max_level, 3);
}

#[test]
fn test_args_to_config_with_include_patterns() {
    let args = Args {
        show_all: false,
        color_on: false,
        color_off: true,
        dir: ".".to_string(),
        include_pattern: vec!["*.txt".to_string(), "*.md".to_string()],
        exclude_pattern: vec![],
        max_level: usize::MAX,
        only_dirs: false,
    };

    let config = Config::try_from(&args).unwrap();

    assert_eq!(config.include_globs.len(), 2);
}

#[test]
fn test_args_to_config_with_exclude_patterns() {
    let args = Args {
        show_all: false,
        color_on: false,
        color_off: true,
        dir: ".".to_string(),
        include_pattern: vec![],
        exclude_pattern: vec!["*.log".to_string(), "*.tmp".to_string()],
        max_level: usize::MAX,
        only_dirs: false,
    };

    let config = Config::try_from(&args).unwrap();

    assert_eq!(config.exclude_globs.len(), 2);
}

#[test]
fn test_args_to_config_with_invalid_include_pattern() {
    let args = Args {
        show_all: false,
        color_on: false,
        color_off: true,
        dir: ".".to_string(),
        include_pattern: vec!["[invalid".to_string()],
        exclude_pattern: vec![],
        max_level: usize::MAX,
        only_dirs: false,
    };

    let result = Config::try_from(&args);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("include_pattern"));
}

#[test]
fn test_args_to_config_with_invalid_exclude_pattern() {
    let args = Args {
        show_all: false,
        color_on: false,
        color_off: true,
        dir: ".".to_string(),
        include_pattern: vec![],
        exclude_pattern: vec!["[invalid".to_string()],
        max_level: usize::MAX,
        only_dirs: false,
    };

    let result = Config::try_from(&args);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("exclude_pattern"));
}

#[test]
fn test_args_to_config_all_options() {
    let args = Args {
        show_all: true,
        color_on: true,
        color_off: false,
        dir: "/some/path".to_string(),
        include_pattern: vec!["*.rs".to_string()],
        exclude_pattern: vec!["*.bak".to_string()],
        max_level: 5,
        only_dirs: true,
    };

    let config = Config::try_from(&args).unwrap();

    assert!(config.use_color);
    assert!(config.show_hidden);
    assert!(config.show_only_dirs);
    assert_eq!(config.max_level, 5);
    assert_eq!(config.include_globs.len(), 1);
    assert_eq!(config.exclude_globs.len(), 1);
}

// Tests for the run() function

#[test]
fn test_run_basic() {
    let config = Config::default();
    let mut term = TestTerminal::new();

    let result = run(config, Path::new("tests/simple"), false, &mut term);

    assert!(result.is_ok());
    let summary = result.unwrap();
    assert!(summary.num_folders >= 6);
    assert!(summary.num_files >= 4);

    let output: String = term.try_into().unwrap();
    assert!(output.contains("directories"));
    assert!(output.contains("files"));
}

#[test]
fn test_run_only_directories() {
    let config = Config {
        show_only_dirs: true,
        ..Default::default()
    };
    let mut term = TestTerminal::new();

    let result = run(config, Path::new("tests/simple"), true, &mut term);

    assert!(result.is_ok());
    let summary = result.unwrap();
    assert!(summary.num_folders >= 6);

    let output: String = term.try_into().unwrap();
    assert!(output.contains("directories"));
    assert!(!output.contains("files"));
}

#[test]
fn test_run_with_nonexistent_path() {
    let config = Config::default();
    let mut term = TestTerminal::new();

    let result = run(
        config,
        Path::new("/nonexistent/path/that/does/not/exist"),
        false,
        &mut term,
    );

    // The iterator handles errors gracefully and still produces output
    // It prints the error but doesn't fail
    assert!(result.is_ok());
    let summary = result.unwrap();
    assert_eq!(summary.num_folders, 0);
    assert_eq!(summary.num_files, 0);
}

#[test]
fn test_run_with_max_level() {
    let config = Config {
        max_level: 2,
        ..Default::default()
    };
    let mut term = TestTerminal::new();

    let result = run(config, Path::new("tests/simple"), false, &mut term);

    assert!(result.is_ok());
    let summary = result.unwrap();
    assert_eq!(summary.num_folders, 4);
}

#[test]
fn test_run_with_include_pattern() {
    let include_globs = vec![Glob::new("*.txt").unwrap().compile_matcher()];

    let config = Config {
        include_globs: Arc::from(include_globs),
        ..Default::default()
    };
    let mut term = TestTerminal::new();

    let result = run(config, Path::new("tests/simple"), false, &mut term);

    assert!(result.is_ok());
    let output: String = term.try_into().unwrap();
    assert!(output.contains("test.txt"));
}

#[test]
fn test_run_with_exclude_pattern() {
    let exclude_globs = vec![Glob::new("*.txt").unwrap().compile_matcher()];

    let config = Config {
        exclude_globs: Arc::from(exclude_globs),
        ..Default::default()
    };
    let mut term = TestTerminal::new();

    let result = run(config, Path::new("tests/simple"), false, &mut term);

    assert!(result.is_ok());
    let output: String = term.try_into().unwrap();
    assert!(!output.contains("test.txt"));
}

#[test]
fn test_run_with_hidden_files() {
    use std::fs::File;

    // Create a hidden file for testing
    let hidden_file = "tests/simple/yyy/.test_hidden";
    let _ = std::fs::remove_file(hidden_file);
    File::create(hidden_file).unwrap();

    let config = Config {
        show_hidden: true,
        ..Default::default()
    };
    let mut term = TestTerminal::new();

    let result = run(config, Path::new("tests/simple"), false, &mut term);

    // Clean up
    let _ = std::fs::remove_file(hidden_file);

    assert!(result.is_ok());
    let output: String = term.try_into().unwrap();
    assert!(output.contains(".test_hidden"));
}
