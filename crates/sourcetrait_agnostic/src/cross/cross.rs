use crate::*;

pub trait CrossPlatform: Sized {
    const OS: Os;
    const OS_FAMILY: OsFamily;
    
    fn access(&self) -> impl AccessComponentTrait;

    fn capabilities(&self) -> Capabilities;
    
    fn cmd(&self) -> impl CmdComponentTrait;
    
    fn fs(&self) -> impl FilesComponentTrait;

    fn net(&self) -> impl NetComponentTrait;

    fn path(&self) -> impl PathsComponentTrait;
    
    fn ui(&self) -> impl UiComponentTrait;
    
    fn capable<C: Into<Capabilities>>(&self, capabilities: C) -> CrossResult<bool> {
        Ok(self.capabilities() & capabilities.into() != Capabilities::NONE)
    }
    
    fn os(&self) -> Os {
        Self::OS
    }
    
    fn os_family(&self) -> OsFamily {
        Self::OS_FAMILY
    }
}

pub struct Platform<OS: CrossPlatform> {
    pub(crate) os: OS
}

impl<OS: CrossPlatform> Platform<OS> {
    pub const fn new(os: OS) -> Self {
        Self {
            os
        }
    }
}

impl<PLAT: CrossPlatform> CrossPlatform for Platform<PLAT> {
    const OS: Os = PLAT::OS;
    const OS_FAMILY: OsFamily = PLAT::OS_FAMILY;
    
    
    #[inline]
    fn access(&self) -> impl AccessComponentTrait {
        self.os.access()
    }

    #[inline]
    fn capabilities(&self) -> Capabilities {
        self.os.capabilities()
    }

    #[inline]
    fn cmd(&self) -> impl CmdComponentTrait {
        self.os.cmd()
    }
    
    #[inline]
    fn fs(&self) -> impl FilesComponentTrait {
        self.os.fs()
    }

    #[inline]
    fn net(&self) -> impl NetComponentTrait {
        self.os.net()
    }

    #[inline]
    fn path(&self) -> impl PathsComponentTrait {
        self.os.path()
    }

    #[inline]
    fn ui(&self) -> impl UiComponentTrait {
        self.os.ui()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Os {
    Unsupported,
    Linux,
    MacOS,
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum OsFamily {
    Unsupported,
    Unix,
    Windows,
}
