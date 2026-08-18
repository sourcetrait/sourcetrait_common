/// Encoding / Decoding
pub(crate) mod code {
    pub(crate) mod nom;
    pub(crate) mod nonce;
}

pub use crate::{
    code::{
        nom::{Nom,NomPair,NomStr},
        nonce::{Nonce,NoncePair,NonceStr,NonceGenerator},
    },
};

pub(crate) use std::{
    path::{Path,PathBuf},
    hash::Hash,
    sync::atomic::{self, AtomicUsize},
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) mod cereal {
    pub(crate) use sourcetrait_cereal::*;
    pub(crate) use sourcetrait_cereal_macro::*;
}
