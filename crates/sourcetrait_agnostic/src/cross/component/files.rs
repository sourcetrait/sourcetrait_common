use crate::*;

pub trait FilesComponentTrait {
    fn copy_preserved<P1, P2>(&self, src: P1, dst: P2) -> CrossResult<()>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>
    {
        self.copy_preserved_with(src, dst, FsOptions::default())
    }
    
    fn copy_preserved_with<P1, P2>(&self, src: P1, dst: P2, opts: impl AsRef<FsOptions>) -> CrossResult<()>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>;
    
    fn own_capable<P, UAID, GAID>(
        &self,
        dst: P,
        user: UAID,
        group: impl AsRef<Capable<PrimaryUserGroupsCapable, GAID>>,
        perms: impl AsRef<BasicPermissionMode>,
    ) -> CrossResult<()>
    where
        P: AsRef<Path>,
        UAID: AsAID,
        GAID: AsAID,
    {
        self.own_capable_with(dst, user, group, perms, &FsOptions::default())
    }
    
    fn own_capable_with<P, UAID, GAID>(
        &self,
        dst: P,
        user: UAID,
        group: impl AsRef<Capable<PrimaryUserGroupsCapable, GAID>>,
        perms: impl AsRef<BasicPermissionMode>,
        opts: impl AsRef<FsOptions>) -> CrossResult<()>
    where
        P: AsRef<Path>,
        UAID: AsAID,
        GAID: AsAID;
}

#[allow(private_bounds)]
pub struct StandardFilesComponent<LOOKUP: FilesComponentLookup>(pub(crate) LOOKUP);

#[allow(private_bounds)]
impl<LOOKUP: FilesComponentLookup> StandardFilesComponent<LOOKUP> {
    #[inline]
    fn lookup(&self) -> &LOOKUP { &self.0 }
}

impl<LOOKUP: FilesComponentLookup> FilesComponentTrait for StandardFilesComponent<LOOKUP> {
    fn copy_preserved_with<P1, P2>(&self, src: P1, dst: P2, opts: impl AsRef<FsOptions>) -> CrossResult<()>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>
    {
        self.lookup().copy_preserved_with(src, dst, opts)?;
        Ok(())
    }
        
    fn own_capable_with<P,UAID,GAID>(
        &self,
        dst: P,
        user: UAID,
        group: impl AsRef<Capable<PrimaryUserGroupsCapable, GAID>>,
        perms: impl AsRef<BasicPermissionMode>,
        opts: impl AsRef<FsOptions>) -> CrossResult<()>
    where
        UAID: AsAID,
        GAID: AsAID,
        P: AsRef<Path>,
    {
        self.lookup().own_capable_with(dst, user, group, perms, opts)?;
        Ok(())
    }
}
