use crate::*;

pub trait PathsComponentLookup {
    fn lookup_env_paths(&self) -> BridgeResult<Vec<PathBuf>>;
}
