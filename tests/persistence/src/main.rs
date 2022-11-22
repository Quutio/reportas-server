use chrono::Utc;
use sqlx::{postgres::PgPoolOptions, types::Uuid};

pub mod model {

    use sqlx::types::Uuid;

    use chrono::NaiveDateTime;

    #[derive(sqlx::Type)]
    #[sqlx(type_name = "resolution_enum", rename_all = "lowercase")]
    #[derive(Debug, Clone)]
    pub enum Resolution {
        Positive,
        Neutral,
        Negative,
    }

    pub struct NewReport<'a> {
        pub active: bool,
        pub timestamp: NaiveDateTime,
        pub reporter: Uuid,
        pub reported: Uuid,
        pub description: &'a str,
        pub tags: &'a [NewTag<'a>],
    }

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

    #[derive(Debug, Clone)]
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

    #[derive(Debug, Clone)]
    pub struct Tag {
        pub id: i64,
        pub content: String,
    }
}

pub mod service {
    use sqlx::{PgPool, QueryBuilder, Transaction, Executor, Postgres};
    use uuid::Uuid;

    use crate::model::{NewReport, Report};

    use super::model::*;

    pub struct ReportDao {
        pub pool: PgPool,
    }

    impl ReportDao {

        pub async fn insert_user<'c, E>(&self, uuid: Uuid, executor: E) -> User
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
                .await
                .unwrap();

            User::new(rec.id)
        }

        pub async fn insert_report(&self, report: &NewReport<'_>) -> Option<Report> {
            let mut txn = self.pool.begin().await.unwrap();

            let user = self.insert_user(report.reporter, &mut txn).await;

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
            .await
            .unwrap();

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
            .await
            .unwrap();

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
            .await
            .unwrap();

            let tag_map = sqlx::query!(
                r#"
INSERT INTO tagmap (report_id, tag_id)
SELECT $1 AS report_id, tag_id FROM UNNEST($2::bigint[]) AS tag_id;
"#,
                report_id,
                &tag_sel.iter().map(|tag| tag.id).collect::<Vec<_>>()
            )
            .execute(&mut txn)
            .await
            .unwrap();

            txn.commit().await.unwrap();

            println!("{:?}", tag_sel);

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
                        id: rec.id,
                        content: rec.content.to_string(),
                    })
                    .collect::<Vec<_>>(),
            };

            Some(rep)
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

    let rdao = ReportDao { pool: pool };

    let nrep = NewReport {
        active: true,
        timestamp: Utc::now().naive_utc(),
        reporter: uuid::Uuid::new_v4(),
        reported: uuid::Uuid::new_v4(),
        description: "vou",
        tags: &vec!["jea", "test", "jouts", "hehe"]
            .into_iter()
            .map(|val| NewTag { content: val })
            .collect::<Vec<_>>(),
    };

    rdao.insert_report(&nrep).await.unwrap();

    println!("Hello, world!");
    Ok(())
}
