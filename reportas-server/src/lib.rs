
mod data;

use tokio::sync::RwLock;

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::{Arc, Mutex};

pub mod report1_0 {
    tonic::include_proto!("report1_0");
}

pub mod report {
    tonic::include_proto!("report");
}

use chrono::NaiveDateTime;
use sqlx::postgres::PgRow;
use sqlx::{Executor, PgPool, Postgres, QueryBuilder, Row, Transaction};
use uuid::Uuid;

use data::filter::ReportFilterPattern;
use data::{NewReport, Report};
use data::User;
use crate::data::{Resolution, Tag};

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
            .fetch_all(&mut *txn)
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
