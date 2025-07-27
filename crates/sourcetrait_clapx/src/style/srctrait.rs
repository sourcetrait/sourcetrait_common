//! SrcTrait's terminal color style
//! 
//! Originally based off of Cargo's styling:
//! 
//! [Source](https://github.com/crate-ci/clap-cargo/blob/master/src/style.rs)

use clap::builder::styling::{Styles, AnsiColor, Effects, Style};

pub mod styl {
    use super::*;
    
    pub const STYL_NOOP: Style = Style::new();
    pub const STYL_HEADER: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
    pub const STYL_USAGE: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
    pub const STYL_LITERAL: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
    pub const STYL_PLACEHOLDER: Style = AnsiColor::Cyan.on_default();
    pub const STYL_ERROR: Style = AnsiColor::Red.on_default().effects(Effects::BOLD);
    pub const STYL_WARN: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);
    pub const STYL_NOTE: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
    pub const STYL_GOOD: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
    pub const STYL_VALID: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
    pub const STYL_INVALID: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);
}

/// SrcTrait's terminal color style
pub const CLAP_STYLE_SOURCETRAIT: Styles = Styles::styled()
    .header(styl::STYL_HEADER)
    .usage(styl::STYL_USAGE)
    .literal(styl::STYL_LITERAL)
    .placeholder(styl::STYL_PLACEHOLDER)
    .error(styl::STYL_ERROR)
    .valid(styl::STYL_VALID)
    .invalid(styl::STYL_INVALID);
