use sqlx::postgres::PgPoolOptions;
pub use sqlx::{
    Error,
    Pool,
    Postgres,
};

pub async fn create_connection_pool(connection_string: &str) -> Result<Pool<Postgres>, Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(connection_string)
        .await
}
