use crate::*;

pub const ENV_HOME: &'static str = "HOME";
pub const ENV_PATH: &'static str = "PATH";
pub const ENV_EDITOR: &'static str = "EDITOR";

pub const CLI_EDITOR_GUESSES: [&'static str; 5] = [
    "hx",
    "nvim",
    "vim",
    "vi",
    "nano",
];

pub const WHERE_EDITOR: [GuessWhere; 2] = [
    GuessWhere::EnvVar(ENV_EDITOR),
    GuessWhere::EnvPaths
];

