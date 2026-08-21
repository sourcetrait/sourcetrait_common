//! SrcTrait's terminal color style
//! 
//! Originally based off of Cargo's styling:
//! 
//! [Source](https://github.com/crate-ci/clap-cargo/blob/master/src/style.rs)

use clap::builder::styling::{Styles, AnsiColor, Effects, Style};

pub const CLAPX_NOOP: Style = Style::new();
pub const CLAPX_HEADER: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
pub const CLAPX_USAGE: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
pub const CLAPX_LITERAL: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
pub const CLAPX_PLACEHOLDER: Style = AnsiColor::Cyan.on_default();
pub const CLAPX_ERROR: Style = AnsiColor::Red.on_default().effects(Effects::BOLD);
pub const CLAPX_WARN: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);
pub const CLAPX_NOTE: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
pub const CLAPX_GOOD: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
pub const CLAPX_VALID: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
pub const CLAPX_INVALID: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);

/// SrcTrait's terminal color style
pub const STYLE_SOURCETRAIT: Styles = Styles::styled()
    .header(CLAPX_HEADER)
    .usage(CLAPX_USAGE)
    .literal(CLAPX_LITERAL)
    .placeholder(CLAPX_PLACEHOLDER)
    .error(CLAPX_ERROR)
    .valid(CLAPX_VALID)
    .invalid(CLAPX_INVALID);
