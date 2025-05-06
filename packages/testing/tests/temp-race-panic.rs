#[cfg(test)]
mod tests{
    use asmov_common_testing::prelude::*;

    static TESTING: testing::Module = testing::module!(Integration, {
        .using_temp_dir()
    });

    #[tested]
    fn test_a() {
        let test = testing::test!({
            .using_temp_dir()
        });

        assert!(test.temp_dir().exists());
    }

    #[should_panic]
    #[tested]
    fn test_b() {
        let test = testing::test!({
            .using_temp_dir()
        });

        assert!(test.temp_dir().exists());
        panic!("should panic");
    }

    #[tested]
    fn test_c() {
        let test = testing::test!({
            .using_temp_dir()
        });

        assert!(test.temp_dir().exists());
    }

    #[should_panic]
    #[tested]
    fn test_d() {
        let test = testing::test!({
            .using_temp_dir()
        });

        assert!(test.temp_dir().exists());
        panic!("should panic");
    }

    #[tested]
    fn test_e() {
        let test = testing::test!({
            .using_temp_dir()
        });

        assert!(test.temp_dir().exists());
    }

    #[should_panic]
    #[tested]
    fn test_f() {
        let test = testing::test!({
            .using_temp_dir()
        });

        assert!(test.temp_dir().exists());
        panic!("should panic");
    }

    #[tested]
    fn test_g() {
        let test = testing::test!({
            .using_temp_dir()
        });

        assert!(test.temp_dir().exists());
    }

    #[should_panic]
    #[tested]
    fn test_h() {
        let test = testing::test!({
            .using_temp_dir()
        });

        assert!(test.temp_dir().exists());
        panic!("should panic");
    }

    #[tested]
    fn test_i() {
        let test = testing::test!({
            .using_temp_dir()
        });

        assert!(test.temp_dir().exists());
    }

    #[should_panic]
    #[tested]
    fn test_j() {
        let test = testing::test!({
            .using_temp_dir()
        });

        assert!(test.temp_dir().exists());
        panic!("should panic");
    }

    #[tested]
    fn test_k() {
        let test = testing::test!({
            .using_temp_dir()
        });

        assert!(test.temp_dir().exists());
    }
}
