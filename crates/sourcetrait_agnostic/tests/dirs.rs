use sourcetrait_testing::prelude::*;
use sourcetrait_agnostic::{self as agnostic, prelude::*};
use std::{path::Path};

static TESTING: testing::Module = testing::module!(Integration, {
    .using_temp_dir()
});

#[tested]
fn test_xdg() {
    let test = testing::test!({
        .inherit_temp_dir()
        .setup(|this| {
            unsafe {
                std::env::set_var("HOME", this.temp_dir());
                std::env::remove_var("XDG_CONFIG_HOME");
                std::env::remove_var("XDG_CACHE_HOME");
                std::env::remove_var("XDG_DATA_HOME");
                std::env::remove_var("XDG_STATE_HOME");
            }
        })
    });
    
    assert_eq!(test.temp_dir(), agnostic::PLATFORM.path().home_dir().unwrap());
    
    assert_eq!(
        test.temp_dir().join(".config/foo/bar"),
        agnostic::PLATFORM.path().xdg_subdir(agnostic::XdgDir::HomeConfig, "foo/bar").unwrap(),
    );
    
    assert_eq!(
        test.temp_dir().join(".local/share/foo/bar"),
        agnostic::PLATFORM.path().xdg_subdir(agnostic::XdgDir::HomeData, "foo/bar").unwrap(),
    );
    
    let actual_dir = agnostic::PLATFORM.path().subdir(agnostic::Dir::HomeConfig, "bar/foo").unwrap(); 
    let expected_dir = match agnostic::PLATFORM.os() {
        agnostic::Os::Linux => test.temp_dir().join(".config/bar/foo"),
        agnostic::Os::MacOS => test.temp_dir().join("Library/Application Support/bar/foo"),
        agnostic::Os::Windows => test.temp_dir().join("AppData/Roaming/bar/foo"),
        agnostic::Os::Unsupported => agnostic::CrossError::err_unsupported().unwrap(),
    };
    
    assert_eq!(expected_dir, actual_dir);
}
