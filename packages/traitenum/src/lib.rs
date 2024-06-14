
#[cfg(feature = "macro")]
pub use asmov_common_traitenum_macro::{self as macros, enumtrait};

#[cfg(feature = "parse")]
pub use traitenum_parse as parse;

pub trait EnumTrait {
    type Iterator: ::std::iter::Iterator<Item = Self>;

    fn iter() -> Self::Iterator;
}
