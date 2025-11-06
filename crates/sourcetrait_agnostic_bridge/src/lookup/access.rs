use crate::*;

pub trait AccessComponentLookup {
    const LOOKUP: AccessLookup;
}

pub(crate) type LookupEffectiveProcessUserFn = fn() -> BridgeResult<User>;
pub(crate) type LookupUserFn = fn(AccessKeyRef<'_>) -> BridgeResult<Option<User>>;
pub(crate) type LookupGroupFn = fn(AccessKeyRef<'_>) -> BridgeResult<Option<UserGroup>>;
/// 1: groups, 2: key of primary group (if supported)
pub(crate) type LookupUserGroupsFn = fn(&User) -> BridgeResult<(Vec<UserGroup>, Capable<PrimaryUserGroupsCapable, AccessKey>)>;
pub(crate) type LookupGroupUsersFn = fn(&UserGroup) -> BridgeResult<Vec<User>>;
//pub(crate) type LookupAuthorityFn = fn(domain: TwoStr<'_>) -> CrossBridgeResult<Option<DomainAuthority>>;

pub struct AccessLookup {
    pub lookup_user_fn: LookupUserFn,
    pub lookup_group_fn: LookupGroupFn,
    pub lookup_user_groups_fn: LookupUserGroupsFn,
    pub lookup_group_users_fn: LookupGroupUsersFn,
    pub lookup_effective_process_user_fn: LookupEffectiveProcessUserFn,
}

