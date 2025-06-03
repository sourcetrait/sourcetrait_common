pub mod cargo;
pub mod srctrait;

pub use self::{
    cargo::CLAP_STYLE_CARGO,
    srctrait::CLAP_STYLE_SRCTRAIT,
};

pub mod styl {
    pub use super::{cargo::styl as cargo, srctrait::styl as srctrait};
}