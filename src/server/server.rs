use std::env;
use std::sync::Arc;

use ::log::{info, warn};
use tokio::task::JoinHandle;
use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};
use protos::echo::v1::echo_service_server::EchoServiceServer;
use crate::database::{CacheClient, PgPool};
use crate::{create_socket_addr, report_error};
use crate::server::services::v1::echo::echo_service::EchoServiceServerImpl;

pub struct TonicServer {
    pub handle: JoinHandle<()>,
    pub tls: bool,
}

pub fn start_server(
    pool: Arc<PgPool>,
    cache_client: CacheClient,
    port: u16,
) -> Result<TonicServer, Box<dyn std::error::Error>> {
    let echo = EchoServiceServerImpl::new(pool, cache_client);

    let (mut tonic_server, secure_mode) = match get_tls_config() {
        Some(tls) => {
            info!("Configuring TLS...");
            match Server::builder().tls_config(tls) {
                Ok(server) => {
                    info!("TLS successfully configured.");
                    (server, true)
                }
                Err(details) => {
                    info!("Error configuring TLS. Connections are not secure.");
                    report_error(&details);
                    (Server::builder(), false)
                }
            }
        }
        _ => {
            warn!("No TLS keys available. Connections are not secure.");
            (Server::builder(), false)
        }
    };

    let reflect = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(protos::echo::v1::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    let tonic_router = tonic_server
        .add_service(reflect)
        .add_service(EchoServiceServer::new(echo));

    let server = tokio::spawn(async move {
        let tonic_addr = create_socket_addr(port);
        info!("Starting server on port {}", port);
        match tonic_router.serve(tonic_addr).await {
            Ok(_) => info!("Server finished on {}", tonic_addr),
            Err(e) => {
                warn!("Unable to start server on port {}", port);
                report_error(&e);
            }
        };
        ()
    });

    Ok(TonicServer {
        handle: server,
        tls: secure_mode,
    })
}

fn get_tls_config() -> Option<ServerTlsConfig> {
    let cert = env::var("TLS_CERT").ok();
    let key = env::var("TLS_KEY").ok();
    let ca_cert = env::var("CA_CERT").ok();

    match (cert, key, ca_cert) {
        (Some(cert), Some(key), Some(ca_cert)) => {
            info!("Configuring TLS with custom CA...");
            Some(
                ServerTlsConfig::new()
                    .identity(Identity::from_pem(cert, key))
                    .client_ca_root(Certificate::from_pem(ca_cert)),
            )
        }
        (Some(cert), Some(key), None) => {
            info!("Configuring TLS with official CAs...");
            Some(ServerTlsConfig::new().identity(Identity::from_pem(cert, key)))
        }
        _ => None,
    }
}
