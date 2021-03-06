pub mod data;

#[macro_use]
extern crate diesel;

pub use data::models;
pub use data::schema;

extern crate dotenv;

use diesel::{insert_into, pg::PgConnection, update};
use diesel::{prelude::*, r2d2::ConnectionManager};

use tokio::sync::RwLock;
use tokio_diesel::AsyncRunQueryDsl;

use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use self::models::{NewReport, Report};

pub mod report {
    tonic::include_proto!("report");
}

// Comment ..

///
/// Specify type kind of query to execute.
///
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

#[tonic::async_trait]
pub trait ReportDb<M>
where
    M: diesel::r2d2::ManageConnection,
{
    async fn insert_report(&self, new_report: NewReport) -> Result<Report, Box<dyn Error>>;

    async fn query_report(&self, query_type: QueryType) -> Result<Vec<Report>, Box<dyn Error>>;

    async fn deactivate_report(
        &self,
        id: i64,
        operator: String,
        comment: Option<String>,
    ) -> Result<Report, Box<dyn Error>>;
}

pub struct PgReportDb {
    pool: diesel::r2d2::Pool<ConnectionManager<PgConnection>>,
    cache: Arc<RwLock<HashMap<i64, Report>>>,
}

impl PgReportDb {
    pub fn new(addr: &str) -> Result<Self, Box<dyn Error>> {
        let manager = ConnectionManager::<PgConnection>::new(addr);
        let pool = diesel::r2d2::Pool::builder().build(manager)?;

        Ok(Self {
            pool,
            cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn load_to_cache(&self, deactive: bool) -> Result<(), Box<dyn Error>> {
        use schema::reports::dsl::*;

        let to_cache: Vec<Report>;

        if deactive {
            to_cache = reports.load(&self.pool.get()?)?;
        } else {

            to_cache = reports
                .filter(active.eq(true))
                .load::<Report>(&self.pool.get()?)?;
        }

        for report in to_cache {
            self.cache.write().await.insert(report.id, report);
        }

        Ok(())
    }

    async fn insert_to_cache(&self, insertee: Report) {
        let mut lock = self.cache.write().await;
        lock.insert(insertee.id, insertee);
    }
}

#[tonic::async_trait]
impl ReportDb<ConnectionManager<PgConnection>> for PgReportDb {
    async fn insert_report(&self, new_report: NewReport) -> Result<Report, Box<dyn Error>> {
        use schema::reports::dsl::*;

        // let rep = new_report.clone();

        let res = insert_into(reports)
            .values(new_report)
            .get_result_async::<Report>(&self.pool).await?;

        self.insert_to_cache(res.clone()).await;

        Ok(res)
    }

    async fn deactivate_report(
        &self,
        identifier: i64,
        operator: String,
        ccomment: Option<String>,
    ) -> Result<Report, Box<dyn Error>> {
        use schema::reports::dsl::*;

        let utc = chrono::Utc::now();
        let ts = utc.timestamp();

        update(reports.filter(id.eq(identifier)))
            .set(active.eq(false))
            .execute_async(&self.pool).await?;

        update(reports.filter(id.eq(identifier)))
            .set(handler.eq(operator))
            .execute_async(&self.pool).await?;

        if let Some(comm) = ccomment {
            update(reports.filter(id.eq(identifier)))
                .set(comment.eq(comm))
                .execute_async(&self.pool).await?;
        }

        let res = update(reports.filter(id.eq(identifier)))
            .set(handle_ts.eq(ts))
            .get_result_async::<Report>(&self.pool).await?;

        self.insert_to_cache(res.clone()).await;

        Ok(res)
    }

    ///
    /// Query reports by a query type.
    ///
    /// # Arguments
    ///
    /// * `query_type` - `QueryType` enum with a required value.
    ///
    ///
    async fn query_report(&self, query_type: QueryType) -> Result<Vec<Report>, Box<dyn Error>> {
        use schema::reports::dsl::*;

        let res: Vec<Report>;

        match query_type {
            QueryType::ALL => {
                let cached: Vec<Report> = self.cache.read().await.values().cloned().collect();

                if cached.is_empty() {
                    res = reports.load_async(&self.pool).await?;
                } else {
                    res = cached;
                }

                for report in &res {
                    self.insert_to_cache(report.clone()).await;
                }
            }
            QueryType::ByReporter(value) => {
                let cached: Vec<Report> = self
                    .cache
                    .read()
                    .await
                    .values()
                    .cloned()
                    .filter(|x| x.reporter == value)
                    .collect();

                if cached.is_empty() {
                    res = reports
                        .filter(reporter.eq(value))
                        .load_async::<Report>(&self.pool).await?;
                } else {
                    res = cached;
                }

                for report in &res {
                    self.insert_to_cache(report.clone()).await;
                }
            }
            QueryType::ByReported(value) => {
                let cached: Vec<Report> = self
                    .cache
                    .read()
                    .await
                    .values()
                    .cloned()
                    .filter(|x| x.reported == value)
                    .collect();

                if cached.is_empty() {
                    res = reports
                        .filter(reported.eq(value))
                        .load_async::<Report>(&self.pool).await?;
                } else {
                    res = cached;
                }

                for report in &res {
                    self.insert_to_cache(report.clone()).await;
                }
            }
            QueryType::ByTimestamp(value) => {
                let cached: Vec<Report> = self
                    .cache
                    .read()
                    .await
                    .values()
                    .cloned()
                    .filter(|x| x.timestamp <= value)
                    .collect();

                if cached.is_empty() {
                    res = reports
                        .filter(timestamp.le(value))
                        .load_async::<Report>(&self.pool).await?;
                } else {
                    res = cached;
                }

                for report in &res {
                    self.insert_to_cache(report.clone()).await;
                }
            }
            QueryType::ById(value) => {
                let cached: Vec<Report> = self
                    .cache
                    .read()
                    .await
                    .values()
                    .cloned()
                    .filter(|x| x.id == value)
                    .collect();

                if cached.is_empty() {
                    res = reports
                        .filter(id.eq(value))
                        .load_async::<Report>(&self.pool).await?;
                } else {
                    res = cached;
                }

                for report in &res {
                    self.insert_to_cache(report.clone()).await;
                }
            }
            QueryType::ByActive => {
                let cached: Vec<Report> = self
                    .cache
                    .read()
                    .await
                    .values()
                    .cloned()
                    .filter(|x| x.active)
                    .collect();

                if cached.is_empty() {
                    res = reports
                        .filter(active.eq(true))
                        .load_async::<Report>(&self.pool).await?
                } else {
                    res = cached;
                }

                for report in &res {
                    self.insert_to_cache(report.clone()).await;
                }
            }
            QueryType::ByHandler(value) => {
                let cached: Vec<Report> = self
                    .cache
                    .read()
                    .await
                    .values()
                    .cloned()
                    .filter(|x| x.handler == Some(value.clone()))
                    .collect();

                if cached.is_empty() {
                    res = reports
                        .filter(handler.eq(value))
                        .load_async::<Report>(&self.pool).await?;
                } else {
                    res = cached;
                }

                for report in &res {
                    self.insert_to_cache(report.clone()).await;
                }
            }
            QueryType::ByHandleTimestamp(value) => {
                let cached: Vec<Report> = self
                    .cache
                    .read()
                    .await
                    .values()
                    .cloned()
                    .filter(|x| x.handle_ts <= Some(value))
                    .collect();

                if cached.is_empty() {
                    res = reports
                        .filter(handle_ts.eq(value))
                        .load_async::<Report>(&self.pool).await?;
                } else {
                    res = cached;
                }

                for report in &res {
                    self.insert_to_cache(report.clone()).await;
                }
            }
        }

        Ok(res)
    }
}
