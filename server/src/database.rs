use bb8_postgres::bb8::{Pool, PooledConnection, RunError};
use bb8_postgres::PostgresConnectionManager;
use bb8_postgres::tokio_postgres::{NoTls, Error};
use warp::Filter;
use std::convert::Infallible;

pub type Connection<'a> = PooledConnection<'a, PostgresConnectionManager<NoTls>>;

#[derive(Clone)]
pub struct Database {
    db: Pool<PostgresConnectionManager<NoTls>>,
}

impl Database {
    pub async fn new(url: &str) -> Result<Self, Error> {
        let database_manager = PostgresConnectionManager::new_from_stringlike(url, NoTls)?;

        Ok(Self {
            db: Pool::builder().build(database_manager).await?,
        })
    }

    pub async fn get<'a>(&'a self) -> Result<Connection<'a>, RunError<Error>> {
        self.db.get().await
    }

    pub fn with(&self) -> impl Filter<Extract = (Self,), Error = Infallible> + Clone {
        let this = self.clone();
        warp::any().map(move || this.clone())
    }
}
