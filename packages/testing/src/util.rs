use std::path::{Path, PathBuf};
use crate::*;

// Helper function for test models configuring their temp_dir during `build()`.
pub(crate) fn build_temp_dir(namepath: &Namepath, base_temp_dir: &Path) -> PathBuf {
    let temp_dir = base_temp_dir.join(namepath.path());

    if !temp_dir.exists() {
        std::fs::create_dir_all(&temp_dir)
            .context(format!("Unable to create temporary testing directory: {} :: Testing Namepath: {}",
                temp_dir.to_string_lossy(),
                namepath.full_path().to_string_lossy()
            ))
            .unwrap();
    }

    temp_dir.canonicalize().unwrap()
}

pub(crate) fn build_fixture_dir(namepath: &Namepath) -> PathBuf {
    // path: ./ testing / fixtures / [ unit | integration | benchmark ] / { module } / { group ... } / { test }
    let fixture_dir = PathBuf::from(strings::TESTING)
        .join(strings::FIXTURES)
        .join(namepath.path());
    let fixture_dir = fixture_dir.canonicalize()
        .context(format!("Fixture directory does not exist: {} :: Testing Namepath: {}",
            fixture_dir.to_string_lossy(),
            namepath.full_path().to_string_lossy()
        ))
        .unwrap();

    fixture_dir
}
