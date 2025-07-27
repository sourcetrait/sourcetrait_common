use std::{fs::File, path::Path};
use tar::Archive;

#[allow(unused)]
pub(crate) fn untar_fixture_repo(tar_filepath: &Path, target_dir: &Path) {
    let tarfile = File::open(tar_filepath).unwrap();
    let mut tar = Archive::new(tarfile);

    for file in tar.entries().unwrap() {
        let mut file = file.unwrap();
        file.set_unpack_xattrs(false);
        file.set_preserve_permissions(false);
        let target_filepath = target_dir.join(file.path().unwrap());
        file.unpack(target_filepath).unwrap();
    }
}
