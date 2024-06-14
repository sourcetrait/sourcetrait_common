
pub mod config;
pub mod connection;
pub mod error;
pub mod log;
pub mod looper;
pub mod message;
pub mod protocol;
pub mod runtime;
pub mod state;
pub mod run;
pub mod stream;
pub mod tls;

pub use config::*;
pub use connection::*;
pub use error::*;
pub use log::*;
pub use looper::*;
pub use message::*;
pub use protocol::*;
pub use runtime::*;
pub use state::*;
pub use run::*;
pub use stream::*;
pub use tls::*;

pub use asmov_common_linux::installpath::{InstallPath, InstallPathSuffix};

/* turn this into a macro or something for the bin file
const CONFIG_DIRPATH_SUFFIX: &'static str = "asmov/se/world";
pub(crate) const PATH_SUFFIX: asmov_common_linux::installpath::InstallPathSuffix = asmov_common_linux::installpath::InstallPathSuffix {
    config_dir: CONFIG_DIRPATH_SUFFIX,
    data_dir: CONFIG_DIRPATH_SUFFIX,
};

pub(crate) fn install_path() -> server_common::InstallPath {
    server_common::InstallPath::from_executable(PATH_SUFFIX)
}*/
