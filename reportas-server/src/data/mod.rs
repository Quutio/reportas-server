use sqlx::types::Uuid;

use chrono::NaiveDateTime;
use crate::reportas;

#[derive(sqlx::Type)]
#[sqlx(type_name = "resolution_enum", rename_all = "lowercase")]
#[derive(Debug, Clone)]
pub enum Resolution {
    Positive,
    Neutral,
    Negative,
}

#[derive(Debug)]
pub struct NewReport<'a> {
    pub active: bool,
    pub timestamp: NaiveDateTime,
    pub reporter: Uuid,
    pub reported: Uuid,
    pub description: &'a str,
    pub tags: &'a [NewTag<'a>],
}

#[derive(Debug)]
pub struct NewTag<'a> {
    pub content: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct User {
    uuid: Uuid,
}

impl User {
    pub fn new(uuid: Uuid) -> Self {
        Self { uuid }
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn to_uuid(self) -> Uuid {
        self.uuid
    }
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Report {
    pub id: i64,
    pub active: bool,
    pub timestamp: NaiveDateTime,
    pub reporter: User,
    pub reported: User,
    pub handler: Option<User>,
    pub handle_ts: Option<NaiveDateTime>,
    pub comment: Option<String>,
    pub description: String,
    pub resolution: Option<Resolution>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Tag {
    pub content: String,
}

impl From<reportas::Tag> for Tag {
    fn from(value: reportas::Tag) -> Self {
        Tag {
            content: value.tag,
        }
    }
}

/*
impl From<Report> for reportas::Report {
    fn from(value: Report) -> Self {
        reportas::Report {
            id: value.id,
            active: value.active,
            insert_timestamp: value.timestamp,
            handler: "".to_string(),
            handle_timestamp: None,
            comment: "".to_string(),
            report: None,
        }
    }
}
 */

impl From<Tag> for reportas::Tag {
    fn from(value: Tag) -> Self {
        reportas::Tag {
            tag: value.content,
        }
    }
}

pub mod filter {
    use super::*;
    use std::fmt;
    use std::fmt::{write, Display, Formatter};

    #[derive(Debug)]
    pub enum TimestampFilter {
        Equals(NaiveDateTime),
        GreaterThan(NaiveDateTime),
        LessThan(NaiveDateTime),
        GreaterThanEqual(NaiveDateTime),
        LessThanEqual(NaiveDateTime),
    }

    impl Display for TimestampFilter {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            match self {
                TimestampFilter::Equals(ts) => {
                    write!(f, "= '{}'", ts)
                }
                TimestampFilter::GreaterThan(ts) => {
                    write!(f, "> '{}'", ts)
                }
                TimestampFilter::LessThan(ts) => {
                    write!(f, "< '{}'", ts)
                }
                TimestampFilter::GreaterThanEqual(ts) => {
                    write!(f, ">= '{}'", ts)
                }
                TimestampFilter::LessThanEqual(ts) => {
                    write!(f, "<= '{}'", ts)
                }
            }
        }
    }

    #[derive(Debug)]
    pub enum IntegerFilter {
        Equals(i64),
        GreaterThan(i64),
        LessThan(i64),
        GreaterThanEqual(i64),
        LessThanEqual(i64),
    }

    impl Display for IntegerFilter {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            match self {
                IntegerFilter::Equals(num) => {
                    write!(f, "= {}", num)
                }
                IntegerFilter::GreaterThan(num) => {
                    write!(f, "> {}", num)
                }
                IntegerFilter::LessThan(num) => {
                    write!(f, "< {}", num)
                }
                IntegerFilter::GreaterThanEqual(num) => {
                    write!(f, ">= {}", num)
                }
                IntegerFilter::LessThanEqual(num) => {
                    write!(f, "<= {}", num)
                }
            }
        }
    }

    impl TimestampFilter {
        pub fn resolve(&self, comparator: &NaiveDateTime) -> bool {
            match self {
                TimestampFilter::Equals(c) => c.eq(comparator),
                TimestampFilter::GreaterThan(c) => c.gt(comparator),
                TimestampFilter::LessThan(c) => c.lt(comparator),
                TimestampFilter::GreaterThanEqual(c) => c.ge(comparator),
                TimestampFilter::LessThanEqual(c) => c.le(comparator),
            }
        }
    }

    impl IntegerFilter {
        pub fn resolve(&self, comparator: &i64) -> bool {
            match self {
                IntegerFilter::Equals(c) => c.eq(comparator),
                IntegerFilter::GreaterThan(c) => c.gt(comparator),
                IntegerFilter::LessThan(c) => c.lt(comparator),
                IntegerFilter::GreaterThanEqual(c) => c.ge(comparator),
                IntegerFilter::LessThanEqual(c) => c.le(comparator),
            }
        }
    }

    #[derive(Debug)]
    pub enum ReportFilter {
        Id(IntegerFilter),
        Reported(Uuid),
        Reporter(Uuid),
        Active(bool),
        Handler(Uuid),
        InsertTimestamp(TimestampFilter),
        HandleTimestamp(TimestampFilter),
        Tag(String),
    }

    #[derive(Debug)]
    pub enum FilterItem {
        Single(ReportFilter),
        Composite(CompositeFilter),
    }

    #[derive(Debug)]
    pub enum CompositeFilter {
        Union(Vec<FilterItem>),
        Intersection(Vec<FilterItem>),
        Complement(Box<FilterItem>),
    }

    #[derive(Debug)]
    pub enum ReportFilterPattern {
        Composite(CompositeFilter),
        Singleton(ReportFilter),
    }

    impl Display for ReportFilter {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                ReportFilter::Id(filter) => write!(f, "reports.id {}", filter),
                ReportFilter::Reported(uuid) => write!(f, "reports.reporter_id = '{}'", uuid),
                ReportFilter::Reporter(uuid) => write!(f, "reports.reported_id ='{}'", uuid),
                ReportFilter::Active(active) => write!(f, "reports.active = {}", active),
                ReportFilter::Handler(uuid) => write!(f, "reports.handler_id = '{}'", uuid),
                ReportFilter::InsertTimestamp(filter) => {
                    write!(f, "reports.insert_timestamp {}", filter)
                }
                ReportFilter::HandleTimestamp(filter) => {
                    write!(f, "reports.handle_timestamp {}", filter)
                }
                ReportFilter::Tag(tag) => {
                    write!(f, "tags.content = '{}'", tag)
                }
            }
        }
    }

    impl Display for FilterItem {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                FilterItem::Single(report_filter) => write!(f, "({})", report_filter),
                FilterItem::Composite(composite_filter) => write!(f, "({})", composite_filter),
            }
        }
    }

    impl Display for CompositeFilter {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                CompositeFilter::Union(items) => {
                    let joined = items
                        .iter()
                        .map(|item| item.to_string())
                        .collect::<Vec<_>>()
                        .join(" OR ");
                    write!(f, "{}", joined)
                }
                CompositeFilter::Intersection(items) => {
                    let joined = items
                        .iter()
                        .map(|item| item.to_string())
                        .collect::<Vec<_>>()
                        .join(" AND ");
                    write!(f, "{}", joined)
                }
                CompositeFilter::Complement(item) => write!(f, "NOT {}", item),
            }
        }
    }

    impl Display for ReportFilterPattern {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                ReportFilterPattern::Composite(filter) => write!(f, "{}", filter),
                ReportFilterPattern::Singleton(report_filter) => write!(f, "{}", report_filter),
            }
        }
    }

    impl ReportFilterPattern {
        pub fn generate_restriction(&self) -> String {
            self.to_string()
        }
    }
}
