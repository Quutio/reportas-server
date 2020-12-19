pub mod models;
pub mod schema;

#[macro_use]
extern crate diesel;

extern crate dotenv;

use diesel::{pg::PgConnection, insert_into};
use diesel::prelude::*;

use schema::reports;

use dotenv::dotenv;

use std::env;
use std::error::Error;

#[derive(Insertable)]
#[table_name = "reports"]
pub struct NewReport<'a> {
    pub reporter: &'a str,
    pub reported: &'a str,
    pub description: &'a str,
}

pub fn insert_report(new_report: NewReport) -> Result<(), Box<dyn Error>> {

    use schema::reports::dsl::*;

    let conn = establish_connection();

    insert_into(reports)
        .values(&new_report)
        .execute(&conn)?;

    Ok(())
}

///
/// Attempt to connect to postgres.
///
pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    PgConnection::establish(&db_url)
        .expect(&format!("An error occured while connectiong to {}", db_url))
}
