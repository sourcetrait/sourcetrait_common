use std::path::{Path, PathBuf};
use rand::Rng;
use crate::*;

/// Creates a subdirectory with a name of `prefix.XXXXXXXX`, where X is a
/// a random alphanumeric character.
pub fn create_random_subdir(base_dir: &Path, prefix: &str) -> anyhow::Result<PathBuf> {
    const MAX_RAND_DIR_RETRIES: i32 = 64;
    const MAX_RAND_DIR_CHARS: i32 = 8;

    let mut randgen = rand::rng();
    let mut random_dir;

    for _ in 0..MAX_RAND_DIR_RETRIES {
        let rand_chars: String = (0..MAX_RAND_DIR_CHARS)
            .map(|_| randgen.sample(rand::distr::Alphanumeric) as char)
            .collect();

        let name = format!("{prefix}.{rand_chars}");
        random_dir = base_dir.join(name);
        if random_dir.exists() {
            continue;
        }

        if std::fs::create_dir_all(&random_dir).is_ok() {
            return Ok(random_dir.canonicalize()?);
        }
    }

    bail!("Unable to create random subdirectory in: {}", base_dir.to_str().unwrap())
}
