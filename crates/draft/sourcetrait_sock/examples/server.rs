mod srvclib { pub mod shared; }
use srvclib::shared::*;
use sourcetrait_sock as sock;
use std::{
    path::PathBuf
};


#[tokio::main]
async fn main() {
    let inbound_config = sock::InboundTlsConfig {
        bind_hostname: "127.0.0.1".to_string(),
        bind_port: 4032,
        cert_pem_file: PathBuf::from("/tmp/certs/cert.pem"),
        key_pem_file: PathBuf::from("/tmp/certs/cert.key.pem"),
        classes: vec![
            sock::InboundTlsClassConfig {
                languages: vec![ServiceDef::LANGUAGE_STRID.to_string()],
                ipv4_cidr: vec!["127.0.0.1/32".to_string()],
                ipv6_cidr: vec![],
            },
        ],
    };
    
    
    let authenticator = Box::new(Authenticator);
    let inbound_tls = sock::InboundTls::bind(&SERVICE, authenticator, inbound_config).await.unwrap();
    
    tokio::select!{
        accept = inbound_tls.accept() => { dbg!(accept.unwrap().unwrap().address()); },
    }
}

pub struct Authenticator;
impl sock::AuthenticatorTrait for Authenticator {
    fn lookup_user(&self, _hash_id: u64) -> std::pin::Pin<Box<dyn Future<Output = sourcetrait_sock::SockResult<Option<std::sync::Arc<sourcetrait_sock::SockUser>>>>>> {
        Box::pin(async move {
            Ok(None)
        })
    }
}
