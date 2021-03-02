extern crate clap;
extern crate dotenv;

mod transporter;

use clap::{App, Arg};

use service::models::{NewReport};
use service::{insert_report, query_report};
use service::*;

use tonic::{transport::Server, Request, Response, Status};

// use futures::{Stream, StreamExt};
use tokio::sync::mpsc;

use report::report_handler_server::{ReportHandler, ReportHandlerServer};
use report::{IdentifiedReportMessage, ReportMessage, ReportQuery, ReportRequest, ReportResponse, ReportId};

use std::sync::Arc;
use std::sync::Mutex;

pub mod report {
    tonic::include_proto!("report");
}

pub struct MainReportHandler {
    db: PgReportDb,
    results: Arc<Mutex<Vec<IdentifiedReportMessage>>>,
}

impl MainReportHandler {
    pub fn new(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {

        let db = PgReportDb::new(addr).unwrap();

        Ok(
            MainReportHandler {
                db: db,
                results: Arc::new(Mutex::new(Vec::<IdentifiedReportMessage>::new()))
            }
        )
    }
}

#[allow(unused_variables)]
#[tonic::async_trait]
impl ReportHandler for MainReportHandler {

    type QueryAllReportsStream = mpsc::Receiver<Result<IdentifiedReportMessage, Status>>;
    type QueryReportsByReporterStream = mpsc::Receiver<Result<IdentifiedReportMessage, Status>>;
    type QueryReportsByReportedStream = mpsc::Receiver<Result<IdentifiedReportMessage, Status>>;
    type QueryReportsByTimestampStream = mpsc::Receiver<Result<IdentifiedReportMessage, Status>>;

    ///
    /// Handle a (gRPC) report submission:
    ///
    /// - TODO: verify validity
    /// - Make entity persistent. (database)
    /// - handle other gRPC stuff.
    ///
    async fn submit_report(
        &self,
        request: Request<ReportRequest>,
    ) -> Result<Response<ReportResponse>, Status> {
        println!("\nREQUEST:\n{:?}\n", request);

        let req_msg = request.into_inner().msg.expect("\nBAD REQUEST :: 400\n");
        let req_clone = req_msg.clone();

        let utc = chrono::Utc::now();
        let ts = utc.timestamp();

        let msg = ReportMessage {
            reporter: req_msg.reporter,
            reported: req_msg.reported,
            desc: req_msg.desc,
        };

        let new_report = NewReport {
            active: true,
            timestamp: ts,
            reporter: req_clone.reporter.as_str(),
            reported: req_clone.reported.as_str(),
            description: req_clone.desc.as_str(),
        };

        let rep = self.db.insert_report(&new_report).expect("[!] data insertion failed");

        println!("id -> {}", rep.id);

        let irm = transporter::report::IdentifiedReportMessage {
            id:         rep.id,
            active:     rep.active,
            timestamp:  rep.timestamp,
            reporter:   rep.reporter,
            reported:   rep.reported,
            desc:       rep.description,
        };

        transporter::transport(irm).await;

        let resp = report::ReportResponse { msg: Some(msg) };

        Ok(Response::new(resp))
    }

    async fn deactivate_report(
        &self,
        request: Request<ReportId>
    ) ->Result<Response<IdentifiedReportMessage>, tonic::Status> {

        let rep = self.db.deactivate_report(request.into_inner().id).unwrap();

        let irm = report::IdentifiedReportMessage {
            id:         rep.id,
            active:     rep.active,
            timestamp:  rep.timestamp,
            reporter:   rep.reporter,
            reported:   rep.reported,
            desc:       rep.description,
        };

        Ok(Response::new(irm))

    }

    ///
    /// Query *ALL* identified reports from the database.
    ///
    async fn query_all_reports(
        &self,
        request: Request<ReportQuery>,
    ) -> Result<Response<Self::QueryAllReportsStream>, Status> {

        let query = request.into_inner().query;
        let queried = self.db.query_report(service::QueryType::ALL).unwrap();

        let mut irms: Vec<IdentifiedReportMessage> = Vec::new();

        for rep in queried.iter() {
            let irm = IdentifiedReportMessage {
                id:         rep.id as i64,
                active:     rep.active,
                timestamp:  rep.timestamp,
                reporter:   rep.reporter.clone(),
                reported:   rep.reported.clone(),
                desc:       rep.description.clone(),
            };

            irms.push(irm);
        }

        let (mut tx, rx) = mpsc::channel(4);
        let res = Arc::new(irms);

        tokio::spawn(async move {
            for result in &res[..] {
                tx.send(Ok(result.clone())).await.unwrap();
            }
        });

        Ok(Response::new(rx))
    }

