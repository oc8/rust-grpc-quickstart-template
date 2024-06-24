use std::env;
use std::sync::Arc;

use axum::{routing::get, Router};
use autometrics::prometheus_exporter;
use dotenvy::dotenv;
use rust_server::{create_socket_addr, database, init_service_logging};
use rust_server::database::CacheClient;
use rust_server::server::start_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    init_service_logging();
    prometheus_exporter::init();

    // Set up the database connection
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    database::check_for_migrations(&database_url.clone())
        .await
        .expect("An error occurred while running migrations");

    let pool = database::connect(&database_url)
        .await
        .expect("Couldn't connect to the database");

    // Set up the Redis connection
    let uri_scheme = match env::var("REDIS_TLS").unwrap_or_default().parse::<bool>() {
        Ok(bool) => if bool { "rediss" } else { "redis" },
        Err(_) => "redis",
    };

    let redis_host = env::var("REDIS_HOSTNAME").expect("REDIS_HOSTNAME must be set");
    let redis_pass = env::var("REDIS_PASSWORD").unwrap_or_default();
    let redis_conn_url = format!("{}://:{}@{}", uri_scheme, redis_pass, redis_host);
    let r_client = database::connect_redis(redis_conn_url)
        .expect("Couldn't connect to Redis");

    // Start the gRPC server
    let port = env::var("PORT").unwrap_or_else(|_| "50051".to_string()).parse().expect("PORT must be a number");
    let cache_ttl = env::var("CACHE_TTL").unwrap_or_else(|_| "60".to_string()).parse::<u64>().expect("CACHE_TTL must be a number");

    let cache_client = CacheClient::new(r_client, cache_ttl);

    let _server = start_server(
        Arc::new(pool),
        cache_client,
        port
    );

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