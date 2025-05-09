#[cfg(test)]
mod tests {
    use asmov_common_testing::prelude::*;

    static TESTING: testing::Module = testing::module!(Integration, {
        .using_fixture_dir()
        .using_temp_dir()
    });

    #[tested]
    fn test_intg() {
        let test = testing::test!({
            .inherit_fixture_dir()
            .using_temp_dir()
        });

        assert!(test.fixture_dir().exists())
    }
}
