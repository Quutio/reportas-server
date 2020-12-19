pub mod dao;
pub mod models;
pub mod schema;

#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::pg::PgConnection;
use diesel::prelude::*;

use dotenv::dotenv;

use std::env;

///
/// Attempt to connect to postgres.
///
pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    PgConnection::establish(&db_url)
        .expect(&format!("An error occured while connectiong to {}", db_url))
}
