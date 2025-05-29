CHANGES: SrcTrait Common Extended Standard Library
===============================================================================

## v1.1
### Added `path::normalize::NormalizePath`
Extends `std::path::{Path, PathBuf}` with the `normalize_relative()` function,
which can be used to sanitize paths provided by user input. It prevents
dot-walking up the file tree at the cost of ignoring symlinked '..' paths.
