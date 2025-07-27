#![doc = include_str!("../docs/DOC/1.head.md")]
#![doc = include_str!("../docs/DOC/3.foot.md")]

pub mod error;
pub mod fromto;
pub mod inlined;
pub mod pathgraph;

pub use self::{
    error::{Error, Result},
    fromto::{FromRon, ToRon},
    inlined::{
        InlinedRon, InlinedRonConfig, InlinedRonIncludeDirs, InlinedRonState,
        FromInlinedRon, ToInlinedRon, InlinedRonResolver, RonIncluded,
        IncludeRon,
    },
    pathgraph::PathGraph,
};

pub use sourcetrait_ronx_macro::RonX;

pub mod prelude {
    pub use crate::{
        FromRon, ToRon,
        inlined::{
            InlinedRon, InlinedRonResolver, InlinedRonState,
            IncludeRon, RonIncluded,
            InlinedRonConfig, InlinedRonIncludeDirs,
            FromInlinedRon, ToInlinedRon,
        }
    };
}

pub(crate) use std::{
    io,
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};
pub(crate) use sourcetrait_stdx::{
   error::fs::FsErrMsg
};
pub(crate) use hashlink::LinkedHashMap;

pub(crate) const RON: &'static str = "RON";
pub(crate) const LIL_RON: &'static str = "ron";
