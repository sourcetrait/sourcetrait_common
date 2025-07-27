#[cfg(test)]
mod tests {
    use std::fs;

    use sourcetrait_testing::prelude::*;
    use sourcetrait_ronx::{self as ronx, prelude::*};
    use serde;

    static TESTING: testing::Module = testing::module!(Integration, {
        .using_fixture_dir()
        .using_temp_dir()
    });

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    enum PathKind {
        Dir(PathKindDir),
        File(PathKindFile)
    }

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    #[serde(rename = "Dir")]
    struct PathKindDir {
        name: String,
        children: Vec<PathKind>
    }

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    #[serde(rename = "File")]
    struct PathKindFile {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        symlink: Option<Box<PathKind>>
    }

    impl ronx::FromRon for PathKind {}
    impl ronx::ToRon for PathKind {}
    impl ronx::FromRon for PathKindDir {}
    impl ronx::ToRon for PathKindDir {}
    impl ronx::FromRon for PathKindFile {}
    impl ronx::ToRon for PathKindFile {}

    #[tested]
    fn test_fromto_str() {
        let _test = testing::test!();

        let expected = expected();

        assert_eq!(expected, PathKind::from_ron(EXPECTED_RON).unwrap());
        let actual_ron = expected.to_ron().unwrap();
        assert_eq!(EXPECTED_RON, actual_ron);
    }

    #[tested]
    fn test_fromto_file() {
        let test = testing::test!({
            .using_fixture_dir()
            .using_temp_dir()
        });

        let expected = expected();
        let fixture_file = test.fixture_dir().join("expected.ron");
        let file_expected = PathKind::from_ron_file(&fixture_file).unwrap();
        // sanity check
        assert_eq!(expected, file_expected);

        let tmp_file = test.temp_dir().join("actual.ron");
        expected.to_ron_file(&tmp_file).unwrap();
        let file_actual = PathKind::from_ron_file(&tmp_file).unwrap();
        assert_eq!(expected, file_actual);
        assert_eq!(fs::read_to_string(fixture_file).unwrap(), fs::read_to_string(tmp_file).unwrap());
    }

    fn expected() -> PathKind {
        PathKind::Dir(PathKindDir {
            name: "sourcetrait_common".to_string(),
            children: Vec::from([
                PathKind::Dir(PathKindDir{
                    name: "packages".to_string(),
                    children: Vec::from([
                        PathKind::Dir(PathKindDir {
                            name: "sourcetrait_ronx".to_string(),
                            children: Vec::from([
                                PathKind::File(PathKindFile {
                                    name: "SOURCETRAIT.md".to_string(),
                                    symlink: None
                                }),
                                PathKind::File(PathKindFile {
                                    name: "SOURCETRAIT".to_string(),
                                    symlink: Some(Box::new(PathKind::File(PathKindFile {
                                        name: "SOURCETRAIT.md".to_string(),
                                        symlink: None
                                    })))
                                }),
                            ])
                        })
                    ])
                })
            ])
        })
    }

    /// how roundtrip implicit ron looks with pretty printing and options
    const EXPECTED_RON: &'static str =
r#"Dir(
    name: "sourcetrait_common",
    children: [
        Dir(
            name: "packages",
            children: [
                Dir(
                    name: "sourcetrait_ronx",
                    children: [
                        File(
                            name: "SOURCETRAIT.md",
                        ),
                        File(
                            name: "SOURCETRAIT",
                            symlink: File(
                                name: "SOURCETRAIT.md",
                            ),
                        ),
                    ],
                ),
            ],
        ),
    ],
)"#;

}
