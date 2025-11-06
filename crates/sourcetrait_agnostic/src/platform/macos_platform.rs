use crate::*;

pub struct MacOsCrossPlatform;
impl CrossPlatform for MacOsCrossPlatform {
    const OS: Os = Os::MacOS;
    const OS_FAMILY: OsFamily = OsFamily::Unix;
    
    #[inline]
    fn capabilities(&self) -> Capabilities {
        Capability::UnixIDs | Capability::PrimaryUserGroups
    }
    
    #[inline]
    fn access(&self) -> impl AccessComponentTrait {
        StandardAccessComponent(unix::UnixAccessComponentLookup)
    }
    
    #[inline]
    fn cmd(&self) -> impl CmdComponentTrait {
        StandardCmdComponent(macos::MacOsCmdComponentLookup)
    }
    
    #[inline]
    fn fs(&self) -> impl FilesComponentTrait {
        StandardFilesComponent(unix::UnixFilesComponentLookup)
    }
    
    #[inline]
    fn net(&self) -> impl NetComponentTrait {
        StandardNetComponent(macos::MacOsNetComponentLookup)
    }
    
    #[inline]
    fn path(&self) -> impl PathsComponentTrait {
        StandardPathsComponent(unix::UnixPathsComponentLookup)
    }
    
    #[inline]
    fn ui(&self) -> impl UiComponentTrait {
        StandardUiComponent(macos::MacOsUiComponentLookup)
    }
}

pub fn copy_preserved<P1: AsRef<Path>, P2: AsRef<Path>>(src: P1, dst: P2, opts: &FsOptions) -> Result<(), CopyError> {
    unix::copy_preserved(src, dst, opts)
}
