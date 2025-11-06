use crate::*;

/// Describes an IO error that was either cleanly recovered from or not.
/// A clean recovery removed any file that was created.
/// A dirty recovery failed to remove any file that was created. 
#[derive(Debug, snafu::Snafu)]
pub enum CopyError {
    #[snafu(display("Failed to copy {{\n  from: {src}\n  to: {dst}\n  err: {source}\n}}",
        src = src.to_string_lossy(), dst = dst.to_string_lossy()
    ))]
    Clean {
        src: PathBuf,
        dst: PathBuf,
        source: io::Error
    },
    #[snafu(display("Failed to copy {{\n  from: {src}\n  to (dirty): {dst}\n  err: {source}\n}}",
        src = src.to_string_lossy(), dst = dst.to_string_lossy()
    ))]
    Dirty {
        src: PathBuf,
        dst: PathBuf,
        source: io::Error,
        recover_source: io::Error,
    }
}

impl CopyError {
    pub fn clean<P1: AsRef<Path>, P2: AsRef<Path>>(src: P1, dst: P2, source: io::Error) -> Self {
        Self::Clean { src: src.as_ref().into(), dst: dst.as_ref().into(), source }
    }
}
