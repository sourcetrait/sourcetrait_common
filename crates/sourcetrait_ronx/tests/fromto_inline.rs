#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::{fs, path::PathBuf};
    use serde;
    use sourcetrait_ronx_macro::RonX;
    use sourcetrait_testing::prelude::*;
    use sourcetrait_ronx::{prelude::*};
    
    static TESTING: testing::Module = testing::module!(Integration, {
        .using_fixture_dir()
        .using_temp_dir()
    });
    
    static GROUP_ABC: testing::Group = testing::group!(
        "fromto_inline/pyramid/valid/abc",
        Integration, {
            .using_fixture_dir()
        }
    );
    
    static GROUP_ABCDEFG: testing::Group = testing::group!(
        "fromto_inline/pyramid/valid/abcdefg",
        Integration, {
            .using_fixture_dir()
        }
    );
    
    static GROUP_ABCDEFG_EC: testing::Group = testing::group!(
        "fromto_inline/pyramid/valid/abcdefg_ec",
        Integration, {
            .using_fixture_dir()
        }
    );
    
    static GROUP_ABCDEFG_EC_GE: testing::Group = testing::group!(
        "fromto_inline/pyramid/invalid/abcdefg_ec_ge",
        Integration, {
            .using_fixture_dir()
        }
    );
    
    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, RonX)]
    #[ronx(inlined)]
    #[serde(untagged)]
    enum PathKind {
        #[ronx(inlined)]
        Dir(PathKindDir),
        #[ronx(inlined)]
        File(PathKindFile),
        #[ronx(inlined)]
        Symlink(PathKindSymlink),
    }

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, RonX)]
    #[serde(rename = "Dir")]
    #[ronx(inlined)]
    struct PathKindDir {
        name: String,
        #[ronx(inlined)]
        children: Vec<InlinedRon<PathKind>>
    }

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, RonX)]
    #[serde(rename = "File")]
    #[ronx(inlined)]
    struct PathKindFile {
        name: String,
    }
    
    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, RonX)]
    #[serde(rename = "Symlink")]
    #[ronx(inlined)]
    struct PathKindSymlink {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        symlink: Option<Box<PathKind>>
    }
    
    #[tested]
    fn test_abc() {
        let _test = testing::test!();
        
        let include_dir = GROUP_ABC.fixture_dir();
        let resolved_expected = abc_resolved_expected();
        let ron_file = include_dir.join("abc.ron");
        
        let ron_str_expected = fs::read_to_string(&ron_file).unwrap();
        let inline_config = InlinedRonConfig::new(include_dir.into());
        
        let resolved_actual = PathKind::from_inlined_ron_file(&ron_file, &inline_config).unwrap();
        assert_eq!(resolved_expected, resolved_actual);
        
        let ron_str_reserialized = resolved_actual.to_ron().unwrap();
        assert_eq!(ron_str_expected, ron_str_reserialized);
        
        let reparsed = PathKind::from_inlined_ron(&ron_str_reserialized, &inline_config).unwrap();
        assert_eq!(resolved_expected, reparsed);
    }
    
    #[tested]
    fn test_abcdefg() {
        let _test = testing::test!();
        
        let include_dir = GROUP_ABCDEFG.fixture_dir();
        let resolved_expected = abcdefg_resolved_expected();
        let ron_file = include_dir.join("abcdefg.ron");
        
        let ron_str_expected = fs::read_to_string(&ron_file).unwrap();
        let inline_config = InlinedRonConfig::new(include_dir.into());
        
        let resolved_actual = PathKind::from_inlined_ron_file(&ron_file, &inline_config).unwrap();
        assert_eq!(resolved_expected, resolved_actual);
        
        let ron_str_reserialized = resolved_actual.to_ron().unwrap();
        assert_eq!(ron_str_expected, ron_str_reserialized);
        
        let reparsed = PathKind::from_inlined_ron(&ron_str_reserialized, &inline_config).unwrap();
        assert_eq!(resolved_expected, reparsed);
    }
    
    #[tested]
    fn test_abcdefg_ec() {
        let _test = testing::test!();
        
        let include_dir = GROUP_ABCDEFG_EC.fixture_dir();
        let resolved_expected = abcdefg_ec_resolved_expected();
        let ron_file = include_dir.join("abcdefg_ec.ron");
        
        let ron_str_expected = fs::read_to_string(&ron_file).unwrap();
        let inline_config = InlinedRonConfig::new(include_dir.into());
        
        let resolved_actual = PathKind::from_inlined_ron_file(&ron_file, &inline_config).unwrap();
        assert_eq!(resolved_expected, resolved_actual);
        
        let ron_str_reserialized = resolved_actual.to_ron().unwrap();
        assert_eq!(ron_str_expected, ron_str_reserialized);
        
        let reparsed = PathKind::from_inlined_ron(&ron_str_reserialized, &inline_config).unwrap();
        assert_eq!(resolved_expected, reparsed);
    }
    
    /// test: circular dependency detection
    #[tested]
    fn test_abcdefg_ec_ge() {
        let _test = testing::test!();
        
        let include_dir = GROUP_ABCDEFG_EC_GE.fixture_dir();
        let ron_file = include_dir.join("abcdefg_ec_ge.ron");
        
        let inline_config = InlinedRonConfig::new(include_dir.into());
        
        assert!(PathKind::from_inlined_ron_file(&ron_file, &inline_config).is_err());
    }
    
    fn abc_resolved_expected() -> PathKind {
        PathKind::Dir(PathKindDir {
            name: "abc".into(),
            children: vec![
                InlinedRon::Included(RonIncluded(
                    None,
                    PathBuf::from("a.ron"),
                    PathKind::Dir(PathKindDir {
                        name: "a".into(),
                        children: vec![
                            InlinedRon::Actual(PathKind::File(PathKindFile {
                                name: "file_a_0.txt".into(),
                            })),
                            InlinedRon::Included(RonIncluded(
                                None,
                                PathBuf::from("a/b.ron"),
                                PathKind::Dir(PathKindDir {
                                    name: "b".into(),
                                    children: vec![
                                        InlinedRon::Actual(PathKind::File(PathKindFile {
                                            name: "file_b_0.txt".into(),
                                        })),
                                        InlinedRon::Included(RonIncluded(
                                            None,
                                            PathBuf::from("a/b/file_b_1.ron"),
                                            PathKind::File(PathKindFile {
                                                name: "file_b_1.txt".into(),
                                            }),
                                        )),
                                    ],
                                })
                            )),
                            InlinedRon::Included(RonIncluded(
                                None,
                                PathBuf::from("a/c.ron"),
                                PathKind::Dir(PathKindDir {
                                    name: "c".into(),
                                    children: vec![
                                        InlinedRon::Actual(PathKind::File(PathKindFile {
                                            name: "file_c_0.txt".into(),
                                        })),
                                        InlinedRon::Included(RonIncluded(
                                            None,
                                            PathBuf::from("a/c/file_c_1.ron"),
                                            PathKind::File(PathKindFile {
                                                name: "file_c_1.txt".into(),
                                            }),
                                        )),
                                    ],
                                })
                            )),
                        ]
                    })),
                )
            ]
        })
    }
    
    fn abcdefg_resolved_expected() -> PathKind {
        PathKind::Dir(PathKindDir {
            name: "abcdefg".into(),
            children: vec![
                InlinedRon::Included(RonIncluded(
                    None,
                    PathBuf::from("a.ron"),
                    PathKind::Dir(PathKindDir {
                        name: "a".into(),
                        children: vec![
                            InlinedRon::Actual(PathKind::File(PathKindFile {
                                name: "file_a_0.txt".into(),
                            })),
                            InlinedRon::Included(RonIncluded(
                                None,
                                PathBuf::from("a/b.ron"),
                                PathKind::Dir(PathKindDir {
                                    name: "b".into(),
                                    children: vec![
                                        InlinedRon::Actual(PathKind::File(PathKindFile {
                                            name: "file_b_0.txt".into(),
                                        })),
                                        InlinedRon::Included(RonIncluded(
                                            None,
                                            PathBuf::from("a/b/file_b_1.ron"),
                                            PathKind::File(PathKindFile {
                                                name: "file_b_1.txt".into(),
                                            }),
                                        )),
                                        InlinedRon::Included(RonIncluded(
                                            None,
                                            PathBuf::from("a/b/d.ron"),
                                            PathKind::Dir(PathKindDir {
                                                name: "d".into(),
                                                children: vec![
                                                    InlinedRon::Actual(PathKind::File(PathKindFile {
                                                        name: "file_d_0.txt".into(),
                                                    })),
                                                    InlinedRon::Included(RonIncluded(
                                                        None,
                                                        PathBuf::from("a/b/d/file_d_1.ron"),
                                                        PathKind::File(PathKindFile {
                                                            name: "file_d_1.txt".into(),
                                                        }),
                                                    )),
                                                ],
                                            }),
                                        )),
                                        InlinedRon::Included(RonIncluded(
                                            None,
                                            PathBuf::from("a/b/e.ron"),
                                            PathKind::Dir(PathKindDir {
                                                name: "e".into(),
                                                children: vec![
                                                    InlinedRon::Actual(PathKind::File(PathKindFile {
                                                        name: "file_e_0.txt".into(),
                                                    })),
                                                    InlinedRon::Included(RonIncluded(
                                                        None,
                                                        PathBuf::from("a/b/e/file_e_1.ron"),
                                                        PathKind::File(PathKindFile {
                                                            name: "file_e_1.txt".into(),
                                                        }),
                                                    )),
                                                ],
                                            }),
                                        )),
                                    ],
                                })
                            )),
                            InlinedRon::Included(RonIncluded(
                                None,
                                PathBuf::from("a/c.ron"),
                                PathKind::Dir(PathKindDir {
                                    name: "c".into(),
                                    children: vec![
                                        InlinedRon::Actual(PathKind::File(PathKindFile {
                                            name: "file_c_0.txt".into(),
                                        })),
                                        InlinedRon::Included(RonIncluded(
                                            None,
                                            PathBuf::from("a/c/file_c_1.ron"),
                                            PathKind::File(PathKindFile {
                                                name: "file_c_1.txt".into(),
                                            }),
                                        )),
                                        InlinedRon::Included(RonIncluded(
                                            None,
                                            PathBuf::from("a/c/f.ron"),
                                            PathKind::Dir(PathKindDir {
                                                name: "f".into(),
                                                children: vec![
                                                    InlinedRon::Actual(PathKind::File(PathKindFile {
                                                        name: "file_f_0.txt".into(),
                                                    })),
                                                    InlinedRon::Included(RonIncluded(
                                                        None,
                                                        PathBuf::from("a/c/f/file_f_1.ron"),
                                                        PathKind::File(PathKindFile {
                                                            name: "file_f_1.txt".into(),
                                                        }),
                                                    )),
                                                ],
                                            }),
                                        )),
                                        InlinedRon::Included(RonIncluded(
                                            None,
                                            PathBuf::from("a/c/g.ron"),
                                            PathKind::Dir(PathKindDir {
                                                name: "g".into(),
                                                children: vec![
                                                    InlinedRon::Actual(PathKind::File(PathKindFile {
                                                        name: "file_g_0.txt".into(),
                                                    })),
                                                    InlinedRon::Included(RonIncluded(
                                                        None,
                                                        PathBuf::from("a/c/g/file_g_1.ron"),
                                                        PathKind::File(PathKindFile {
                                                            name: "file_g_1.txt".into(),
                                                        }),
                                                    )),
                                                ],
                                            }),
                                        )),
                                    ],
                                })
                            )),
                        ]
                    })),
                )
            ]
        })
    }
    
    fn abcdefg_ec_resolved_expected() -> PathKind {
        PathKind::Dir(PathKindDir {
            name: "abcdefg_ec".into(),
            children: vec![
                InlinedRon::Included(RonIncluded(
                    None,
                    PathBuf::from("a.ron"),
                    PathKind::Dir(PathKindDir {
                        name: "a".into(),
                        children: vec![
                            InlinedRon::Actual(PathKind::File(PathKindFile {
                                name: "file_a_0.txt".into(),
                            })),
                            InlinedRon::Included(RonIncluded(
                                None,
                                PathBuf::from("a/b.ron"),
                                PathKind::Dir(PathKindDir {
                                    name: "b".into(),
                                    children: vec![
                                        InlinedRon::Actual(PathKind::File(PathKindFile {
                                            name: "file_b_0.txt".into(),
                                        })),
                                        InlinedRon::Included(RonIncluded(
                                            None,
                                            PathBuf::from("a/b/file_b_1.ron"),
                                            PathKind::File(PathKindFile {
                                                name: "file_b_1.txt".into(),
                                            }),
                                        )),
                                        InlinedRon::Included(RonIncluded(
                                            None,
                                            PathBuf::from("a/b/d.ron"),
                                            PathKind::Dir(PathKindDir {
                                                name: "d".into(),
                                                children: vec![
                                                    InlinedRon::Actual(PathKind::File(PathKindFile {
                                                        name: "file_d_0.txt".into(),
                                                    })),
                                                    InlinedRon::Included(RonIncluded(
                                                        None,
                                                        PathBuf::from("a/b/d/file_d_1.ron"),
                                                        PathKind::File(PathKindFile {
                                                            name: "file_d_1.txt".into(),
                                                        }),
                                                    )),
                                                ],
                                            }),
                                        )),
                                        InlinedRon::Included(RonIncluded(
                                            None,
                                            PathBuf::from("a/b/e.ron"),
                                            PathKind::Dir(PathKindDir {
                                                name: "e".into(),
                                                children: vec![
                                                    InlinedRon::Actual(PathKind::File(PathKindFile {
                                                        name: "file_e_0.txt".into(),
                                                    })),
                                                    InlinedRon::Included(RonIncluded(
                                                        None,
                                                        PathBuf::from("a/b/e/file_e_1.ron"),
                                                        PathKind::File(PathKindFile {
                                                            name: "file_e_1.txt".into(),
                                                        }),
                                                    )),
                                                    InlinedRon::Included(RonIncluded(
                                                        None,
                                                        PathBuf::from("a/c.ron"),
                                                        PathKind::Dir(PathKindDir {
                                                            name: "c".into(),
                                                            children: vec![
                                                                InlinedRon::Actual(PathKind::File(PathKindFile {
                                                                    name: "file_c_0.txt".into(),
                                                                })),
                                                                InlinedRon::Included(RonIncluded(
                                                                    None,
                                                                    PathBuf::from("a/c/file_c_1.ron"),
                                                                    PathKind::File(PathKindFile {
                                                                        name: "file_c_1.txt".into(),
                                                                    }),
                                                                )),
                                                                InlinedRon::Included(RonIncluded(
                                                                    None,
                                                                    PathBuf::from("a/c/f.ron"),
                                                                    PathKind::Dir(PathKindDir {
                                                                        name: "f".into(),
                                                                        children: vec![
                                                                            InlinedRon::Actual(PathKind::File(PathKindFile {
                                                                                name: "file_f_0.txt".into(),
                                                                            })),
                                                                            InlinedRon::Included(RonIncluded(
                                                                                None,
                                                                                PathBuf::from("a/c/f/file_f_1.ron"),
                                                                                PathKind::File(PathKindFile {
                                                                                    name: "file_f_1.txt".into(),
                                                                                }),
                                                                            )),
                                                                        ],
                                                                    }),
                                                                )),
                                                                InlinedRon::Included(RonIncluded(
                                                                    None,
                                                                    PathBuf::from("a/c/g.ron"),
                                                                    PathKind::Dir(PathKindDir {
                                                                        name: "g".into(),
                                                                        children: vec![
                                                                            InlinedRon::Actual(PathKind::File(PathKindFile {
                                                                                name: "file_g_0.txt".into(),
                                                                            })),
                                                                            InlinedRon::Included(RonIncluded(
                                                                                None,
                                                                                PathBuf::from("a/c/g/file_g_1.ron"),
                                                                                PathKind::File(PathKindFile {
                                                                                    name: "file_g_1.txt".into(),
                                                                                }),
                                                                            )),
                                                                        ],
                                                                    }),
                                                                )),
                                                            ],
                                                        })
                                                    )),
                                                ],
                                            }),
                                        )),
                                    ],
                                })
                            )),
                            InlinedRon::Included(RonIncluded(
                                None,
                                PathBuf::from("a/c.ron"),
                                PathKind::Dir(PathKindDir {
                                    name: "c".into(),
                                    children: vec![
                                        InlinedRon::Actual(PathKind::File(PathKindFile {
                                            name: "file_c_0.txt".into(),
                                        })),
                                        InlinedRon::Included(RonIncluded(
                                            None,
                                            PathBuf::from("a/c/file_c_1.ron"),
                                            PathKind::File(PathKindFile {
                                                name: "file_c_1.txt".into(),
                                            }),
                                        )),
                                        InlinedRon::Included(RonIncluded(
                                            None,
                                            PathBuf::from("a/c/f.ron"),
                                            PathKind::Dir(PathKindDir {
                                                name: "f".into(),
                                                children: vec![
                                                    InlinedRon::Actual(PathKind::File(PathKindFile {
                                                        name: "file_f_0.txt".into(),
                                                    })),
                                                    InlinedRon::Included(RonIncluded(
                                                        None,
                                                        PathBuf::from("a/c/f/file_f_1.ron"),
                                                        PathKind::File(PathKindFile {
                                                            name: "file_f_1.txt".into(),
                                                        }),
                                                    )),
                                                ],
                                            }),
                                        )),
                                        InlinedRon::Included(RonIncluded(
                                            None,
                                            PathBuf::from("a/c/g.ron"),
                                            PathKind::Dir(PathKindDir {
                                                name: "g".into(),
                                                children: vec![
                                                    InlinedRon::Actual(PathKind::File(PathKindFile {
                                                        name: "file_g_0.txt".into(),
                                                    })),
                                                    InlinedRon::Included(RonIncluded(
                                                        None,
                                                        PathBuf::from("a/c/g/file_g_1.ron"),
                                                        PathKind::File(PathKindFile {
                                                            name: "file_g_1.txt".into(),
                                                        }),
                                                    )),
                                                ],
                                            }),
                                        )),
                                    ],
                                })
                            )),
                        ]
                    })),
                )
            ]
        })
    }
}
