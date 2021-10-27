use crate::schema::reports;

use crate::report;

#[derive(Queryable, Debug, Clone, PartialEq)]
pub struct Report {
    pub id: i64,
    pub active: bool,
    pub timestamp: i64,

    pub reporter: String,
    pub reported: String,

    pub handler: Option<String>,
    pub handle_ts: Option<i64>,
    pub comment: Option<String>,

    pub description: String,
    pub tags: Option<String>,
}

impl From<report::IdentifiedReportMessage> for Report {
    fn from(f: report::IdentifiedReportMessage) -> Self {
        Self {
            id: f.id,
            active: f.active,
            timestamp: f.timestamp,
            reporter: f.reporter,
            reported: f.reported,
            handler: {
                if !f.handler.is_empty() {
                    Some(f.handler)
                } else {
                    None
                }
            },
            handle_ts: {
                if !f.handle_ts == -1 {
                    Some(f.handle_ts)
                } else {
                    None
                }
            },
            comment: {
                if !f.comment.is_empty() {
                    Some(f.comment)
                } else {
                    None
                }
            },
            description: f.desc,
            tags: {
                if !f.tags.is_empty() {
                    Some(f.tags)
                } else {
                    None
                }
            },
        }
    }
}

impl From<Report> for report::IdentifiedReportMessage {
    fn from(f: Report) -> Self {
        Self {
            id: f.id,
            active: f.active,
            timestamp: f.timestamp,
            reporter: f.reporter,
            reported: f.reported,
            handler: f.handler.unwrap_or_else(|| "".to_owned()),
            handle_ts: f.handle_ts.unwrap_or(-1),
            comment: f.comment.unwrap_or_else(|| "".to_owned()),
            desc: f.description,
            tags: f.tags.unwrap_or_else(|| "".to_owned()),
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "reports"]
pub struct NewReport<'a> {
    pub active: bool,
    pub timestamp: i64,
    pub reporter: &'a str,
    pub reported: &'a str,
    pub description: &'a str,
    pub tags: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReportRequest {
    pub reporter: String,
    pub reported: String,
    pub desc: String,
    pub tags: String,
}

impl From<report::ReportMessage> for ReportRequest {
    fn from(f: report::ReportMessage) -> Self {
        Self {
            reporter: f.reporter,
            reported: f.reported,
            desc: f.desc,
            tags: f.tags,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReportDeactivateRequest {
    pub id: i64,
    pub operator: String,
    pub comment: Option<String>,
}

impl From<report::ReportDeactivateRequest> for ReportDeactivateRequest {
    fn from(f: report::ReportDeactivateRequest) -> Self {
        Self {
            id: f.id,
            operator: f.operator,
            comment: {
                if !f.comment.is_empty() {
                    Some(f.comment)
                } else {
                    None
                }
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReportQuery {
    pub query: String,
    pub id: i64,
}

impl From<report::ReportQuery> for ReportQuery {
    fn from(f: report::ReportQuery) -> Self {
        Self {
            query: f.query,
            id: f.id,
        }
    }
}
