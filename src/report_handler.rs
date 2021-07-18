
use service::{PgReportDb, QueryType, ReportDb};

use crate::{data::{models::*}, transporter::Transporter};

pub struct ReportHandler {
    db: PgReportDb,
    transporter: Transporter,
}

impl ReportHandler {

    pub async fn submit_report(&self, req: ReportRequest) -> Result<Report, Box<dyn std::error::Error>> {

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

        Ok(self.db.insert_report(&new_report).await?)
    }

    pub async fn deactivate_report(&self, req: ReportDeactivateRequest) -> Result<Report, Box<dyn std::error::Error>> {

        let rep = self.db.deactivate_report(req.id, &req.operator, req.comment.as_deref()).await?;

        self.transporter.deactivate(rep.clone().into()).await?;

        Ok(rep)
    }

    pub async fn query_all_reports(&self) -> Result<Vec<Report>, Box<dyn std::error::Error>> {
        let queried = self.db.query_report(QueryType::ALL).await?;
        Ok(queried)
    }

    pub async fn query_reports_by_reporter(&self, query: ReportQuery) -> Result<Vec<Report>, Box<dyn std::error::Error>> {
        let queried = self.db.query_report(QueryType::ByReporter(query.query)).await?;
        Ok(queried)
    }

    pub async fn query_reports_by_reported(&self, query: ReportQuery) -> Result<Vec<Report>, Box<dyn std::error::Error>> {
        let queried = self.db.query_report(QueryType::ByReported(query.query)).await?;
        Ok(queried)
    }

    pub async fn query_reports_by_timestamp(&self, query: ReportQuery) -> Result<Vec<Report>, Box<dyn std::error::Error>> {

        let ts = query.query.parse::<i64>()?;

        let queried = self.db.query_report(QueryType::ByTimestamp(ts)).await?;
        Ok(queried)
    }

    pub async fn query_reports_by_id(&self, query: ReportQuery) -> Result<Vec<Report>, Box<dyn std::error::Error>> {

        let queried = self.db.query_report(QueryType::ById(query.id)).await?;
        Ok(queried)
    }

    pub async fn query_reports_by_handler(&self, query: ReportQuery) -> Result<Vec<Report>, Box<dyn std::error::Error>> {

        let queried = self.db.query_report(QueryType::ByHandler(query.query)).await?;
        Ok(queried)
    }

    pub async fn query_reports_by_handle_timestamp(&self, query: ReportQuery) -> Result<Vec<Report>, Box<dyn std::error::Error>> {

        let ts = query.query.parse::<i64>()?;

        let queried = self.db.query_report(QueryType::ByHandleTimestamp(ts)).await?;
        Ok(queried)
    }

    pub async fn query_reports_by_active(&self) -> Result<Vec<Report>, Box<dyn std::error::Error>> {

        let queried = self.db.query_report(QueryType::ByActive).await?;
        Ok(queried)
    }
}
