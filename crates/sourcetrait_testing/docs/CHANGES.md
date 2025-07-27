CHANGES: SourceTrait Common Testing
===============================================================================

## v4.3.0
### Supported benchmarks and examples natively
Use `#[benched]` instead of `#[test]`. `TestingKind` now hase the variants:
`Example` and `Benchmark`. The macros `testing::module!()`, `testing::group!()`,
etc. can be passed a `Benchmark` or `Example` to use them.
### Added `Stepper`
Sequentially runs a series of test steps, allowing for more complex test
scenarios that are reproducable.
### Added `Testable` enum
Allows static dispatch of the `Testable` trait between Test, Module, and Group.
### Added `as_testable()` to Test, Module, and Group
Creates a `Testable` reference.
### Added `Testing::kind()`
Returns the `TestingKind`.

## v4.2.0
### Added `GroupBuilder::skip_temp_dir_teardown()`
Skips deletion of its temp_dir and logs the location. See docs for v4.1.0.

## v4.1.0
### Added `ModuleBuilder::skip_temp_dir_teardown()`
Skips deletion of its temp_dir and logs the location.

Use this to capture new expected output. It will be logged as:  
"TESTING: <namepath> :: Skipped teardown of temp_dir"

Reminder, use `cargo test -- --show-output` to see the log.

## v4.0.1
Added a linking exception to the license via AGPL3 section 7.

tldr: Any project can use this library as a dependency, regardless of their
own license, so long as they play by AGPL3 rules when dealing with this library.
