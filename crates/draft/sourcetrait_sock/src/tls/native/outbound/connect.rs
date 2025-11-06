use crate::*;

impl OutboundTls {
    pub async fn connect(
        service_def: &'static TlsServiceDef,
        indexed_language: &'static IndexedLanguage,
        cfg: OutboundTlsConfig
    ) -> SockResult<Self> {
        let addr = resolve_address(&cfg.hostname, cfg.port).await?;
        
        let mut certs = rustls::RootCertStore::empty();
        if let Some(cert_file) = cfg.cert_file {
            for cert in r::tls::CertificateDer::pem_file_iter(cert_file)? {
                certs.add(cert?)?;
            }
        } else {
            certs.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        }
        
        
        todo!()
    }
}