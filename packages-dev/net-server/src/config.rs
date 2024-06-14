use serde;
use toml;
use crate::*;

pub trait Config: Sized + serde::Serialize + serde::de::DeserializeOwned {
    const FILENAME: &'static str = "server.toml";
    const INSTALL_PATH_SUFFIX: InstallPathSuffix;

    async fn load() -> LoggedResult<Self> {
        let default_config_filepath = Self::install_path().config_dir().join(Self::FILENAME);
        Self::load_file(&default_config_filepath).await
    }

    async fn load_file(cfg_filepath: &std::path::Path) -> LoggedResult<Self> {
        match read_toml(cfg_filepath).await {
            Ok(c) => Ok(c),
            Err(e) => {
                log_error!("{e}");
                Err(Error::Server(e))
            }
        }
    }

    fn install_path_suffix(&self) -> &'static InstallPathSuffix {
        &Self::INSTALL_PATH_SUFFIX
    }

    fn install_path() -> &'static InstallPath;
}

async fn read_toml<CFG: Config>(config_path: &std::path::Path) -> Result<CFG, ServerError> {
    tokio::fs::read_to_string(&config_path).await
        .map_err(|e| ServerError::fileio(e, &config_path))
        .and_then(|config_contents| Ok(toml::from_str(&config_contents)))?
        .map_err(|e| ServerError::config_file(e, &config_path))
}
