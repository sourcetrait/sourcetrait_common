use crate::*;

pub trait FilesComponentLookup {
    fn copy_preserved<P1, P2>(&self, src: P1, dst: P2) -> BridgeResult<()>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        self.copy_preserved_with(src, dst, FsOptions::default())
    }
    
    fn copy_preserved_with<P1, P2>(&self, src: P1, dst: P2, opts: impl AsRef<FsOptions>) -> BridgeResult<()>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>;
    
    fn own_capable<P, UAID, GAID>(
        &self,
        dst: P,
        user: UAID,
        group: impl AsRef<Capable<PrimaryUserGroupsCapable, GAID>>,
        perms: impl AsRef<BasicPermissionMode>,
    ) -> BridgeResult<()>
    where
        UAID: AsAID,
        GAID: AsAID,
        P: AsRef<Path>
    {
        self.own_capable_with(dst, user, group, perms, FsOptions::default())
    }
    
    fn own_capable_with<P, UAID, GAID>(
        &self,
        dst: P,
        user: UAID,
        group: impl AsRef<Capable<PrimaryUserGroupsCapable, GAID>>,
        perms: impl AsRef<BasicPermissionMode>,
        opts: impl AsRef<FsOptions>,
    ) -> BridgeResult<()>
    where
        UAID: AsAID,
        GAID: AsAID,
        P: AsRef<Path>;
}
