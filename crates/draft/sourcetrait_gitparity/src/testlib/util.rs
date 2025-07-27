use std::sync::{Arc, LazyLock};
use chrono::{DateTime, TimeZone, Utc};
use crate::{self as gitparity};

pub const TIME_INIT: LazyLock<DateTime<Utc>> = LazyLock::new(||
    Utc.with_ymd_and_hms(2025, 07, 04, 2, 30, 00).unwrap()
);

pub const TIME_STEP_1: LazyLock<DateTime<Utc>> = LazyLock::new(||
    Utc.with_ymd_and_hms(2025, 07, 05, 2, 30, 00).unwrap()
);

pub const TIME_STEP_2: LazyLock<DateTime<Utc>> = LazyLock::new(||
    Utc.with_ymd_and_hms(2025, 07, 06, 2, 30, 00).unwrap()
);

pub const TIME_STEP_3: LazyLock<DateTime<Utc>> = LazyLock::new(||
    Utc.with_ymd_and_hms(2025, 07, 07, 2, 30, 00).unwrap()
);

pub const USERNAME_TESTER_1: &'static str = "tester-1";
pub const EMAIL_TESTER_1: &'static str = "tester-1@test.tld";
pub const USERNAME_TESTER_2: &'static str = "tester-2";
pub const EMAIL_TESTER_2: &'static str = "tester-2@test.tld";

pub static USER_TESTER_1: LazyLock<Arc<gitparity::GitUser>> = LazyLock::new(||
    Arc::new(gitparity::GitUser::new(USERNAME_TESTER_1.to_string(), EMAIL_TESTER_1.to_string()))
);

pub static USER_TESTER_2: LazyLock<Arc<gitparity::GitUser>> = LazyLock::new(||
    Arc::new(gitparity::GitUser::new(USERNAME_TESTER_2.to_string(), EMAIL_TESTER_2.to_string()))
);

pub fn build_git_env(user: &'static gitparity::GitUser, time: DateTime<Utc>) -> gitparity::GitEnv {
    gitparity::GitEnv::builder()
        .author_name(user.name().into())
        .author_email(user.email().into())
        .author_date(time)
        .committer_name(user.name().into())
        .committer_email(user.email().into())
        .committer_date(time)
        .build()
        .expect("built")
}

pub const NEXT: &'static str = "next";

pub fn _var_for_test_username(username: &str) -> &'static str {
    match username {
        USERNAME_TESTER_1 => "USER_TESTER_1",
        USERNAME_TESTER_2 => "USER_TESTER_2",
        _ => panic!("Unknown test username in testlib macro: {username}"),
    }
}

pub fn _var_for_test_time(time: DateTime<Utc>) -> &'static str {
    if time == *TIME_INIT {
        "TIME_INIT"
    } else if time == *TIME_STEP_1 {
        "TIME_STEP_1"
    } else if time == *TIME_STEP_2 {
        "TIME_STEP_2"
    } else {
        panic!("Unknown test username in testlib macro: {time}");
    }
}

#[macro_export]
macro_rules! assert_expected_commits {
    ($git:expr, $commits:tt) => {
        let log = $git.log_with(gitparity::LogOptions {
            show_message: true,
            ..Default::default()
        }).unwrap();
        
        let expected: Vec<&Commit> = $commits.iter().map(|c| &***c).collect();
        let actual: Vec<&Commit> = log.commits().iter().map(|c| c).collect();
        assert_eq!(expected, actual);
    };
}
    
#[macro_export]
macro_rules! dbg_last_commit {
    ($git:expr) => { dbg_commit!($git, 0) };
}

#[macro_export]
macro_rules! dbg_commit {
    ($git:expr, $n:literal) => {
        {
            let log = $git.log_with(LogOptions {
                show_message: true,
                ..Default::default()
            }).unwrap();
            
            let c = log.commits().get($n).unwrap();
            
            let author_var = $crate::testlib::util::_var_for_test_username(c.author().name());
            let committer_var = $crate::testlib::util::_var_for_test_username(c.committer().name());
            let author_time_var = $crate::testlib::util::_var_for_test_time(c.author_time());
            let committer_time_var = $crate::testlib::util::_var_for_test_time(c.committer_time());
            
            println!("LOGGING LAST COMMIT:");
            println!("static LAST_COMMIT: LazyLock<gitparity::Commit> = LazyLock::new(|| gitparity::Commit {{");
            println!("    commit_hash: gitparity::GitHash::from_str(\"{}\").unwrap(),", c.commit_hash());
            println!("    tree_hash: gitparity::GitHash::from_str(\"{}\").unwrap(),", c.tree_hash());
            println!("    parent_hashes: vec![");
            println!("        // todo: convert if possible. eg: COMMIT_SOMETHING.commit_hash.clone()");
            for parent_hash in c.parent_hashes() {
                println!("        gitparity::GitHash::from_str(\"{parent_hash}\").unwrap(),");
            }
            println!("    ],");
            println!("    author: Arc::clone(&{author_var}),");
            println!("    author_time: *{author_time_var},");
            println!("    committer: Arc::clone(&{committer_var}),");
            println!("    committer_time: *{committer_time_var},");
            println!("    signature_fingerprint: {},",
                c.signature_fingerprint().map_or("None".into(), |m| format!("Some(\"{m}\".into())")));
            println!("    message: {},",
                c.message().map_or("None".into(), |m| format!("Some(\"{m}\".into())")));
            println!("}});");
        }
    };
}

#[macro_export]
macro_rules! dbg_commit_log {
    ($git:expr, $title:literal) => {
        let logs = $git.log_with(LogOptions { show_message: true, ..Default::default() }).unwrap();
        let lines: Vec<&str> = logs.commits().iter().map(|c| c.message().unwrap()).collect();
        println!("[DBG] COMMITS for '{}': {:#?}", $title, lines);
    };
}