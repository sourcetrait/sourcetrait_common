use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use filetime::FileTime;

/// Performs a coreutils touch on a file, creating it if necessary, and setting
/// its modified time if provided.
pub fn touch_file(path: &Path, datetime: Option<DateTime<Utc>>) {
    let source = match datetime {
        Some(dt) => {
            uu_touch::Source::Timestamp(FileTime::from_unix_time(dt.timestamp(), dt.timestamp_subsec_nanos()))
        },
        None => uu_touch::Source::Now
    };

    uu_touch::touch(
        &[uu_touch::InputFile::Path(PathBuf::from(path))],
        &uu_touch::Options {
            no_create: false,
            no_deref: false,
            source,
            date: None,
            change_times: uu_touch::ChangeTimes::Both,
            strict: true,
        }
    ).unwrap();
}
