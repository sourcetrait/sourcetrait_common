use crate::*;

pub fn run_error_srctrait(e: impl Error) -> std::process::ExitCode {
    use crate::styles::srctrait::*;
    let source = e.source()
        .map_or(String::new(), |s| format!("\n       {s}"));

    eprintln!("{CLAPX_ERROR}error:{CLAPX_ERROR:#} {e}{source}");
    std::process::ExitCode::FAILURE
}