use std::fs;
use native_tls as tls;
use crate::*;

// the identity.p12 file is a PKCS#12 file that contains the server's private key and certificate
const TLS_IDENTITY_FILENAME: &'static str = "tls_identity_p12";

pub fn load_tls_identity(password: String, suffix: InstallPathSuffix) -> Result<tls::Identity, ServerError> {
    let identity_filepath = InstallPath::from_executable(suffix)
        .config_secrets_dir().join(TLS_IDENTITY_FILENAME);
    let bytes = &fs::read(&identity_filepath)
        .map_err(|e| ServerError::FileIO{filepath: identity_filepath.to_str().unwrap().to_string(), cause: e.to_string()})?;
    tls::Identity::from_pkcs12(bytes, &password)
        .map_err(|e| ServerError::TLS(e))
}

const TLS_IDENTITY_PASSWORD_FILENAME: &'static str = "tls_passwd";

pub fn read_identity_password(path_suffix: InstallPathSuffix) -> LoggedResult<String> {
    let secrets_dir = InstallPath::from_executable(path_suffix).config_secrets_dir();
    let filepath = secrets_dir.join(TLS_IDENTITY_PASSWORD_FILENAME);
    std::fs::read_to_string(&filepath)
        .and_then(|s| Ok(s.trim().to_owned()))
        .map_err(|e| {
            log_error!("Unable to load TLS certification password from `{}`. :> {e}",
                filepath.to_str().unwrap());
            Error::Server(ServerError::fileio(e, &filepath))
        })
}


pub fn build_tls_acceptor(path_suffix: InstallPathSuffix) -> LoggedResult<tokio_native_tls::TlsAcceptor> {
    let identity = load_tls_identity(read_identity_password(path_suffix)?, path_suffix)
        .map_err(|e| {
            log_error!("Unable to load TLS identity :> {e}");
            Error::Server(e)
        })?;

    let acceptor = native_tls::TlsAcceptor::builder(identity).build()
        .map_err(|e| {
            log_error!("Unable to build TLS acceptor :> {e}");
            Error::Server(ServerError::TLS(e))
        })?;

    Ok(tokio_native_tls::TlsAcceptor::from(acceptor))
}

const CERT_DER: &'static str = "tls_cert_der";
const ROOT_CA_DER: &'static str = "tls_root_ca_der";

pub async fn load_certs(config: &impl Config) -> LoggedResult<Vec<tls::Certificate>> {
    const FILENAMES: [&'static str; 2] = [CERT_DER, ROOT_CA_DER];
    let secrets_dir = InstallPath::from_executable(config.install_path_suffix().clone()).config_secrets_dir();
    let mut certs = Vec::new();

    for filename in FILENAMES {
        let bytes = &tokio::fs::read(secrets_dir.join(filename)).await
            .map_err(|e| {
                log_error!("Failed to read certificate file {}: {}", &filename, e);
                Error::Server(ServerError::FileIO{cause: e.to_string(), filepath: filename.to_string()})
            })?;
        let cert = tls::Certificate::from_der(bytes).unwrap();
        certs.push(cert);
    }

    Ok(certs)
}

pub async fn build_tls_connector(config: &impl Config) -> LoggedResult<tokio_tungstenite::Connector> {
    let mut native_tls_connector_builder = tls::TlsConnector::builder();

    #[cfg(debug_assertions)]
    native_tls_connector_builder.danger_accept_invalid_hostnames(true);

    for cert in load_certs(config).await? {
        native_tls_connector_builder.add_root_certificate(cert);
    }

    let native_tls_connector = native_tls_connector_builder.build()
        .map_err(|e| {
            log_error!("Failed to build TLS connector: {}", e);
            Error::Server(ServerError::TLS(e))
        })?;

    Ok(tokio_tungstenite::Connector::NativeTls(native_tls_connector))
}
