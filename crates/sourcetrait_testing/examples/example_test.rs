fn main() {}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use sourcetrait_testing::prelude::*;
    
    static TESTING: testing::Module = testing::module!(Example, {
        .using_fixture_dir()
    });
    
    #[tested]
    fn test_example() {
        const NAMEPATH: &'static str = "sourcetrait_testing/example/example-test/test-example";
        let test = testing::test!({
            .using_fixture_dir()
        });
        
        let expected = Path::new(NAMEPATH);
        let actual = test.namepath().full_path();
        assert_eq!(expected, actual);
    }
}
