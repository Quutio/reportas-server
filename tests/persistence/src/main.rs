use crate::model::filter::{
    CompositeFilter, FilterItem, IntegerFilter, ReportFilter, ReportFilterPattern,
};
use chrono::Utc;
use sqlx::{postgres::PgPoolOptions, types::Uuid};

use reportas::Report;
use reportas::ReportStub;
use reportas::Tag;
use reportas::Tags;

pub mod reportas {
    tonic::include_proto!("report1_0");
}

pub mod model {
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
}

pub mod service {
    use chrono::NaiveDateTime;
    use sqlx::postgres::PgRow;
    use sqlx::{Executor, PgPool, Postgres, QueryBuilder, Row, Transaction};
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    use crate::model::filter::ReportFilterPattern;
    use crate::model::{NewReport, Report};

    use super::model::*;

    pub struct ReportDao {
        pub pool: PgPool,
        pub users: Arc<Mutex<HashSet<User>>>,
        pub user_reports: Arc<Mutex<HashMap<Uuid, Report>>>,
        pub reported_users: Arc<Mutex<HashMap<Uuid, Report>>>,
    }

    impl ReportDao {
        pub fn with_pool(pool: PgPool) -> ReportDao {
            ReportDao {
                pool,
                users: Arc::new(Mutex::new(HashSet::new())),
                user_reports: Arc::new(Mutex::new(HashMap::new())),
                reported_users: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    impl ReportDao {
        pub async fn insert_user<'c, E>(&self, uuid: Uuid, executor: E) -> anyhow::Result<User>
        where
            E: Executor<'c, Database = Postgres>,
        {
            let rec = sqlx::query!(
                r#"
INSERT INTO users (id) VALUES ($1) ON CONFLICT DO NOTHING RETURNING id
"#,
                uuid
            )
            .fetch_one(executor)
            .await?;

            let user = User::new(rec.id);
            // self.cache_user(user);
            Ok(user)
        }

        pub async fn insert_report(&self, report: &NewReport<'_>) -> anyhow::Result<Report> {
            let mut txn = self.pool.begin().await?;

            let reporter = self.insert_user(report.reporter, &mut txn).await;
            let reported = self.insert_user(report.reported, &mut txn).await;

            let report_rec = sqlx::query!(
                r#"
INSERT INTO reports (timestamp, reporter_id, reported_id, description)
VALUES ($1, $2, $3, $4)
RETURNING id, active, timestamp, reporter_id, reported_id, handler_id, handle_ts, comment, description, resolution as "resolution?: Resolution"
"#,
                report.timestamp,
                report.reporter,
                report.reported,
                report.description,
            )
                .fetch_one(&mut txn)
                .await?;

            let report_id = report_rec.id;

            sqlx::query!(
                r#"
INSERT INTO tags (content)
SELECT * FROM UNNEST($1::text[])
ON CONFLICT DO NOTHING
"#,
                &report
                    .tags
                    .iter()
                    .map(|tag| tag.content.to_string())
                    .collect::<Vec<_>>()
            )
            .execute(&mut txn)
            .await?;

            let tag_sel = sqlx::query!(
                r#"
SELECT * FROM tags
WHERE content IN (SELECT UNNEST($1::text[]))
"#,
                &report
                    .tags
                    .iter()
                    .map(|tag| tag.content.to_string())
                    .collect::<Vec<_>>()
            )
            .fetch_all(&mut txn)
            .await?;

            sqlx::query!(
                r#"
INSERT INTO tagmap (report_id, tag_id)
SELECT $1 AS report_id, tag_id FROM UNNEST($2::bigint[]) AS tag_id;
"#,
                report_id,
                &tag_sel.iter().map(|tag| tag.id).collect::<Vec<_>>()
            )
            .execute(&mut txn)
            .await?;

            txn.commit().await?;

            let rep = Report {
                id: report_rec.id,
                active: report_rec.active,
                timestamp: report_rec.timestamp,
                reporter: User::new(report_rec.reporter_id),
                reported: User::new(report_rec.reported_id),
                handler: report_rec.handler_id.map(|handler| User::new(handler)),
                handle_ts: report_rec.handle_ts,
                comment: report_rec.comment,
                description: report_rec.description,
                resolution: report_rec.resolution,
                tags: tag_sel
                    .iter()
                    .map(|rec| Tag {
                        content: rec.content.to_string(),
                    })
                    .collect::<Vec<_>>(),
            };

            //self.reported_users.lock().unwrap().insert(reported.to_uuid(), rep.clone());
            //self.user_reports.lock().unwrap().insert(reporter.to_uuid(), rep.clone());

            Ok(rep)
        }

        pub async fn query_report(&self, filter: &ReportFilterPattern) -> anyhow::Result<Vec<Report>> {
            let mut txn = self.pool.begin().await?;

            print!("SELECT reports.* FROM reports,tags,tagmap WHERE tagmap.tag_id = tags.id AND tagmap.report_id = reports.id AND ({}) GROUP BY reports.id", filter.generate_restriction());

            let asd = sqlx::query(&format!(
                r#"
SELECT reports.* FROM reports,tags,tagmap WHERE tagmap.tag_id = tags.id AND tagmap.report_id = reports.id AND ({}) GROUP BY reports.id
"#,
                filter.generate_restriction()
            ))
            .map(|row: PgRow| {
                let id: i64 = row.get("id");
                let active: bool = row.get("active");
                let timestamp: NaiveDateTime = row.get("timestamp");
                let reporter_id: Uuid = row.get("reporter_id");
                let reported_id: Uuid = row.get("reported_id");
                let handler_id: Option<Uuid> = row.get("handler_id");
                let handle_ts: Option<NaiveDateTime> = row.get("handle_ts");
                let comment: Option<String> = row.get("comment");
                let description: String = row.get("description");
                let resolution: Option<Resolution> = row.get("resolution");

                return Report {
                    id,
                    active,
                    timestamp,
                    reporter: User::new(reporter_id),
                    reported: User::new(reported_id),
                    handler: handler_id.map(|uuid| User::new(uuid)),
                    handle_ts,
                    comment,
                    description,
                    resolution,
                    tags: vec![],
                };
            })
            .fetch_all(&mut txn)
            .await?;

            let mut res: Vec<Report> = Vec::new();

            for rep in asd {
                let report_id = rep.id;

                let tags: Vec<Tag> = sqlx::query_as!(Tag, r#"
SELECT tags.content FROM tags,tagmap WHERE tags.id = tagmap.tag_id AND tagmap.report_id = $1 GROUP BY tags.id
"#,
                                         report_id
                ).fetch_all(&mut txn).await?;

                res.push(Report {
                    id: rep.id,
                    active: rep.active,
                    timestamp: rep.timestamp,
                    reporter: rep.reporter,
                    reported: rep.reported,
                    handler: rep.handler,
                    handle_ts: rep.handle_ts,
                    comment: rep.comment,
                    description: rep.description,
                    resolution: rep.resolution,
                    tags,
                })
            }

            Ok(res)
        }

        async fn deactivate_report(&self, id: i64, operator: User, comment: Option<String>) -> anyhow::Result<()> {
            let mut txn = self.pool.begin().await?;

            let _query = sqlx::query!(r#"
UPDATE reports SET active = false, handler_id = $1, comment = $2 WHERE id = $3
"#,
            operator.as_uuid(),
                                     comment,
                                     id
            ).execute(&mut txn).await?;
            Ok(())
        }
    }

    #[test]
    pub fn test_difference() {
        let vec1 = vec!["aa", "bb", "abba"];
        let vec2 = vec!["aa", "bb"];

        let difference = vec1
            .iter()
            .filter(|v1| vec2.iter().filter(|v2| v1 == v2).next().is_none())
            .map(|v| *v)
            .collect::<Vec<_>>();

        assert_eq!(difference, vec!["abba"]);
    }
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    use model::*;
    use service::*;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://reportas:passwd@localhost/reportas")
        .await?;

    let rdao = ReportDao::with_pool(pool);

    for _ in 0..10 {
        let nrep = NewReport {
            active: true,
            timestamp: Utc::now().naive_utc(),
            reporter: uuid::Uuid::new_v4(),
            reported: uuid::Uuid::new_v4(),
            description: "vou",
            tags: &vec!["jea", "test", "jouts", "hehe", "haha"]
                .into_iter()
                .map(|val| NewTag { content: val })
                .collect::<Vec<_>>(),
        };

        rdao.insert_report(&nrep).await.unwrap();
    }

    let filter = ReportFilterPattern::Composite(CompositeFilter::Union(vec![
        FilterItem::Single(ReportFilter::Id(IntegerFilter::Equals(33))),
        FilterItem::Composite(CompositeFilter::Intersection(vec![
            FilterItem::Single(ReportFilter::Active(true)),
            FilterItem::Single(ReportFilter::Id(IntegerFilter::GreaterThan(100))),
        ])),
    ]));

    let filter2 = ReportFilterPattern::Composite(CompositeFilter::Union(vec![
        FilterItem::Single(ReportFilter::Tag("haha".to_string())),
        FilterItem::Single(ReportFilter::Id(IntegerFilter::Equals(34))),
    ]));

    let res = rdao.query_report(&filter2).await.unwrap();

    println!("{:#?}", res);

    /*

    let filter = ReportFilterPattern::Composite(CompositeFilter::Intersection(vec![
        FilterItem::Single(ReportFilter::Active(true)),
        FilterItem::Single(ReportFilter::Id(IntegerFilter::Equals(1))),
        FilterItem::Composite(CompositeFilter::Complement(Box::new(
            FilterItem::Composite(CompositeFilter::Union(vec![
                FilterItem::Single(ReportFilter::Id(IntegerFilter::Equals(2))),
                FilterItem::Single(ReportFilter::Handler(Uuid::new_v4())),
            ])),
        ))),
    ]));

    */

    println!("{}", filter.generate_restriction());

    println!("{:#?}", filter);

    println!("Hello, world!");
    Ok(())
}
