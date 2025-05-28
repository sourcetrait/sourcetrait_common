CHANGES: SrcTrait Common Testing
===============================================================================

## v4.1
### Added `ModuleBuilder::skip_temp_dir_teardown()`
Skips deletion of its temp_dir and logs the location.

Use this to capture new expected output. It will be logged as:  
"TESTING: <namepath> :: Skipped teardown of temp_dir"

Reminder, use `cargo test -- --show-output` to see the log.

## v4.0.1
Added a linking exception to the license via AGPL3 section 7.

tldr: Any project can use this library as a dependency, regardless of their
own license, so long as they play by AGPL3 rules when dealing with this library.
