use asmov_common_traitenum_cargo::{self as cargo_traitenum, cli};
use colored::Colorize;
use clap::Parser;
use std::process;

fn main() {
    let cli = cli::Cli::parse();
    match cargo_traitenum::run(cli) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("{}{}", "[traitenum] ".red(), e);
            process::exit(1);
        }
    }
}
