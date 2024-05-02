use std::env;
use std::sync::Arc;

use axum::{routing::get, Router};
use autometrics::prometheus_exporter;
use dotenvy::dotenv;
use redis::Client;
use rust_server::{create_socket_addr, init_service_logging};
use crate::server::start_server;

mod services;
mod validations;
mod server;
mod database;
mod rpcs;
mod errors;
#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    init_service_logging();
    prometheus_exporter::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = Arc::new(database::establish_pool(database_url));

    pool
        .get()
        .expect("Couldn't get a connection from the pool");

    let port = env::var("PORT").unwrap_or_else(|_| "50051".to_string()).parse().expect("PORT must be a number");

    let uri_scheme = match env::var("REDIS_TLS").unwrap_or_default().parse::<bool>() {
        Ok(bool) => if bool { "rediss" } else { "redis" },
        Err(_) => "redis",
    };

    let redis_host = env::var("REDIS_HOSTNAME").expect("REDIS_HOSTNAME must be set");
    let redis_pass = env::var("REDIS_PASSWORD").unwrap_or_default();
    let redis_conn_url = format!("{}://:{}@{}", uri_scheme, redis_pass, redis_host);
    let r_client = Client::open(redis_conn_url)?;

    let _server = start_server(pool.clone(), r_client, port);

    let app = Router::new().route(
        "/metrics",
        get(|| async { prometheus_exporter::encode_http_response() }),
    );

    let metrics_port: u16 = env::var("METRICS_PORT").unwrap_or_else(|_| "3000".to_string()).parse().expect("METRICS_PORT must be a number");
    let metrics_addr = create_socket_addr(metrics_port);
    let listener = tokio::net::TcpListener::bind(metrics_addr).await.unwrap();
    log::info!("Metrics server listening on port {}", metrics_port);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}