use crate::*;

pub struct LinuxCrossPlatform;
impl CrossPlatform for LinuxCrossPlatform {
    const OS: Os = Os::Linux;
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
        StandardCmdComponent(linux::LinuxCmdComponentLookup)
    }
    
    #[inline]
    fn fs(&self) -> impl FilesComponentTrait {
        StandardFilesComponent(unix::UnixFilesComponentLookup)
    }

    #[inline]
    fn net(&self) -> impl NetComponentTrait {
        StandardNetComponent(linux::LinuxNetComponentLookup)
    }
    
    #[inline]
    fn path(&self) -> impl PathsComponentTrait {
        StandardPathsComponent(unix::UnixPathsComponentLookup)
    }
    
    #[inline]
    fn ui(&self) -> impl UiComponentTrait {
        StandardUiComponent(linux::LinuxUiComponentLookup)
    }
}
