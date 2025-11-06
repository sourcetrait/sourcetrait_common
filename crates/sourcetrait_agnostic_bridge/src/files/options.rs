//use crate::*;

/// Options used with unix filesystem operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FsOptions {
    /// Dereference symbolic links.
    /// - FALSE: Operates on the symbolic link itself
    /// - TRUE (default): Operates on the actual file linked to
    pub follow_symlinks: bool,
    
    /// Ignore when extended attributes exist for the source path,
    /// but are not supported by destination filesystem.
    /// - FALSE (default): Operation fails and throws an error  
    /// - TRUE: Operation proceeds
    /// 
    /// Valid only for [copy_preserved]
    pub lossy_extended_attributes: bool,
}

impl FsOptions {
    pub const DEFAULT: Self = Self {
        follow_symlinks: true,
        lossy_extended_attributes: false,
    };
    
    pub const DEFAULT_NOFOLLOW: Self = Self {
        follow_symlinks: false,
        ..Self::DEFAULT
    };
}

impl FsOptions {
    pub fn default_nofollow() -> Self { Self::DEFAULT_NOFOLLOW }
}

impl Default for FsOptions {
    #[inline]
    fn default() -> Self { Self::DEFAULT }
}

impl AsRef<FsOptions> for FsOptions {
    #[inline]
    fn as_ref(&self) -> &FsOptions { self }
}