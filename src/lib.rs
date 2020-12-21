pub mod models;
pub mod schema;

#[macro_use]
extern crate diesel;

extern crate dotenv;

use diesel::prelude::*;
use diesel::{insert_into, pg::PgConnection};

use dotenv::dotenv;

use std::env;
use std::error::Error;

use self::models::{NewReport, Report};

pub enum QueryType {
    ALL,
    ByReporter(String),
    ByReported(String),
    ById(i32),
}

pub fn query_report(query_type: QueryType) -> Result<Vec<Report>, Box<dyn Error>> {
    use schema::reports::dsl::*;

    let conn = establish_connection();

    let res: Vec<Report>;

    match query_type {
        QueryType::ALL => {
            res = reports.load(&conn)?;
        }
        QueryType::ByReporter(value) => {
            res = reports.filter(reporter.eq(value)).load::<Report>(&conn)?;
        }
        QueryType::ByReported(value) => {
            res = reports.filter(reported.eq(value)).load::<Report>(&conn)?;
        }
        QueryType::ById(value) => {
            res = reports.filter(id.eq(value)).load::<Report>(&conn)?;
        }
    }

    Ok(res)
}

pub fn insert_report(new_report: NewReport) -> Result<(), Box<dyn Error>> {
    use schema::reports::dsl::*;

    let conn = establish_connection();

    insert_into(reports).values(&new_report).execute(&conn)?;

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
