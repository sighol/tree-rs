use std::path::Path;

use crate::pathiterator::{path_to_str, FileIterator, FileIteratorConfig};
use std::sync::Arc;

#[test]
fn test_path_to_str_with_root_path() {
    // Test with a path that has no filename (e.g., "/")
    let path = Path::new("/");
    let result = path_to_str(path);
    // Should fallback to path.to_str() when no filename exists
    assert!(!result.is_empty());
}

#[test]
fn test_path_to_str_with_normal_path() {
    let path = Path::new("/some/path/file.txt");
    let result = path_to_str(path);
    assert_eq!(result, "file.txt");
}

#[test]
fn test_iterator_with_empty_directory() {
    use std::fs;

    // Create an empty directory
    let empty_dir = "tests/empty_test_dir";
    fs::create_dir_all(empty_dir).unwrap();

    let config = FileIteratorConfig {
        show_hidden: false,
        show_only_dirs: false,
        max_level: usize::MAX,
        include_globs: Arc::new([]),
        exclude_globs: Arc::new([]),
    };

    let iterator = FileIterator::new(Path::new(empty_dir), config);
    let items: Vec<_> = iterator.collect();

    // Clean up
    fs::remove_dir_all(empty_dir).unwrap();

    // Should contain only the root directory itself
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].level, 0);
}

#[test]
#[cfg(unix)]
fn test_unreadable_directory() {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    // Create a directory structure
    let test_dir = "tests/unreadable_test";
    let sub_dir = format!("{test_dir}/subdir");
    fs::create_dir_all(&sub_dir).unwrap();

    // Make subdirectory unreadable
    let mut perms = fs::metadata(&sub_dir).unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(&sub_dir, perms.clone()).unwrap();

    let config = FileIteratorConfig {
        show_hidden: false,
        show_only_dirs: false,
        max_level: usize::MAX,
        include_globs: Arc::new([]),
        exclude_globs: Arc::new([]),
    };

    let iterator = FileIterator::new(Path::new(test_dir), config);
    let items: Vec<_> = iterator.collect();

    // Restore permissions before cleanup
    perms.set_mode(0o755);
    fs::set_permissions(&sub_dir, perms).unwrap();
    fs::remove_dir_all(test_dir).unwrap();

    // Should still iterate, but handle the error gracefully
    assert!(!items.is_empty()); // At least the root
}

#[test]
fn test_iterator_max_level_zero() {
    let config = FileIteratorConfig {
        show_hidden: false,
        show_only_dirs: false,
        max_level: 0,
        include_globs: Arc::new([]),
        exclude_globs: Arc::new([]),
    };

    let iterator = FileIterator::new(Path::new("tests/simple"), config);
    let items: Vec<_> = iterator.collect();

    // Should only return root directory at level 0
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].level, 0);
}

#[test]
fn test_iterator_with_hidden_files() {
    use std::fs::{self, File};

    // Create a test directory with hidden files
    let test_dir = "tests/hidden_test_dir";
    fs::create_dir_all(test_dir).unwrap();
    File::create(format!("{test_dir}/.hidden")).unwrap();
    File::create(format!("{test_dir}/visible.txt")).unwrap();

    // Test without showing hidden files
    let config = FileIteratorConfig {
        show_hidden: false,
        show_only_dirs: false,
        max_level: usize::MAX,
        include_globs: Arc::new([]),
        exclude_globs: Arc::new([]),
    };

    let iterator = FileIterator::new(Path::new(test_dir), config);
    let items: Vec<_> = iterator.collect();
    let visible_only = items.iter().any(|item| item.file_name == ".hidden");

    // Test with showing hidden files
    let config_with_hidden = FileIteratorConfig {
        show_hidden: true,
        show_only_dirs: false,
        max_level: usize::MAX,
        include_globs: Arc::new([]),
        exclude_globs: Arc::new([]),
    };

    let iterator_with_hidden = FileIterator::new(Path::new(test_dir), config_with_hidden);
    let items_with_hidden: Vec<_> = iterator_with_hidden.collect();
    let has_hidden = items_with_hidden
        .iter()
        .any(|item| item.file_name == ".hidden");

    // Clean up
    fs::remove_dir_all(test_dir).unwrap();

    assert!(
        !visible_only,
        "Hidden file should not be visible without show_hidden"
    );
    assert!(has_hidden, "Hidden file should be visible with show_hidden");
}
