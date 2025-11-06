# SourceTrait Common: Testing
> Structured testing with setup, teardown, and standardized fixture and temp directories

## Features
- Structure tests into inheritable heirarchies: Module -> Test
- Bundle common assets outside of heirarchies using Group
- Setup and teardown callbacks for each Module, Test, and Group 
- Module and Group setup occurs before Test
- Categorize based on use-case: Unit and Integration
- Standardized paths for fixture and temp directories 
- Miscellaneous helper functions for common tasks

## Restrictions
- One Module per Rust test module
- One Test per Rust test function
- Module and Group are static to a Rust module
- Test is instantiated non-static inside of the Rust test function
- Unit tests live in `src`
- Integration tests live in `tests` or `example`
- Fixture asset files live in `testing`

## Standard paths
> *use-case / module_path / function_name*

## Namepaths
The crate name and `::tests` are stripped from paths.

### Format
|        |                                             | 
|--------|---------------------------------------------|
| Module | `use-case / module_path ..`                 |
| Test   | `use_case / module_path .. / function_name` |
| Group  | `use_case / arbitrary_path ..`              |

### Examples
|        |             |                                                      |
|--------|-------------|------------------------------------------------------|
| Module | Unit        | `my-crate::process::numbers::tests`                  |
|        |             | *unit/process/numbers*                               |
| Module | Integration | `test-module::tests`                                 |
|        |             | *integration/test-module*                            |
| Test   | Unit        | `my-crate::process::numbers::tests::test_func_one()` |
|        |             | *unit/process/numbers/test_func_one*                 |
| Test   | Integration | `test-module::tests::test_func_two()`                |
|        |             | *integration/test-module/test_func_two*              |
| Group  | Unit        | `my-crate::process::numbers::tests` "primes/large"   |
|        |             | *unit/primes/large*                                  |
| Group  | Integration | `test-module::tests` "primes/large"                  |
|        |             | *integration/primes/large*                           |

## Example
```rust
// Just something to test against
pub fn fibonacci(n: u8) -> Option<u128> {
    if n == 0 {
        return Some(0);
    } else if n == 1 {
        return Some(1);
    } else if n > FIBONACCI_MAX_N_U128 { // overflow u128
        return None;
    }

    let mut sum: u128 = 0;
    let mut last: u128 = 0;
    let mut curr: u128 = 1;
    for _i in 1..n {
        if let Some(r) = last.checked_add(curr) {
            sum = r;
        } else {
            return None;
        }

        last = curr;
        curr = sum;
    }

    Some(sum)
}

const FIBONACCI_MAX_N_U128: u8 = 185;

// Testing namepaths will strip '::tests' out.
#[cfg(test)]
mod tests {
    use std::{fs, sync::LazyLock};
    use super::*;
    use sourcetrait_testing::prelude::*;

    // Module setup runs before any tests that use the module.
    // Teardown runs on process exit.
    //
    // Fixture dirs are enforced at runtime and panic if they don't exist.
    //
    // This module's namepath().full_path() will be:
    //     sourcetrait_testing/integration/example-fibonacci
    //
    // This module's namepath().path() will be:
    //     integration/example-fibonacci
    //
    // fixture dir: $CRATE/testing/fixtures/integration/example-fibonacci
    // temp dir: $TMP/sourcetrait_testing-example-fibonacci.XXXXXXXX
    static TESTING: testing::Module = testing::module!(Integration, {
            .using_fixture_dir()
            .using_temp_dir()
            .teardown_static(teardown)
            .setup(|module| {
                let fibonacci_u128_csv = (0..FIBONACCI_MAX_N_U128)
                    .map(|n| fibonacci(n).unwrap().to_string())
                    .collect::<Vec<_>>()
                    .join(",");

                let fibonacci_csv_tmp_file = module.temp_dir()
                    .join(&*FIBONACCI_U128_CSV_FILENAME);

                fs::write(&fibonacci_csv_tmp_file, fibonacci_u128_csv).unwrap();
            })
    });

    static FIBONACCI_U128_CSV_FILENAME: LazyLock<String> = LazyLock::new(||
        "fibonacci-u128.csv".to_string());

    // Anything const or static can be used with static teardown functions,
    // including things like LazyLock.
    //
    // Module's internal teardown will delete its temp directory before
    // running its custom teardown.
    extern "C" fn teardown() {
        println!("Farewell, Fibonacci. Don't worry, {} has already been deleted.",
            *FIBONACCI_U128_CSV_FILENAME);
    }

    // Groups are standalone and work similar to Modules, but without children.
    // They have their own tmp and fixture directories, setup, and teardown.
    //
    // This group's namepath().full_path() will be:
    //     sourcetrait_testing/integration/example-fibonacci/100
    // This group's namepath().path() will be:
    //     integration/example-fibonacci/100
    static GROUP_100: testing::Group = testing::group!("example-fibonacci/100", Integration, {
        .using_fixture_dir()
    });

    // The #[tested] helper is merely the equivalent of #[test] and #[named]
    #[tested]
    fn test_u128() {
        // test!() assumes a module named `TESTING` is in scope and links to it.
        //
        // The "inherit_" methods use the same dir as their parent module.
        // The "using_" methods will append a subdir named for the test function.
        //
        // Tests block on their module's setup. The tmp file needed here will exist.
        //
        // fixture dir: $CRATE/testing/fixtures/integration/example-fibonacci/test-u128
        // tmp dir: $TMP/sourcetrait_testing-example-fibonacci.XXXXXXXX
        let test = testing::test!({
            .using_fixture_dir()
            .inherit_temp_dir()
        });

        let fibonacci_fixture_csv_file = test.fixture_dir().join(&*FIBONACCI_U128_CSV_FILENAME);
        let expected_fibonacci_u128 = fs::read_to_string(fibonacci_fixture_csv_file).unwrap()
            .split(",")
            .map(|d| d.trim().parse().unwrap())
            .collect::<Vec<u128>>();

        let fibonacci_tmp_csv_file = test.temp_dir().join(&*FIBONACCI_U128_CSV_FILENAME);
        let actual_fibonacci_u128 = fs::read_to_string(fibonacci_tmp_csv_file).unwrap()
            .split(",")
            .map(|i| i.parse().unwrap())
            .collect::<Vec<u128>>();

        assert_eq!(expected_fibonacci_u128, actual_fibonacci_u128);
    }

    // Demonstration using a Group
    #[tested]
    fn test_100() {
        // This empty Test will block on its Module's setup.
        let _test = testing::test!();

        // Groups will block on their setup.
        let fib100_file = GROUP_100.fixture_dir().join("fib-100.txt");

        let fib100: u128 = fs::read_to_string(fib100_file)
            .map(|d| d.trim().parse().unwrap())
            .unwrap();

        assert_eq!(fib100, fibonacci(100).unwrap());
    }
}
```
