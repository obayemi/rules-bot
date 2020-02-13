use diesel::r2d2::{ Pool, ConnectionManager, PoolError };
use diesel::pg::PgConnection;
use serenity::prelude::{TypeMapKey};
use std::sync::Arc;
use std::env;

type PgPool = Pool<ConnectionManager<PgConnection>>;
// type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

use dotenv::dotenv;

fn init_pool(database_url : &str) -> Result<PgPool, PoolError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().build(manager)
}

pub fn establish_connection() -> PgPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    init_pool(&database_url).unwrap()
}

pub struct DbKey;
impl TypeMapKey for DbKey {
    type Value = Arc<PgPool>;
}