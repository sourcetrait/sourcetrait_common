use std::path::Path;

use walkdir;
use sourcetrait_dotrepo::*;

pub(crate) fn map_designated<R: 'static + DotRepoType>(entry: walkdir::Result<(walkdir::DirEntry, DesignatorMatches<R>)>, base: &Path) -> (String, DesignatorMatches<R>) {
    let (entry, matches) = entry.unwrap();
    
    let path = entry.into_path()
        .strip_prefix(base).unwrap()
        .to_str().unwrap()
        .to_string();
    
    (path, matches)
}

pub(crate) fn map_paths(entry: walkdir::Result<walkdir::DirEntry>, base: &Path) -> String {
    entry.unwrap()
        .into_path()
        .strip_prefix(base).unwrap()
        .to_str().unwrap()
        .to_string()
}
