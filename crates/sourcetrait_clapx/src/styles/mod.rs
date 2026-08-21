pub mod cargo;
pub mod srctrait;

pub use self::{
    cargo::CLAP_STYLE_CARGO,
    srctrait::STYLE_SOURCETRAIT,
};

pub mod style {
    pub use super::{cargo::styl as cargo, srctrait};
}