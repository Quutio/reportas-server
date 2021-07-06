pub mod models;
pub mod schema;

#[macro_use]
extern crate diesel;

extern crate dotenv;

use diesel::{r2d2::ConnectionManager, prelude::*};
use diesel::{insert_into, update, pg::PgConnection};

use dotenv::dotenv;

use std::env;
use std::error::Error;

use self::models::{NewReport, Report};

pub enum QueryType {
    ALL,
    ByReporter(String),
    ByReported(String),
    ByTimestamp(i64),
    ById(i64),
    ByActive,
    ByHandler(String),
    ByHandleTimestamp(i64),
}

pub trait ReportDb<M>
where
    M: diesel::r2d2::ManageConnection
{
    fn insert_report(&self, new_report: &NewReport) -> Result<Report, Box<dyn Error>>;

    fn query_report(&self, query_type: QueryType) -> Result<Vec<Report>, Box<dyn Error>>;

    fn deactivate_report(&self, id: i64, operator: &str) -> Result<Report, Box<dyn Error>>;
}

pub struct PgReportDb {
    pool: diesel::r2d2::Pool<ConnectionManager<PgConnection>>,
}

impl PgReportDb {

    pub fn new(addr: &str) -> Result<Self, Box<dyn Error>> {

        let manager = ConnectionManager::<PgConnection>::new(addr);
        let pool = diesel::r2d2::Pool::builder().build(manager)?;

        Ok( Self { pool } )
    }
}

impl ReportDb<ConnectionManager<PgConnection>> for PgReportDb {

    fn insert_report(&self, new_report: &NewReport) -> Result<Report, Box<dyn Error>> {
        use schema::reports::dsl::*;

        let res = insert_into(reports).values(new_report).get_result::<Report>(&self.pool.get().unwrap())?;

        Ok(res)
    }

    fn deactivate_report(&self, identifier: i64, operator: &str) -> Result<Report, Box<dyn Error>> {
        use schema::reports::dsl::*;

        let utc = chrono::Utc::now();
        let ts = utc.timestamp();

        update(reports.filter(id.eq(identifier)))
            .set(active.eq(false))
            .execute(&self.pool.get()?)?;

        update(reports.filter(id.eq(identifier)))
            .set(handler.eq(operator))
            .execute(&self.pool.get()?)?;

        let res = update(reports.filter(id.eq(identifier)))
            .set(handle_ts.eq(ts))
            .get_result::<Report>(&self.pool.get()?)?;

        Ok(res)
    }

    ///
    /// Query reports by a query type.
    ///
    /// # Arguments
    ///
    /// * `query_type` - `QueryType` enum with a required value.
    ///
    /// # Examples
    ///
    /// ```
    /// use models::QueryType;
    ///
    /// let queried = query_report(QueryType::ById(420));
    /// ```
    ///
    fn query_report(&self, query_type: QueryType) -> Result<Vec<Report>, Box<dyn Error>> {

        use schema::reports::dsl::*;

        let res: Vec<Report>;

        match query_type {
            QueryType::ALL => {
                res = reports
                    .load(&self.pool.get().unwrap())?;
            }
            QueryType::ByReporter(value) => {
                res = reports
                    .filter(reporter.eq(value))
                    .load::<Report>(&self.pool.get().unwrap())?;
            }
            QueryType::ByReported(value) => {
                res = reports
                    .filter(reported.eq(value))
                    .load::<Report>(&self.pool.get().unwrap())?;
            }
            QueryType::ByTimestamp(value) => {
                res = reports
                    .filter(timestamp.le(value))
                    .load::<Report>(&self.pool.get().unwrap())?;
            }
            QueryType::ById(value) => {
                res = reports
                    .filter(id.eq(value))
                    .load::<Report>(&self.pool.get().unwrap())?;
            }
            QueryType::ByActive => {
                res = reports
                    .filter(active.eq(true))
                    .load::<Report>(&self.pool.get()?)?;
            }
            QueryType::ByHandler(value) => {
                res = reports
                    .filter(handler.eq(value))
                    .load::<Report>(&self.pool.get().unwrap())?;
            }
            QueryType::ByHandleTimestamp(value) => {
                res = reports
                    .filter(handle_ts.le(value))
                    .load::<Report>(&self.pool.get().unwrap())?;
            }
        }

        Ok(res)
    }
}
