use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

pub type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

pub fn establish_pool(database_url: String) -> PgPool {
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool")
}
