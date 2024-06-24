use std::env;
use std::net::{Ipv6Addr, SocketAddr, SocketAddrV6};
use ::log::{error, info};

pub mod server;
pub mod database;
pub mod errors;
pub mod utils;

pub fn init_service_logging() {
    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .parse_env("RUST_LOG")
        .init();
}

pub fn report_error<E: 'static>(err: &E)
where
    E: std::error::Error,
    E: Send + Sync,
{
    let mut stack = String::from("\n");
    if let Some(cause) = err.source() {
        for (i, e) in std::iter::successors(Some(cause), |e| e.source()).enumerate() {
            stack.push_str(&format!("   {}: {}\n", i, e));
        }
    }
    error!("[ERROR] {}\nCaused by: {}", err, stack);
}

pub fn create_socket_addr(port: u16) -> SocketAddr {
    let is_ipv6 = env::var("ENABLE_IPV6").unwrap_or_default().parse::<bool>().unwrap_or(false);

    if is_ipv6 {
        info!("Using IPv6");
        SocketAddr::from(SocketAddrV6::new(
            Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0),
            port,
            0,
            0,
        ))
    } else {
        info!("Using IPv4");
        SocketAddr::from(([0, 0, 0, 0], port))
    }
}
