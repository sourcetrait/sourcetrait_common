#[cfg(test)]
mod tests {
    use std::{io, path::PathBuf};

    use srctrait_common_tooling as tooling;
    use tooling::path::diff as pathdiff;
    use srctrait_common_testing::prelude::*;

    static TESTING: testing::Module = testing::module!(Integration, {
        .using_fixture_dir()
    });

    #[tested]
    fn test_two_files() {
        let test = testing::test!({
            .using_fixture_dir()
        });

        // "This is some text."
        let first_same_filepath = test.fixture_dir().join("first-same.txt");
        // "This is some text."
        let second_same_filepath = test.fixture_dir().join("second-same.txt");
        // "This is some text."
        let first_different_filepath = test.fixture_dir().join("first-different.txt");
        // "This is some different text."
        let second_different_filepath = test.fixture_dir().join("second-different.txt");
        // "a"
        let a_filepath = test.fixture_dir().join("a.txt");
        // "b"
        let b_filepath = test.fixture_dir().join("b.txt");
        // dir with .gitignore in it
        let dir_path = test.fixture_dir().join("dir");

        assert!(!tooling::paths_differ(&first_same_filepath, &second_same_filepath).unwrap());
        assert!(tooling::paths_differ(&first_different_filepath, &second_different_filepath).unwrap());

        let expected_differences = Some(vec![
            pathdiff::Difference::FileDiffers(second_different_filepath.clone())
        ]);

        assert_eq!(expected_differences, tooling::path_diff(&first_different_filepath, &second_different_filepath).unwrap());

        // do the same with a file that shares the same size, but different content
        let expected_differences = Some(vec![
            pathdiff::Difference::FileDiffers(b_filepath.clone())
        ]);

        assert_eq!(expected_differences, tooling::path_diff(&a_filepath, &b_filepath).unwrap());

        // error testing for file not found
        let err = tooling::paths_differ(&first_same_filepath, &test.fixture_dir().join("noexist"));
        matches!(err, Err(e) if e.kind() == io::ErrorKind::NotFound);
        let err = tooling::paths_differ(&test.fixture_dir().join("noexist"), &first_same_filepath);
        matches!(err, Err(e) if e.kind() == io::ErrorKind::NotFound);

        // test first as a dir and second as a file
        let expected_differences = Some(vec![
            pathdiff::Difference::SubjectTypesDiffer
        ]);

        assert_eq!(expected_differences, tooling::path_diff(&dir_path, &b_filepath).unwrap());

        // test first as a file and second as a dir
        let expected_differences = Some(vec![
            pathdiff::Difference::SubjectTypesDiffer
        ]);

        assert_eq!(expected_differences, tooling::path_diff(&b_filepath, &dir_path).unwrap());
    }

    #[tested]
    fn test_two_dirs() {
        let test = testing::test!({
            .using_fixture_dir()
        });

        let module_fixture_dir = test.module().fixture_dir();
        assert!(!tooling::paths_differ(module_fixture_dir, module_fixture_dir).unwrap());
        assert!(tooling::paths_differ(module_fixture_dir, test.fixture_dir()).unwrap());

        let expected_diffs = Some(Vec::from([
            pathdiff::Difference::DirectoryMissing(
                PathBuf::from("1"),
                pathdiff::Subject::Second,
            ),
            pathdiff::Difference::DirectoryMissing(
                PathBuf::from("3/alpha"),
                pathdiff::Subject::Second,
            ),
            pathdiff::Difference::FileDiffers(
                PathBuf::from("3/different.txt"),
            ),
            pathdiff::Difference::FileMissing(
                PathBuf::from("3/two"),
                pathdiff::Subject::Second,
            ),
            pathdiff::Difference::TypesDiffer(
                PathBuf::from("3/typedir"),
            ),
            pathdiff::Difference::DirectoryMissing(
                PathBuf::from("2"),
                pathdiff::Subject::First,
            ),
            pathdiff::Difference::DirectoryMissing(
                PathBuf::from("3/bravo"),
                pathdiff::Subject::First,
            ),
            pathdiff::Difference::FileMissing(
                PathBuf::from("3/three"),
                pathdiff::Subject::First,
            ),
        ]));

        let diffs = tooling::path_diff(test.fixture_dir().join("a"), test.fixture_dir().join("other/b")).unwrap();
        assert_eq!(expected_diffs, diffs);

        let expected_diffs = Some(Vec::from([
            pathdiff::Difference::DirectoryMissing(
                PathBuf::from("2"),
                pathdiff::Subject::Second,
            ),
            pathdiff::Difference::DirectoryMissing(
                PathBuf::from("3/bravo"),
                pathdiff::Subject::Second,
            ),
            pathdiff::Difference::FileDiffers(
                PathBuf::from("3/different.txt"),
            ),
            pathdiff::Difference::FileMissing(
                PathBuf::from("3/three"),
                pathdiff::Subject::Second,
            ),
            pathdiff::Difference::TypesDiffer(
                PathBuf::from("3/typedir"),
            ),
            pathdiff::Difference::DirectoryMissing(
                PathBuf::from("1"),
                pathdiff::Subject::First,
            ),
            pathdiff::Difference::DirectoryMissing(
                PathBuf::from("3/alpha"),
                pathdiff::Subject::First,
            ),
            pathdiff::Difference::FileMissing(
                PathBuf::from("3/two"),
                pathdiff::Subject::First,
            ),
        ]));

        // swap positions with last and run again
        let diffs = tooling::path_diff(test.fixture_dir().join("other/b"), test.fixture_dir().join("a")).unwrap();
        assert_eq!(expected_diffs, diffs);
    }
}
