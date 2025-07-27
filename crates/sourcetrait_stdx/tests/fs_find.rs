#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use sourcetrait_stdx::fs::find;
    use sourcetrait_testing::prelude::*;

    /// SAFETY: this will break if this file moves and the path isn't updated
    const GIT_FROM_FIXTURE_DIR: &'static str = "../../";

    static TESTING: testing::Module = testing::module!(Integration, {
        .using_fixture_dir()
    });

    #[tested]
    fn test_find_git() {
        let test = testing::test!();
        // SAFETY: see GIT_FROM_FIXTURE_DIR
        let expected_repo_path = PathBuf::from(GIT_FROM_FIXTURE_DIR)
            .canonicalize().unwrap();


        assert_eq!(
            Some(expected_repo_path),
            find::find_parent_dir(test.module().fixture_dir(), ".git")
        );
    }
}
