Asmov Common Traitenum
================================================================================
[![Latest Version]][crates.io]

[Latest Version]: https://img.shields.io/crates/v/asmov-common-traitenum.svg
[crates.io]: https://crates.io/crates/asmov-common-traitenum

*Library for using fieldless enums as schema definitions*

In short, a trait is declared with a configurable set of const properties,   using attributes for each method. An enum is then derived for that trait, with each variant filling in property values via attributes.

Relations (`Rel`) between traits / enums can be defined with a `nature` of:
- `OneToOne`  - Points to to a single variant of another enum
- `OneToMany` - Provides an iterator over another enum's variants
- `ManyToOne` -Points to a single variant of another enum

Supported types include:
- `Str` (static)
- `Num` (usize, i64, f32, etc.)
- `Enum`
- `Bool`

Default implementations for trait methods can be used to extend functionality.

Each method signature must properly correspond with its attribute. On the other hand, attributes can be elided from method signatures, either partially or completely. `Num`, for example, uses the method signature to determine what specific type of primitive to support (f64, u8, etc.).

Properties support defaults and presets.

Presets set a default value for a property in a pre-determined way:
- `Str` converts the variant name (snake case, kebab case, etc.)
- `Num` converts the ordinal of the variant (offset, increment, etc.)

Both default and preset values can be overridden by each enum variant.

Relationships require a method signature to return:
- `OneToOne` and `ManyToOne`
  + `-> Box<dyn OtherTrait>`
- `OneToMany`
  + `-> Box<dyn Iterator<Item = Box<dyn OtherTrait>>>`

Example
-------

```rust
// ... my-trait-crate/src/lib.rs ...

#[enumtrait(crate::MyParentTrait)]
pub trait MyParentTrait {
    #[enumtrait::Str()]
    pub fn alias(&self) -> &'static str;

    #[enumtrait::Num(preset(Serial), start(1), increment(100))]
    pub fn index(&self) -> usize;

    #[enumtrait::Rel(nature(OneToMany))]
    pub fn children(&self) -> Box<dyn Iterator<Item = Box<dyn MyChildTrait>>>;
}

#[enumtrait(crate::MyChildTrait)]
pub trait MyChildTrait {
    #[enumtrait::Str(preset(Kebab))]
    pub fn kebab_name(&self) -> &'static str;

    #[enumtrait::Num()]
    pub fn column(&self) -> i32;

    #[enumtrait::Rel(nature(ManyToOne))]
    pub fn parent(&self) -> Box<dyn MyParentTrait>;
}

// ... my-enum-crate/src/lib.rs ...

#[derive(MyParentTrait)]
pub enum MyParentEnum {
    #[traitenum(alias("Uno"), children(MyFirstChildEnum))]
    First,
    #[traitenum(alias("Dos"), children(MySecondChildEnum))]
    Second,
}

#[derive(MyChildTrait)]
#[traitenum(parent(MyParentEnum::First))]
pub enum MyFirstChildEnum {
    #[traitenum(column(1))]
    AlphaBravo,
    #[traitenum(column(3))]
    CharlieDelta,
}

#[derive(MyChildTrait)]
#[traitenum(parent(MyParentEnum::Second))]
pub enum MySecondChildEnum {
    #[traitenum(column(2))]
    EchoFoxtrot,
    #[traitenum(column(4))]
    GolfHotel 
}

// ... testing ...

assert_eq!(1, MyFirstChildEnum::AlphaBravo.column())
assert_eq!("echo-foxtrot", MySecondChildEnum::EchoFoxtrot.kebab_name())
assert_eq!("Uno", MyFirstChildEnum::CharlieDelta.parent().alias())
assert_eq!(2, MyParentEnum::Second.children().nth(0).unwrap().column())
```

Packages
--------
- [traitenum](../traitenum-macro) : Macros used to define traitenum traits
- [traitenum-parse](../traitenum-parse) : Parsing library used by traitenum macros
- [cargo-traitneum](../traitenum-cargo) : Cargo addon that initializes traitenum workspaces and adds / removes traitenum traits


Repository
--------------------------------------------------------------------------------
Contributors, please review [ASMOV.md](./ASMOV.md).  

Found a bug? Search for an existing issue on GitHub.  
If an issue exists, chime in to add weight to it.  
If an issue does not exist, create one and tell us how to reproduce the bug. 


License (AGPL3)
--------------------------------------------------------------------------------
Asmov Common Traitenum: Library for using fieldless enums as schema definitions  
Copyright (C) 2024-2025 Asmov LLC  

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as
published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a [copy](./LICENSE-AGPL-3.txt) of the
GNU Affero General Public License along with this program.
If not, see https://www.gnu.org/licenses/.
