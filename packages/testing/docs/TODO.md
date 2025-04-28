todo!()
======================================================

In Progress
------------------------------------------------------


Task Pool
------------------------------------------------------

# Rewrite

## Groups
Group isn't very useful in a heirarchy. It would be better to make groups
act like standalone bundles of resources and configurations.

- Remove module dependency on Group
- Modules build with `using_group(namepath)`
- Modules retrieve groups with `group(namepath)`
- Tests build with `inherit_groups()` (from the Module) and `using_group(namepath)` as a one-off
- Tests retrieve groups with `group(namepath)`
- Tests retrieve group fixture dirs with `group_fixture_dir(namepath)`
- Tests retrieve group tmp dirs with `group_tmp_dir(namepath)`

Much more useful this way.

## Const Namepaths
It's tedious to LazyLock static namepaths. It would be better to use them as const
with any locking performed internally.

## Macro
`#[testing]` implies `#[named]`

## Temp dir defaults to cargo's target dir
Another breaking change. The user should be able to select either System, Target,
or Custom(..) as a base tempdir.

## Tooling
Diffing directories is common and there should be a utility for it.

Likewise with sha256 hashing directories.

Tar without attributes/permissions should be standardized.

A bash lib for these, as such should be provided somehow, for fixture setup.

Some common uutil operations like touch, should be wrapped.
