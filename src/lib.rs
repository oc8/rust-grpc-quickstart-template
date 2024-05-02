use std::env;
use std::net::{Ipv6Addr, SocketAddr, SocketAddrV6};
use rand::Rng;
use redis::{Commands, RedisResult};
use ::log::{error, info};

pub fn init_service_logging() {
    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .parse_env("RUST_LOG")
        .init();
}

pub fn env_var(name: &str) -> Option<String> {
    env::var(name).ok().filter(|s| !s.is_empty())
}

pub fn report_error<E: 'static>(err: E)
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

pub fn store_token(conn: &mut redis::Connection, token: &str, expiration_seconds: usize) -> RedisResult<()> {
    conn.set_ex(token, expiration_seconds, expiration_seconds as u64)?;
    Ok(())
}

pub fn is_token_valid(conn: &mut redis::Connection, token: &str) -> RedisResult<bool> {
    let exists: bool = conn.exists(token)?;
    Ok(!exists)
}

pub fn generate_secret() -> String {
    let mut secret_key = vec![0u8; 20];
    rand::thread_rng().fill(&mut secret_key[..]);

    let base32_secret = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &secret_key);

    base32_secret
}