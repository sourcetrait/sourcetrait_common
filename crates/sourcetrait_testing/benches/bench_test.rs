#![cfg_attr(feature = "nightly", feature(test))]

#[cfg(feature = "nightly")]
#[cfg(test)]
mod benches {
    extern crate test;
    use std::path::Path;
    use sourcetrait_testing::prelude::*;
    
    static TESTING: testing::Module = testing::module!(Benchmark, {
        .using_fixture_dir()
    });
    
    #[benched]
    fn bench_bencher(bencher: &mut test::Bencher) {
        const NAMEPATH: &'static str = "sourcetrait_testing/benchmark/bench-test/bench-bencher";
        let test = testing::test!({
            .using_fixture_dir()
        });
        
        let expected = Path::new(NAMEPATH);
        let actual = test.namepath().full_path();
        assert_eq!(expected, actual);
        bencher.iter(|| assert_eq!(expected, actual));
    }
}
