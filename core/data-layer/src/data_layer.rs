use std::path::Path;

use sqlx::{
    mysql::MySqlPoolOptions, postgres::PgPoolOptions, sqlite::SqlitePoolOptions, MySql, Pool,
    Postgres, Sqlite,
};

#[derive(Debug)]
pub enum DbPool {
    Postgres(Pool<Postgres>),
    Sqlite(Pool<Sqlite>),
    MySql(Pool<MySql>),
}

#[derive(Debug)]
pub struct DataLayer {
    // Pool is a connection pool to the database. Postgres, Mysql, SQLite supported.
    pub pool: DbPool,
}

pub struct DataLayerNewParam {
    pub sql_url: String,
}
impl DataLayer {
    pub async fn new(params: DataLayerNewParam) -> Self {
        let data_layer = DataLayer {
            pool: DataLayer::get_pool(&params.sql_url).await,
        };
        data_layer.migrate().await;
        return data_layer;
    }
    async fn get_pool(sql_url: &str) -> DbPool {
        let pool = if sql_url.starts_with("postgres://") {
            let options = PgPoolOptions::new()
                .max_connections(5)
                .connect(sql_url)
                .await
                .unwrap();
            DbPool::Postgres(options)
        } else if sql_url.starts_with("sqlite://") {
            // Create the SQLite file and directories if they don't exist
            let path = Path::new(sql_url.trim_start_matches("sqlite://"));
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            // Create the SQLite file if it doesn't exist
            if !path.exists() {
                std::fs::File::create(&path);
            }

            let options = SqlitePoolOptions::new()
                .max_connections(5)
                .connect(sql_url)
                .await
                .unwrap();
            DbPool::Sqlite(options)
        } else if sql_url.starts_with("mysql://") {
            let options = MySqlPoolOptions::new()
                .max_connections(5)
                .connect(sql_url)
                .await
                .unwrap();
            DbPool::MySql(options)
        } else {
            panic!("Unsupported database type");
        };
        println!("pool: {:?}", pool);
        return pool;
    }
    async fn migrate(&self) {
        match &self.pool {
            DbPool::Postgres(pool) => {
                sqlx::migrate!("./migrations/postgres")
                    .run(pool)
                    .await
                    .unwrap();
            }
            DbPool::Sqlite(pool) => {
                sqlx::migrate!("./migrations/sqlite")
                    .run(pool)
                    .await
                    .unwrap();
            }
            DbPool::MySql(pool) => {
                sqlx::migrate!("./migrations/mysql")
                    .run(pool)
                    .await
                    .unwrap();
            }
        }
    }
}
