use flecstract::tar::*;
use std::path::{Path, PathBuf};

const TEST_PATH: &str = "/tmp/flecstract/test/";

#[test]
fn extract_test() {
    let test_path = Path::new(TEST_PATH).join("extracted/");
    let _ = std::fs::remove_dir_all(&test_path);
    assert!(!test_path.try_exists().unwrap());
    std::fs::create_dir_all(&test_path).unwrap();
    let archive = include_bytes!("archives/test_archive.tar");
    test_sym_links(archive.as_slice(), &test_path)
}

fn test_sym_links(archive: &[u8], test_path: &Path) {
    extract(archive, test_path).unwrap();
    // Check level 0
    assert_eq!(std::fs::read_dir(test_path).unwrap().count(), 5);
    assert!(
        std::fs::metadata(test_path.join("level_1"))
            .unwrap()
            .is_dir()
    );
    assert!(
        std::fs::symlink_metadata(test_path.join("link_0_to_2_0"))
            .unwrap()
            .is_symlink()
    );
    assert_eq!(
        std::fs::read_link(test_path.join("link_0_to_2_0")).unwrap(),
        Path::new("level_1/level_2/file_0")
    );
    assert!(
        std::fs::symlink_metadata(test_path.join("link_1_to_1_1"))
            .unwrap()
            .is_symlink()
    );
    assert_eq!(
        std::fs::read_link(test_path.join("link_1_to_1_1")).unwrap(),
        Path::new("level_1/file_1")
    );
    assert_eq!(
        std::fs::read_to_string(test_path.join("file_0")).unwrap(),
        "level 0 file 0\n"
    );
    assert_eq!(
        std::fs::read_to_string(test_path.join("file_1")).unwrap(),
        "level 0 file 1\n"
    );
    // Check level 1
    assert_eq!(
        std::fs::read_dir(test_path.join("level_1"))
            .unwrap()
            .count(),
        3
    );
    assert!(
        std::fs::metadata(test_path.join("level_1/level_2"))
            .unwrap()
            .is_dir()
    );
    assert_eq!(
        std::fs::read_to_string(test_path.join("level_1/file_0")).unwrap(),
        "level 1 file 0\n"
    );
    assert_eq!(
        std::fs::read_to_string(test_path.join("level_1/file_1")).unwrap(),
        "level 1 file 1\n"
    );
    // Check level 2
    assert_eq!(
        std::fs::read_dir(test_path.join("level_1/level_2"))
            .unwrap()
            .count(),
        3
    );
    assert!(
        std::fs::metadata(test_path.join("level_1/level_2/level_3"))
            .unwrap()
            .is_dir()
    );
    assert_eq!(
        std::fs::read_to_string(test_path.join("level_1/level_2/file_0")).unwrap(),
        "level 2 file 0\n"
    );
    assert_eq!(
        std::fs::read_to_string(test_path.join("level_1/level_2/file_1")).unwrap(),
        "level 2 file 1\n"
    );
    // Check level 3
    assert_eq!(
        std::fs::read_dir(test_path.join("level_1/level_2/level_3"))
            .unwrap()
            .count(),
        4
    );
    assert_eq!(
        std::fs::read_to_string(test_path.join("level_1/level_2/level_3/file_0")).unwrap(),
        "level 3 file 0\n"
    );
    assert_eq!(
        std::fs::read_to_string(test_path.join("level_1/level_2/level_3/file_1")).unwrap(),
        "level 3 file 1\n"
    );
    assert!(
        std::fs::symlink_metadata(test_path.join("level_1/level_2/level_3/link_0_to_0_0"))
            .unwrap()
            .is_symlink()
    );
    assert_eq!(
        std::fs::read_link(test_path.join("level_1/level_2/level_3/link_0_to_0_0")).unwrap(),
        Path::new("../../../file_0")
    );

    assert!(
        std::fs::symlink_metadata(test_path.join("level_1/level_2/level_3/link_1_to_2_0"))
            .unwrap()
            .is_symlink()
    );
    assert_eq!(
        std::fs::read_link(test_path.join("level_1/level_2/level_3/link_1_to_2_0")).unwrap(),
        Path::new("../file_0")
    );
}

