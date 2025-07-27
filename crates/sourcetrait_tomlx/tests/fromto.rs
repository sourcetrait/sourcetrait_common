#[cfg(test)]
mod tests {
    use std::fs;

    use sourcetrait_testing::prelude::*;
    use sourcetrait_tomlx::{self as tomlx, prelude::*};
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

    impl tomlx::FromToml for PathKind {}
    impl tomlx::ToToml for PathKind {}
    impl tomlx::FromToml for PathKindDir {}
    impl tomlx::ToToml for PathKindDir {}
    impl tomlx::FromToml for PathKindFile {}
    impl tomlx::ToToml for PathKindFile {}

    #[tested]
    fn test_fromto_str() {
        let _test = testing::test!();

        let expected = expected();
println!("{}", expected.to_toml().unwrap());
        assert_eq!(expected, PathKind::from_toml(EXPECTED_TOML).unwrap());
        let actual_toml = expected.to_toml().unwrap();
        assert_eq!(EXPECTED_TOML, actual_toml);
    }

    #[tested]
    fn test_fromto_file() {
        let test = testing::test!({
            .using_fixture_dir()
            .using_temp_dir()
        });

        let expected = expected();
        let fixture_file = test.fixture_dir().join("expected.toml");
        let file_expected = PathKind::from_toml_file(&fixture_file).unwrap();
        // sanity check
        assert_eq!(expected, file_expected);

        let tmp_file = test.temp_dir().join("actual.toml");
        expected.to_toml_file(&tmp_file).unwrap();
        let file_actual = PathKind::from_toml_file(&tmp_file).unwrap();
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
                            name: "sourcetrait_tomlx".to_string(),
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

    /// how roundtrip implicit toml looks with pretty printing and options
    const EXPECTED_TOML: &'static str =
r#"[Dir]
name = "sourcetrait_common"

[[Dir.children]]

[Dir.children.Dir]
name = "packages"

[[Dir.children.Dir.children]]

[Dir.children.Dir.children.Dir]
name = "sourcetrait_tomlx"

[[Dir.children.Dir.children.Dir.children]]

[Dir.children.Dir.children.Dir.children.File]
name = "SOURCETRAIT.md"

[[Dir.children.Dir.children.Dir.children]]

[Dir.children.Dir.children.Dir.children.File]
name = "SOURCETRAIT"

[Dir.children.Dir.children.Dir.children.File.symlink.File]
name = "SOURCETRAIT.md"
"#;

}
