use tokio;

pub trait ServerConfig: Sized {
    fn load() -> crate::LoggedResult<Self>;
    fn read_toml(config_path: &std::path::Path) -> Result<Self, crate::ServerError>;
}

pub trait ServerState: Sized {
    type Error: std::error::Error + Send + Sync + 'static;
    type Config: Sized;
    type Sync: Sized + Send + Sync + 'static;

    fn init(config: Self::Config) -> crate::LoggedResult<Self>;
    fn config(&self) -> &Self::Config;
    fn tick_interval(&self) -> tokio::time::Duration;
    fn tick(&mut self) -> impl std::future::Future<Output=Result<(), Self::Error>>;
}

