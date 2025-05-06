#[cfg(test)]
mod tests {
    use asmov_common_testing::prelude::*;

    static TESTING: testing::Module = testing::module!(Integration);

    const GROUP_NAME: &'static str = "namepath-group/uno/dos";
    static GROUP: testing::Group = testing::group!(GROUP_NAME, Integration);

    #[tested]
    fn test_namepath() {
        const EXPECTED_PACKAGE_NAME: &'static str = "asmov-common-testing";
        const EXPECTED_USE_CASE: testing::UseCase = testing::UseCase::Integration;

        const EXPECTED_MODULE_KIND: testing::TestingKind = testing::TestingKind::Module;
        const EXPECTED_MODULE_FULL_PATH: &'static str = "asmov-common-testing/integration/namepaths/name-path/one/two";
        const EXPECTED_MODULE_PATH: &'static str = "integration/namepaths/name-path/one/two";
        const EXPECTED_MODULE_NAME: &'static str = "two";
        const EXPECTED_MODULE_RAW: &'static str = "module;integration;asmov-common-testing;namepaths::name_path::one::two::tests";

        const EXPECTED_GROUP_KIND: testing::TestingKind = testing::TestingKind::Group;
        const EXPECTED_GROUP_FULL_PATH: &'static str = "asmov-common-testing/integration/namepath-group/uno/dos";
        const EXPECTED_GROUP_PATH: &'static str = "integration/namepath-group/uno/dos";
        const EXPECTED_GROUP_NAME: &'static str = "dos";
        const EXPECTED_GROUP_RAW: &'static str = "group;integration;asmov-common-testing;namepath-group/uno/dos";

        const EXPECTED_TEST_KIND: testing::TestingKind = testing::TestingKind::Test;
        const EXPECTED_TEST_FULL_PATH: &'static str = "asmov-common-testing/integration/namepaths/name-path/one/two/test-namepath";
        const EXPECTED_TEST_PATH: &'static str = "integration/namepaths/name-path/one/two/test-namepath";
        const EXPECTED_TEST_NAME: &'static str = "test-namepath";
        const EXPECTED_TEST_RAW: &'static str = "test;integration;asmov-common-testing;namepaths::name_path::one::two::tests;test_namepath";

        // Module
        assert_eq!(EXPECTED_PACKAGE_NAME, TESTING.namepath().package_name());
        assert_eq!(EXPECTED_MODULE_KIND, TESTING.namepath().kind());
        assert_eq!(EXPECTED_USE_CASE, TESTING.namepath().use_case());
        assert_eq!(EXPECTED_MODULE_FULL_PATH, TESTING.namepath().full_path().to_string_lossy());
        assert_eq!(EXPECTED_MODULE_PATH, TESTING.namepath().path().to_string_lossy());
        assert_eq!(EXPECTED_MODULE_NAME, TESTING.namepath().name());
        assert_eq!(EXPECTED_MODULE_RAW, TESTING.namepath().raw().to_string());

        // Group
        assert_eq!(EXPECTED_PACKAGE_NAME, GROUP.namepath().package_name());
        assert_eq!(EXPECTED_GROUP_KIND, GROUP.namepath().kind());
        assert_eq!(EXPECTED_USE_CASE, GROUP.namepath().use_case());
        assert_eq!(EXPECTED_GROUP_FULL_PATH, GROUP.namepath().full_path().to_string_lossy());
        assert_eq!(EXPECTED_GROUP_PATH, GROUP.namepath().path().to_string_lossy());
        assert_eq!(EXPECTED_GROUP_NAME, GROUP.namepath().name());
        assert_eq!(EXPECTED_GROUP_RAW, GROUP.namepath().raw().to_string());

        // Test
        let test = testing::test!();
        assert_eq!(EXPECTED_PACKAGE_NAME, test.namepath().package_name());
        assert_eq!(EXPECTED_TEST_KIND, test.namepath().kind());
        assert_eq!(EXPECTED_USE_CASE, test.namepath().use_case());
        assert_eq!(EXPECTED_TEST_FULL_PATH, test.namepath().full_path().to_string_lossy());
        assert_eq!(EXPECTED_TEST_PATH, test.namepath().path().to_string_lossy());
        assert_eq!(EXPECTED_TEST_NAME, test.namepath().name());
        assert_eq!(EXPECTED_TEST_RAW, test.namepath().raw().to_string());
    }
}