    ///
    /// Query identified reports from the database / cache by
    /// their corresponding `reporter` field as a UUID string.
    ///
    async fn query_reports_by_reporter(
        &self,
        request: Request<ReportQuery>,
    ) -> Result<Response<Self::QueryReportsByReporterStream>, Status> {

        let query = request.into_inner().query;
        let queried = self.db.query_report(service::QueryType::ByReporter(query)).unwrap();

        let mut irms: Vec<IdentifiedReportMessage> = Vec::new();

        for rep in queried.iter() {
            let irm = IdentifiedReportMessage {
                id:         rep.id as i64,
                active:     rep.active,
                timestamp:  rep.timestamp,
                reporter:   rep.reporter.clone(),
                reported:   rep.reported.clone(),
                desc:       rep.description.clone(),
            };

            irms.push(irm);
        }

        let (mut tx, rx) = mpsc::channel(4);
        let res = Arc::new(irms);

        tokio::spawn(async move {
            for result in &res[..] {
                tx.send(Ok(result.clone())).await.unwrap();
            }
        });

        Ok(Response::new(rx))
    }

    ///
    /// Query identified reports from the database / cache by
    /// their corresponding `reported` field as a UUID string.
    ///
    async fn query_reports_by_reported(
        &self,
        request: Request<ReportQuery>,
    ) -> Result<Response<Self::QueryReportsByReportedStream>, Status> {

        let query = request.into_inner().query;
        let queried = self.db.query_report(service::QueryType::ByReported(query)).unwrap();

        let mut irms: Vec<IdentifiedReportMessage> = Vec::new();

        for rep in queried.iter() {

            let irm = IdentifiedReportMessage {

                id:         rep.id as i64,
                active:     rep.active,
                timestamp:  rep.timestamp,
                reporter:   rep.reporter.clone(),
                reported:   rep.reported.clone(),
                desc:       rep.description.clone(),
            };

            irms.push(irm);
        }

        let (mut tx, rx) = mpsc::channel(4);
        let res = Arc::new(irms);

        tokio::spawn(async move {
            for result in &res[..] {
                tx.send(Ok(result.clone())).await.unwrap();
            }
        });

        Ok(Response::new(rx))
    }

    async fn query_reports_by_timestamp(
        &self,
        request: Request<ReportQuery>,
    ) -> Result<Response<Self::QueryReportsByTimestampStream>, tonic::Status> {

        let query = request.into_inner().query;
        let queried = self.db.query_report(service::QueryType::ByTimestamp(query.parse::<i64>().unwrap())).unwrap();

        let mut irms: Vec<IdentifiedReportMessage> = Vec::new();

        for rep in queried.iter() {

            let irm = IdentifiedReportMessage {

                id:         rep.id as i64,
                active:     rep.active,
                timestamp:  rep.timestamp,
                reporter:   rep.reporter.clone(),
                reported:   rep.reported.clone(),
                desc:       rep.description.clone(),
            };

            irms.push(irm);
        }

        let (mut tx, rx) = mpsc::channel(4);
        let res = Arc::new(irms);

        tokio::spawn(async move {
            for result in &res[..] {
                tx.send(Ok(result.clone())).await.unwrap();
            }
        });

        Ok(Response::new(rx))
    }


    ///
    /// Query an identified report from the database by
    /// its corresponding `id` field as an integer.
    ///
    async fn query_report_by_id(
        &self,
        request: Request<ReportQuery>,
    ) -> Result<Response<IdentifiedReportMessage>, Status> {

        let query = request.into_inner().id;
        let queried = self.db.query_report(service::QueryType::ById(query as i64)).unwrap();

        let res = IdentifiedReportMessage {
            id:         queried[0].id as i64,
            active:     queried[0].active,
            timestamp:  queried[0].timestamp,
            reporter:   queried[0].reporter.clone(),
            reported:   queried[0].reported.clone(),
            desc:       queried[0].reported.clone(),
        };

        Ok(Response::new(res))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let matches = App::new("reportas-server")
        .version("0.1.0")
        .author("7Gv")
        .arg(
            Arg::with_name("address")
                .short("a")
                .long("address")
                .required(true)
                .value_name("ADDRESS")
                .help("Given address for server to listen to")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .required(true)
                .value_name("PORT")
                .help("Given TCP port for server to listen to")
                .takes_value(true),
        )
        .get_matches();

    let addr = format!(
        "{}:{}",
        matches.value_of("address").unwrap(),
        matches.value_of("port").unwrap()
    )
    .parse()?;

    let report_handler = MainReportHandler::new(&dotenv::var("DATABASE_URL").unwrap())?;

    println!("\nLISTENING TO CHANNEL BEGUN: {}\n", &addr);

    Server::builder()
        .add_service(ReportHandlerServer::new(report_handler))
        .serve(addr)
        .await?;

    Ok(())
}