fn test_hard_links(archive: &[u8], test_path: &Path) {
    extract(archive, test_path).unwrap();
    // Check level 0
    assert_eq!(std::fs::read_dir(test_path).unwrap().count(), 5);
    assert!(
        std::fs::metadata(test_path.join("level_1"))
            .unwrap()
            .is_dir()
    );
    assert!(
        !std::fs::symlink_metadata(test_path.join("link_0_to_2_0"))
            .unwrap()
            .is_symlink()
    );
    assert_eq!(
        std::fs::read_to_string(test_path.join("link_0_to_2_0")).unwrap(),
        "level 2 file 0\n"
    );
    assert!(
        !std::fs::symlink_metadata(test_path.join("link_1_to_1_1"))
            .unwrap()
            .is_symlink()
    );
    assert_eq!(
        std::fs::read_to_string(test_path.join("link_1_to_1_1")).unwrap(),
        "level 1 file 1\n"
    );
    assert_eq!(
        std::fs::read_to_string(test_path.join("file_0")).unwrap(),
        "level 0 file 0\n"
    );
    assert_eq!(
        std::fs::read_to_string(test_path.join("file_1")).unwrap(),
        "level 0 file 1\n"
    );
    // Check level 1
    assert_eq!(
        std::fs::read_dir(test_path.join("level_1"))
            .unwrap()
            .count(),
        3
    );
    assert!(
        std::fs::metadata(test_path.join("level_1/level_2"))
            .unwrap()
            .is_dir()
    );
    assert_eq!(
        std::fs::read_to_string(test_path.join("level_1/file_0")).unwrap(),
        "level 1 file 0\n"
    );
    assert_eq!(
        std::fs::read_to_string(test_path.join("level_1/file_1")).unwrap(),
        "level 1 file 1\n"
    );
    // Check level 2
    assert_eq!(
        std::fs::read_dir(test_path.join("level_1/level_2"))
            .unwrap()
            .count(),
        3
    );
    assert!(
        std::fs::metadata(test_path.join("level_1/level_2/level_3"))
            .unwrap()
            .is_dir()
    );
    assert_eq!(
        std::fs::read_to_string(test_path.join("level_1/level_2/file_0")).unwrap(),
        "level 2 file 0\n"
    );
    assert_eq!(
        std::fs::read_to_string(test_path.join("level_1/level_2/file_1")).unwrap(),
        "level 2 file 1\n"
    );
    // Check level 3
    assert_eq!(
        std::fs::read_dir(test_path.join("level_1/level_2/level_3"))
            .unwrap()
            .count(),
        4
    );
    assert_eq!(
        std::fs::read_to_string(test_path.join("level_1/level_2/level_3/file_0")).unwrap(),
        "level 3 file 0\n"
    );
    assert_eq!(
        std::fs::read_to_string(test_path.join("level_1/level_2/level_3/file_1")).unwrap(),
        "level 3 file 1\n"
    );
    assert!(
        !std::fs::symlink_metadata(test_path.join("level_1/level_2/level_3/link_0_to_0_0"))
            .unwrap()
            .is_symlink()
    );
    assert_eq!(
        std::fs::read_to_string(test_path.join("level_1/level_2/level_3/link_0_to_0_0")).unwrap(),
        "level 0 file 0\n"
    );

    assert!(
        !std::fs::symlink_metadata(test_path.join("level_1/level_2/level_3/link_1_to_2_0"))
            .unwrap()
            .is_symlink()
    );
    assert_eq!(
        std::fs::read_to_string(test_path.join("level_1/level_2/level_3/link_1_to_2_0")).unwrap(),
        "level 2 file 0\n"
    );
}

#[test]
fn extract_hard_links() {
    let test_path = Path::new(TEST_PATH).join("extracted_hard/");
    let _ = std::fs::remove_dir_all(&test_path);
    assert!(!test_path.try_exists().unwrap());
    std::fs::create_dir_all(&test_path).unwrap();
    let archive = include_bytes!("archives/test_archive_follow_links.tar");
    test_hard_links(archive, &test_path);
}

#[test]
fn archive_follow_symlinks() {
    let src_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/archives/test_directory");
    let extract_path = Path::new(TEST_PATH).join("archive_follow_links/");
    let _ = std::fs::remove_dir_all(&extract_path);
    assert!(!extract_path.try_exists().unwrap());
    std::fs::create_dir_all(&extract_path).unwrap();
    let buffer = Vec::<u8>::default();
    let data = archive(src_path, buffer, true).unwrap();
    test_hard_links(&data, &extract_path);
}

#[test]
fn archive_no_follow_symlinks() {
    let src_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/archives/test_directory");
    let extract_path = Path::new(TEST_PATH).join("archive_no_follow_links/");
    let _ = std::fs::remove_dir_all(&extract_path);
    assert!(!extract_path.try_exists().unwrap());
    std::fs::create_dir_all(&extract_path).unwrap();
    let buffer = Vec::<u8>::default();
    let data = archive(src_path, buffer, false).unwrap();
    test_sym_links(&data, &extract_path);
}

#[test]
fn extract_single_file() {
    let extract_path = Path::new(TEST_PATH).join("extract_single_file/");
    let _ = std::fs::remove_dir_all(&extract_path);
    assert!(!extract_path.try_exists().unwrap());
    std::fs::create_dir_all(&extract_path).unwrap();
    let archive = include_bytes!("archives/single_test_file.tar");
    extract(archive.as_slice(), &extract_path).unwrap();
    assert_eq!(std::fs::read_dir(&extract_path).unwrap().count(), 1);
    assert_eq!(
        std::fs::read_to_string(extract_path.join("single_test_file")).unwrap(),
        "Single Test File\n"
    );
    assert!(
        !std::fs::symlink_metadata(extract_path.join("single_test_file"))
            .unwrap()
            .is_symlink()
    );
}
#[test]
fn archive_single_file() {
    let src_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/archives/single_test_file");
    let extract_path = Path::new(TEST_PATH).join("archive_single_file/");
    let _ = std::fs::remove_dir_all(&extract_path);
    assert!(!extract_path.try_exists().unwrap());
    std::fs::create_dir_all(&extract_path).unwrap();
    let buffer = Vec::<u8>::default();
    let data = archive(src_path, buffer, false).unwrap();
    extract(data.as_slice(), &extract_path).unwrap();
    assert_eq!(std::fs::read_dir(&extract_path).unwrap().count(), 1);
    assert_eq!(
        std::fs::read_to_string(extract_path.join("single_test_file")).unwrap(),
        "Single Test File\n"
    );
    assert!(
        !std::fs::symlink_metadata(extract_path.join("single_test_file"))
            .unwrap()
            .is_symlink()
    );
}
