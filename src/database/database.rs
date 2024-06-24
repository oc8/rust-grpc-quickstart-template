use std::env;
use std::time::Duration;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Connection, PgConnection, Pool, Postgres};
use sqlx::migrate::MigrateDatabase;
use tonic::Status;

pub type PgPooledConnection = sqlx::pool::PoolConnection<Postgres>;
pub type PgPool = Pool<Postgres>;

pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    log::info!("Connecting to database: {}", database_url);

    let pool = PgPoolOptions::new()
        .min_connections(
            env::var("DATABASE_MIN_CONNECTIONS")
                .ok()
                .and_then(|x| x.parse().ok())
                .unwrap_or(0),
        )
        .max_connections(
            env::var("DATABASE_MAX_CONNECTIONS")
                .ok()
                .and_then(|x| x.parse().ok())
                .unwrap_or(16),
        )
        .max_lifetime(Some(Duration::from_secs(60 * 60)))
        .connect(database_url)
        .await?;

    Ok(pool)
}

pub async fn check_for_migrations(database_url: &str) -> Result<(), sqlx::Error> {
    if !Postgres::database_exists(database_url).await? {
        log::info!("Database does not exist, creating it...");
        Postgres::create_database(database_url).await?;
    }

    log::info!("Running migrations...");

    let mut conn: PgConnection = PgConnection::connect(database_url).await?;

    sqlx::migrate!()
        .run(&mut conn)
        .await
        .expect("Failed to run migrations");

    Ok(())
}

pub async fn get_connection(pool: &PgPool) -> Result<PgPooledConnection, Status> {
    pool.acquire().await.map_err(|_| Status::internal("Failed to acquire connection"))
}
