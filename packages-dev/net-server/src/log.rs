use chrono::{self, Timelike, Datelike};

pub enum Log {
    Standard,
    Error,
    Warning,
    Debug,
}

impl std::fmt::Display for Log {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standard => write!(f, " "),
            Self::Error => write!(f, " ERROR "),
            Self::Warning => write!(f, " WARNING "),
            Self::Debug => write!(f, " DEBUG "),
        }
    }
}

pub fn log(log: Log, output: &str) {
    let now = chrono::offset::Local::now();
    let prefix = format!("[{}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}]{log}",
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
        now.timestamp_subsec_millis());

    match log {
        Log::Standard | Log::Debug => println!("{prefix}{output}"),
        Log::Error | Log::Warning => eprintln!("{prefix}{output}"),
    }
}

#[macro_export]
macro_rules!log {
    ($($arg:tt)*) => {
        $crate::log($crate::Log::Standard, &format!($($arg)*))
    }
}

#[macro_export]
macro_rules!log_error {
    ($($arg:tt)*) => {
        $crate::log($crate::Log::Error, &format!($($arg)*))
    }
}

#[macro_export]
macro_rules!try_log_error {
    ($result:ident, $($arg:tt)*) => {
        match $result {
            Ok(t) => Ok(t),
            Err(e) => {
                $crate::log($crate::Log::Error, &format!($($arg)*));
                Err(e)
            }
        }
    }
}
