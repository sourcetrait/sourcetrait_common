pub(crate) mod cli {
    pub(crate) mod cli;
}
pub(crate) mod err {
    pub(crate) mod error;
}
pub(crate) mod parse {
    pub(crate) mod schema {
        pub(crate) mod jsonl {
            pub(crate) mod assume;
            pub(crate) mod parser;
            pub(crate) mod guess;
            pub(crate) mod model;
        }
    }
}
pub(crate) mod run;
pub(crate) mod runstate;
pub(crate) mod schema {
    pub(crate) mod config;
    pub(crate) mod json;
    pub(crate) mod sqlite;
}
pub(crate) mod stree;

pub use self::{
    cli::{
        cli::*,
    },
    err::{
        error::*,
    },
    run::{run, run_with},
    schema::{
        config::*,
        json::*,
        sqlite::*,
    },
    stree::*,
};

pub(crate) use crate::{
    parse::{
        schema::{
            jsonl::{
                assume::*,
                guess::*,
                parser::*,
                model::*,
            },
        },
    },
    runstate::*,
};

#[allow(unused_imports)]
pub(crate) use std::{
    pin::Pin,
    borrow::Cow,
    collections::HashMap,
    error::Error,
    io::{
        self,
        stdout,
        Write,
        BufReader, BufRead,
    },
    fmt::{Debug, Display},
    fs::{
        self,
        File,
    },
    path::{PathBuf, Path},
    process::ExitCode,
    sync::{atomic::AtomicU64, Arc, Mutex, OnceLock},
    cell::RefCell,
    time::Duration,
};

pub(crate) use clap::Parser;
pub(crate) use heck::ToSnakeCase;
pub(crate) use serde_json::Value;

#[allow(unused_imports)]
pub(crate) mod r {
    pub(crate) mod serde {
        pub(crate) use serde_json::json;
    }
}

