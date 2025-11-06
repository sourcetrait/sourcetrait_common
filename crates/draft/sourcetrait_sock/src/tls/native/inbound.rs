use std::{net::IpAddr, pin::Pin, str::FromStr, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::*;

pub(crate) const BUFFER_SIZE: usize = 4096;
pub(crate) const STREAM_ACCEPT_TIMEOUT: Duration = Duration::from_secs(4);

pub struct InboundTlsConfig {
    pub bind_hostname: String,
    pub bind_port: u16,
    pub cert_pem_file: PathBuf,
    pub key_pem_file: PathBuf,
    pub classes: Vec<InboundTlsClassConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SockUser {
    hash_id: u64,
    pubkey: SockPubkey,
    recovery_pubkey: SockPubkey,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SockPubkey {
    Ed(EdPubkey),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EdPubkey(pub(crate) [u8; 32]);

pub trait AuthenticatorTrait {
    fn lookup_user(&self, hash_id: u64) -> Pin<Box<dyn Future<Output = SockResult<Option<Arc<SockUser>>>>>>;
}

pub type BoxedAuthenticator = Box<dyn AuthenticatorTrait>;

pub struct InboundTls {
    pub(crate) listener: r::tokio::TcpListener,
    pub(crate) acceptor: r::tls::TlsAcceptor,
    pub(crate) authenticator: BoxedAuthenticator,
    pub(crate) service_def: &'static TlsServiceDef,
    pub(crate) classes: InboundTlsClasses,
}

pub trait TlsSystem {
    type Msg: Sized;
}

#[repr(u8)]
pub enum AcceptResponse {
    Ok = 0x0,
    /// The language requested is either unknown or not available to the user.
    LanguageUnavailable = 0x1,
    /// The major version of the language requested does not match the servers.
    /// The expected version will be passed afterwards (u32).
    VersionMismatch = 0x2,
    /// The response to the signature challenge was incorrect.
    AuthenticationFailed = 0x4,
}

impl InboundTls {
    pub async fn bind(service_def: &'static TlsServiceDef, authenticator: BoxedAuthenticator, cfg: InboundTlsConfig) -> SockResult<Self> {
        let classes = InboundTlsClasses::from_config(service_def, cfg.classes)?;
        let addr = resolve_address(&cfg.bind_hostname, cfg.bind_port).await?;
        let cert = r::tls::CertificateDer::from_pem_file(cfg.cert_pem_file)?;
        let key = r::tls::PrivateKeyDer::from_pem_file(cfg.key_pem_file)?;
        
        let server_cfg = r::tls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)?;
        
        let acceptor = r::tls::TlsAcceptor::from(Arc::new(server_cfg));
        let listener = r::tokio::TcpListener::bind(addr).await
            .map_err(|source| SockError::Tls{ e: TlsSockError::Io { source }})?;
        
        Ok(Self {
            service_def,
            authenticator,
            classes,
            listener,
            acceptor,
        })
    }
    
    pub async fn accept(&self) -> SockResult<Option<InboundTlsChannel>> {
        let (mut stream, address) = self.listener.accept().await
            .map_err(|source| SockError::Tls{ e: TlsSockError::Io { source }})?;
        
        let Some(classes) = self.classes.match_address(&address) else {
            let _ = stream.shutdown();
            return Ok(None);
        };
        
        let mut stream = self.acceptor.accept(stream).await
            .map_err(|source| SockError::Tls{ e: TlsSockError::Io { source }})?;
        
        const STREAM_TIMEOUT: Duration = STREAM_ACCEPT_TIMEOUT;
        
        macro_rules! exit_stream {
            () => {
                {
                    let _ = stream.shutdown();
                    return Ok(None);
                }
            }
        }
        
        macro_rules! read_stream {
            ($op:expr) => {
                match tokio::time::timeout(STREAM_TIMEOUT, $op).await {
                    Ok(Ok(v)) => v,
                    _ => exit_stream!(),
                }
            }
        }
        
        macro_rules! write_stream {
            ($op:expr) => {
                match tokio::time::timeout(STREAM_TIMEOUT, $op).await {
                    Ok(Ok(_)) => {},
                    _ => exit_stream!(),
                }
            }
        }
        
        // shutdown the stream quietly if:
        // - the language id doesn't exist
        // - the user doesn't exist
        
        let language_id = read_stream!(stream.read_u8());
        let Some((classes, language)) = self.classes.filter_language_id(classes, language_id) else {
            exit_stream!();
        };
        
        let user_hash_id = read_stream!(stream.read_u64());
        let Some(user) = self.authenticator.lookup_user(user_hash_id).await? else {
            exit_stream!();
        };
        
        let major_version = read_stream!(stream.read_u16());        
        
        let language = match self.classes.filter(classes, language, major_version, &user) {
            LanguageMatch::LanguageUnavailable => {
                write_stream!(stream.write_u8(AcceptResponse::LanguageUnavailable as u8));
                exit_stream!();
            }
            LanguageMatch::VersionMismatch(expected_version) => {
                write_stream!(stream.write_u8(AcceptResponse::VersionMismatch as u8));
                write_stream!(stream.write_u32(expected_version));
                exit_stream!();
            },
            LanguageMatch::Some(class) => {
                write_stream!(stream.write_u8(AcceptResponse::Ok as u8));
                class
            }
        };
        
        let challenge = 42; //todo: generate nonce
        write_stream!(stream.write_u128(challenge));
        
        let signature = read_stream!(stream.read_u64());
        
        //todo: verify signature
        
        write_stream!(stream.write_u8(AcceptResponse::Ok as u8));
        
        let buffer = Buffer::new();
        
        Ok(Some(InboundTlsChannel {
            address,
            stream,
            buffer,
            language,
        }))
    }
}

pub struct InboundTlsClass {
    pub(crate) languages: Vec<&'static Language>,
    pub(crate) ipv4_pools: Vec<subnetwork::Ipv4Pool>,
    pub(crate) ipv6_pools: Vec<subnetwork::Ipv6Pool>,
}

impl InboundTlsClass {
    pub fn has_language(&self, hashcode: u64) -> bool {
        self.languages.iter().find(|lang| lang.hashcode() == hashcode).is_some()
    }
}

pub trait LanguageLookupTrait {
    fn lookup_language_id(&self, id: u8) -> Option<&'static Language>;
    fn lookup_language_strid(&self, strid: &str) -> Option<&'static Language>;
}

pub type BoxedLanguageIndex = Box<dyn LanguageLookupTrait>;

pub struct InboundTlsClasses {
    language_index: &'static TlsServiceDef,
    classes: Vec<InboundTlsClass>
}

impl InboundTlsClasses {
    pub fn from_config(language_index: &'static TlsServiceDef, classes: Vec<InboundTlsClassConfig>) -> SockResult<Self> {
        let classes = classes.into_iter()
            .map(|cls| cls.try_into_class(language_index))
            .collect::<SockResult<Vec<InboundTlsClass>>>()?;
        
        Ok(Self {
            language_index,
            classes,
        })
    }
    
    pub fn match_address(&self, addr: &SocketAddr) -> Option<Vec<&InboundTlsClass>> {
        let ip = addr.ip();
        let results: Vec<&InboundTlsClass> = self.classes.iter().filter(|class| {
            match ip {
                IpAddr::V4(ip) => class.ipv4_pools.iter().find(|pool| pool.contains(ip)).is_some(),
                IpAddr::V6(ip) => class.ipv6_pools.iter().find(|pool| pool.contains(ip)).is_some(),
            }
        })
        .collect();
        
        match results.is_empty() {
            true => None,
            false => Some(results),
        }
    }
    
    pub fn filter_language_id<'a>(&self, class_vec: Vec<&'a InboundTlsClass>, language_id: u8) -> Option<(Vec<&'a InboundTlsClass>, &'static Language)> {
        let Some(language) = self.language_index.lookup_id(language_id) else {
            return None;
        };
            
        let class_vec: Vec<&InboundTlsClass> = class_vec.into_iter()
            .filter(|class| class.has_language(language.hashcode()))
            .collect();
        
        match class_vec.is_empty() {
            false => Some((class_vec, language)),
            true => None
        }
    }
    
    
    pub fn filter(&self, class_vec: Vec<&InboundTlsClass>, language: &'static Language, major_version: u16, user: &SockUser) -> LanguageMatch {
        todo!()
    }
}

pub struct InboundTlsClassConfig {
    pub languages: Vec<String>,
    pub ipv4_cidr: Vec<String>,
    pub ipv6_cidr: Vec<String>,
}

impl InboundTlsClassConfig {
    pub fn try_into_class(self, language_index: &'static TlsServiceDef) -> SockResult<InboundTlsClass> {
        let mut languages = Vec::with_capacity(self.languages.len());
        for strid in self.languages {
            let language = language_index.lookup_strid(&strid)
                .ok_or_else(|| SockError::Config { cause: format!("Unknown language: {strid}") })?;
            languages.push(language);
        }
        
        let mut ipv4_pools = Vec::with_capacity(self.ipv4_cidr.len());
        for pool in self.ipv4_cidr {
            let pool = subnetwork::Ipv4Pool::from_str(&pool)
                .map_err(|_| SockError::Config { cause: format!("Invalid IPv4 pool: {pool}") })?;
            ipv4_pools.push(pool);
        }
        
        let mut ipv6_pools = Vec::with_capacity(self.ipv6_cidr.len());
        for pool in self.ipv6_cidr {
            let pool = subnetwork::Ipv6Pool::from_str(&pool)
                .map_err(|_| SockError::Config { cause: format!("Invalid IPv4 pool: {pool}") })?;
            ipv6_pools.push(pool);
        }
        
        Ok(InboundTlsClass {
            languages,
            ipv4_pools,
            ipv6_pools,
        })
    }
}

pub enum LanguageMatch {
    LanguageUnavailable,
    VersionMismatch(u32), // report 1u8 (failure) and then the u32 version of the server's language
    Some(&'static Language), // report 0u8 (success)
}

pub struct InboundTlsChannel {
    pub(crate) address: SocketAddr,
    pub(crate) buffer: Buffer<BUFFER_SIZE>,
    pub(crate) language: &'static Language,
    pub(crate) stream: tokio_rustls::server::TlsStream<tokio::net::TcpStream>,
}

impl InboundTlsChannel {
    pub fn address(&self) -> &SocketAddr { &self.address }
}

pub(crate) struct PendingInboundTlsChannel<'a> {
    pub(crate) address: SocketAddr,
    pub(crate) buffer: Buffer<BUFFER_SIZE>,
    pub(crate) classes: Vec<&'a InboundTlsClass>,
    pub(crate) stream: tokio_rustls::server::TlsStream<tokio::net::TcpStream>,
}