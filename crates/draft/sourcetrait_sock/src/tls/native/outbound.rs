pub(crate) mod connect;

use crate::*;
pub(crate) use self::{
    connect::*,
};

pub struct OutboundTlsConfig {
    pub hostname: String,
    pub port: u16,
    pub user_hash_id: u64,
    pub cert_file: Option<PathBuf>,
}

pub struct OutboundTls {
    
}

impl OutboundTls {
    
    pub async fn query<T, U>(&self, request: T) -> SockResult<U> {
        todo!()
    }
}