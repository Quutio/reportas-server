use crate::{data::models::*, report_transporter::Transporter};
use service::{PgReportDb, QueryType, ReportDb};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("database failed")]
    DatabaseFailed,
    #[error("transport error")]
    TransportError,
    #[error("invalid timestamp")]
    InvalidTimestamp,
}

pub struct ReportHandler {
    db: PgReportDb,
    transporter: Transporter,
}

impl ReportHandler {
    pub async fn new(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let db = PgReportDb::new(addr).unwrap();
        db.load_to_cache(false).await?;

        let addrs = vec!["http://[::1]:50024", "http://[::1]:50025"];

        let transporter = Transporter::new(addrs).await?;

        Ok(ReportHandler {
            db: db,
            transporter: transporter,
        })
    }

    pub async fn submit_report(&self, req: ReportRequest) -> Result<Report, Error> {
        let utc = chrono::Utc::now();
        let ts = utc.timestamp();

        let tags: Option<&str>;
        if req.tags.is_empty() {
            tags = None;
        } else {
            tags = Some(&req.tags)
        }

        let new_report = NewReport {
            active: true,
            timestamp: ts,
            reporter: req.reporter.as_str(),
            reported: req.reported.as_str(),
            description: req.desc.as_str(),
            tags: tags,
        };

        let rep = match self.db.insert_report(&new_report).await {
            Ok(val) => val,
            Err(_) => return Err(Error::DatabaseFailed),
        };

        match self.transporter.transport(rep.clone().into()).await {
            Ok(_) => {}
            Err(_) => return Err(Error::TransportError),
        }

        Ok(rep.into())
    }

    pub async fn deactivate_report(&self, req: ReportDeactivateRequest) -> Result<Report, Error> {
        let rep = match self
            .db
            .deactivate_report(req.id, &req.operator, req.comment.as_deref())
            .await
        {
            Ok(val) => val,
            Err(_) => return Err(Error::DatabaseFailed),
        };

        match self.transporter.deactivate(rep.clone().into()).await {
            Ok(_) => {}
            Err(_) => return Err(Error::TransportError),
        }

        Ok(rep)
    }

    pub async fn query_all_reports(&self) -> Result<Vec<Report>, Error> {
        let queried = match self.db.query_report(QueryType::ALL).await {
            Ok(val) => val,
            Err(_) => return Err(Error::DatabaseFailed),
        };

        Ok(queried)
    }

    pub async fn query_reports_by_reporter(
        &self,
        query: ReportQuery,
    ) -> Result<Vec<Report>, Error> {
        let queried = match self
            .db
            .query_report(QueryType::ByReporter(query.query))
            .await
        {
            Ok(val) => val,
            Err(_) => return Err(Error::DatabaseFailed),
        };

        Ok(queried)
    }

    pub async fn query_reports_by_reported(
        &self,
        query: ReportQuery,
    ) -> Result<Vec<Report>, Error> {
        let queried = match self
            .db
            .query_report(QueryType::ByReported(query.query))
            .await
        {
            Ok(val) => val,
            Err(_) => return Err(Error::DatabaseFailed),
        };

        Ok(queried)
    }

    pub async fn query_reports_by_timestamp(
        &self,
        query: ReportQuery,
    ) -> Result<Vec<Report>, Error> {
        let ts = match query.query.parse::<i64>() {
            Ok(val) => val,
            Err(_) => return Err(Error::InvalidTimestamp),
        };

        let queried = match self.db.query_report(QueryType::ByTimestamp(ts)).await {
            Ok(val) => val,
            Err(_) => return Err(Error::DatabaseFailed),
        };

        Ok(queried)
    }

    pub async fn query_reports_by_id(&self, query: ReportQuery) -> Result<Vec<Report>, Error> {
        let queried = match self.db.query_report(QueryType::ById(query.id)).await {
            Ok(val) => val,
            Err(_) => return Err(Error::DatabaseFailed),
        };

        Ok(queried)
    }

    pub async fn query_reports_by_handler(&self, query: ReportQuery) -> Result<Vec<Report>, Error> {
        let queried = match self
            .db
            .query_report(QueryType::ByHandler(query.query))
            .await
        {
            Ok(val) => val,
            Err(_) => return Err(Error::DatabaseFailed),
        };

        Ok(queried)
    }

    pub async fn query_reports_by_handle_timestamp(
        &self,
        query: ReportQuery,
    ) -> Result<Vec<Report>, Error> {
        let ts = match query.query.parse::<i64>() {
            Ok(val) => val,
            Err(_) => return Err(Error::InvalidTimestamp),
        };

        let queried = match self.db.query_report(QueryType::ByHandleTimestamp(ts)).await {
            Ok(val) => val,
            Err(_) => return Err(Error::DatabaseFailed),
        };
        Ok(queried)
    }

    pub async fn query_reports_by_active(&self) -> Result<Vec<Report>, Error> {
        let queried = match self.db.query_report(QueryType::ByActive).await {
            Ok(val) => val,
            Err(_) => return Err(Error::DatabaseFailed),
        };
        Ok(queried)
    }
}
