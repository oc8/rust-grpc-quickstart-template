use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use redis::Client;
use sqlx::migrate::Migrator;
use tokio::sync::oneshot;
use tonic::transport::Server;
use protos::echo::v1::echo_service_server::EchoServiceServer;
use rust_server::database::{CacheClient, PgPool};
use rust_server::server::services::v1::echo::echo_service::EchoServiceServerImpl;
use futures_util::FutureExt;

pub mod rpcs;
pub mod fixtures;

static MIGRATIONS: Migrator = sqlx::migrate!("./migrations");

struct TestContext {
    db_url: String,
    db_name: String,
    addr: SocketAddr,
    url: String,
    service: EchoServiceServerImpl,
}

// TODO: Add mock redis server
impl TestContext {
    async fn new(db_url: &str, db_name: &str, r_url: &str, port: u16) -> Self {
        dotenvy::dotenv().ok();
        let pool = PgPool::connect(&format!("{}/postgres", db_url)).await.expect("Cannot connect to postgres database.");

        let query = format!("DROP DATABASE IF EXISTS {}", db_name);
        sqlx::query(&query).execute(&pool).await.expect(&format!("Could not drop database {}", db_name));

        let query = format!("CREATE DATABASE {}", db_name);
        sqlx::query(&query).execute(&pool).await.expect(&format!("Could not create database {}", db_name));

        let pool = Arc::new(PgPool::connect(&format!("{}/{}", db_url, db_name)).await.expect("Cannot connect to new database"));

        MIGRATIONS.run(pool.as_ref()).await.expect("Failed to run migrations");

        let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
        let url = format!("http://{}:{}", addr.ip(), addr.port());

        let r_client = Client::open(r_url)
            .expect("Cannot connect to redis server");

        let cache_ttl = env::var("CACHE_TTL").unwrap_or_else(|_| "60".to_string()).parse::<u64>().expect("CACHE_TTL must be a number");
        let cache_client = CacheClient::new(r_client, cache_ttl);

        Self {
            db_url: db_url.to_string(),
            db_name: db_name.to_string(),
            addr,
            url,
            service: EchoServiceServerImpl { pool: pool.clone(), cache: cache_client.clone() },
        }
    }

    async fn mock_database<F, Fut>(&self, mut f: F)
    where
        F: FnMut(&PgPool) -> Fut,
        Fut: std::future::Future<Output = ()>
    {
        f(&self.service.pool).await;
    }

    async fn cleanup(&self) {
        let pool = PgPool::connect(&format!("{}/postgres", self.db_url)).await.expect("Cannot connect to postgres database.");

        let disconnect_users = format!(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}';",
            self.db_name
        );

        sqlx::query(&disconnect_users).execute(&pool).await.unwrap();

        let query = format!("DROP DATABASE {}", self.db_name);
        sqlx::query(&query).execute(&pool).await.expect(&format!("Couldn't drop database {}", self.db_name));
    }
}

async fn setup_test_context(name: &str, port: u16) -> (TestContext, oneshot::Sender<()>, tokio::task::JoinHandle<()>) {
    let ctx = TestContext::new(
        "postgres://postgres:postgres@localhost:5432",
        name,
        "redis://:@localhost:6380",
        port,
    ).await;
    let (tx, rx) = oneshot::channel();
    let echo_service = ctx.service.clone();

    let jh = tokio::spawn(async move {
        Server::builder()
            .add_service(EchoServiceServer::new(echo_service))
            .serve_with_shutdown(ctx.addr, rx.map(|_| ()))
            .await
            .unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    (ctx, tx, jh)
}