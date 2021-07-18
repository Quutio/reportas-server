extern crate clap;
extern crate dotenv;

mod grpc;

pub mod report_handler;
pub mod report_transporter;

mod transporter;
use clap::{App, Arg};
use service::models::NewReport;
use service::*;
use tonic::{transport::Server, Request, Response, Status};
use transporter::Transporter;

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use report::report_handler_server::{ReportHandler, ReportHandlerServer};
use report::{
    IdentifiedReportMessage, ReportDeactivateRequest, ReportMessage, ReportQuery, ReportRequest,
};

use tracing::{debug, error, info};

use std::sync::Arc;
use std::sync::Mutex;

pub struct MainReportHandler {
    db: PgReportDb,
    transporter: Transporter,
    _results: Arc<Mutex<Vec<IdentifiedReportMessage>>>,
}

impl MainReportHandler {
    pub async fn new(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let db = PgReportDb::new(addr).unwrap();
        db.load_to_cache(false).await?;

        let addrs = vec!["http://[::1]:50024", "http://[::1]:50025"];

        let transporter = Transporter::new(addrs).await?;

        Ok(MainReportHandler {
            db: db,
            transporter: transporter,
            _results: Arc::new(Mutex::new(Vec::<IdentifiedReportMessage>::new())),
        })
    }
}

#[allow(unused_variables)]
#[tonic::async_trait]
impl ReportHandler for MainReportHandler {
    type QueryAllReportsStream = ReceiverStream<Result<IdentifiedReportMessage, Status>>;
    type QueryReportsByReporterStream = ReceiverStream<Result<IdentifiedReportMessage, Status>>;
    type QueryReportsByReportedStream = ReceiverStream<Result<IdentifiedReportMessage, Status>>;
    type QueryReportsByActiveStream = ReceiverStream<Result<IdentifiedReportMessage, Status>>;
    type QueryReportsByTimestampStream = ReceiverStream<Result<IdentifiedReportMessage, Status>>;
    type QueryReportsByHandlerStream = ReceiverStream<Result<IdentifiedReportMessage, Status>>;
    type QueryReportsByHandleTimestampStream =
        ReceiverStream<Result<IdentifiedReportMessage, Status>>;

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
    ) -> Result<Response<IdentifiedReportMessage>, Status> {
        let req_msg = request.into_inner().msg;
        let req_msg = match req_msg {
            Some(val) => val,
            None => {
                return Err(Status::invalid_argument("invalid argument"));
            }
        };

        let req_clone = req_msg.clone();
        let req_clone0 = req_msg.clone();

        let utc = chrono::Utc::now();
        let ts = utc.timestamp();

         let tags: Option<&str>;

        if req_clone.tags.is_empty() {
            tags = None;
        } else {
            tags = Some(&req_clone.tags);
        }

        let msg = ReportMessage {
            reporter: req_msg.reporter,
            reported: req_msg.reported,
            desc: req_msg.desc,
            tags: req_msg.tags,
        };

        let new_report = NewReport {
            active: true,
            timestamp: ts,
            reporter: req_clone.reporter.as_str(),
            reported: req_clone.reported.as_str(),
            description: req_clone.desc.as_str(),
            tags: tags,
        };

        let rep = self
            .db
            .insert_report(&new_report).await
            .expect("[!] data insertion failed");

        debug!("{:?}", &new_report);
        debug!("{:?}", &rep);

        let irm = IdentifiedReportMessage {
            id: rep.id,
            active: rep.active,
            timestamp: rep.timestamp,
            reporter: rep.reporter,
            reported: rep.reported,
            handler: rep.handler.unwrap_or("".to_owned()),
            handle_ts: rep.handle_ts.unwrap_or(-1),
            comment: rep.comment.unwrap_or("".to_owned()),
            desc: rep.description,
            tags: rep.tags.unwrap_or("".to_owned()),
        };

        info!(
            "\n\nrpc#SubmitReport :: ({:?}) \n\n{:?}\n",
            &req_clone0, &irm
        );

        match self.transporter.transport(irm.clone()).await {
            Ok(_) => {
                info!("Report with ID {} successfully transported", &irm.id);
            }
            Err(err) => {
                error!("Failed to transport Report with ID {}", &irm.id);
                return Err(Status::aborted("Failed to transport request."));
            }
        };

        let resp = report::ReportResponse { msg: Some(msg) };

        Ok(Response::new(irm))
    }

    async fn deactivate_report(
        &self,
        request: Request<ReportDeactivateRequest>,
    ) -> Result<Response<IdentifiedReportMessage>, tonic::Status> {
        let rdr = request.into_inner();

        let rdr_clone = rdr.clone();

        let id = rdr.id;
        let operator = rdr.operator;
        let comment: Option<&str>;

        if rdr.comment.is_empty() {
            comment = None;
        } else {
            comment = Some(&rdr.comment);
        }

        let rep = match self.db.deactivate_report(id, &operator, comment).await {
            Ok(val) => val,
            Err(_) => {
                error!("database failed");
                return Err(Status::failed_precondition("database failed"));
            }
        };

        let irm = report::IdentifiedReportMessage {
            id: rep.id,
            active: rep.active,
            timestamp: rep.timestamp,
            reporter: rep.reporter,
            reported: rep.reported,
            handler: rep.handler.unwrap_or("".to_owned()),
            handle_ts: rep.handle_ts.unwrap_or(-1),
            comment: rep.comment.unwrap_or("".to_owned()),
            desc: rep.description,
            tags: rep.tags.unwrap_or("".to_owned()),
        };

        info!(
            "\n\nrpc#DeactivateReport :: ({:?}) \n\n{:?}\n",
            &rdr_clone, &irm
        );

        match self.transporter.deactivate(irm.clone()).await {
            Ok(_) => {
                info!("Report with ID {} successfully transported", &irm.id);
            }
            Err(_) => {
                error!("Failed to transport Report with ID {}", &irm.id);
                return Err(Status::aborted("Failed to transport deactivation request."));
            }
        }

        Ok(Response::new(irm))
    }

    ///
    /// Query *ALL* identified reports from the database.
    ///
    async fn query_all_reports(
        &self,
        request: Request<ReportQuery>,
    ) -> Result<Response<Self::QueryAllReportsStream>, Status> {
        let req = request.into_inner();
        let query = req.clone().query;

        let queried = match self.db.query_report(service::QueryType::ALL).await {
            Ok(val) => val,
            Err(_) => {
                error!("database failed");
                return Err(Status::failed_precondition("database failed"));
            }
        };

        let mut irms: Vec<IdentifiedReportMessage> = Vec::new();

        info!(
            "\n\nrpc#QueryAllReports :: ({:?}) \n\nGot {} reports to stream\n",
            &req,
            &queried.len()
        );

        for rep in queried.iter() {
            let irm = IdentifiedReportMessage {
                id: rep.id as i64,
                active: rep.active,
                timestamp: rep.timestamp,
                reporter: rep.reporter.clone(),
                reported: rep.reported.clone(),
                handler: rep.handler.clone().unwrap_or("".to_owned()),
                handle_ts: rep.handle_ts.clone().unwrap_or(-1),
                comment: rep.comment.clone().unwrap_or("".to_owned()),
                desc: rep.description.clone(),
                tags: rep.tags.clone().unwrap_or("".to_owned()),
            };

            irms.push(irm);
        }

        let (tx, rx) = mpsc::channel(4);
        let res = Arc::new(irms);

        tokio::spawn(async move {
            for result in &res[..] {
                tx.send(Ok(result.clone())).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    ///
    /// Query identified reports from the database / cache by
    /// their corresponding `reporter` field as a UUID string.
    ///
    async fn query_reports_by_reporter(
        &self,
        request: Request<ReportQuery>,
    ) -> Result<Response<Self::QueryReportsByReporterStream>, Status> {
        let req = request.into_inner();
        let query = req.clone().query;

        let queried = match self.db.query_report(service::QueryType::ByReporter(query)).await {
            Ok(val) => val,
            Err(_) => {
                error!("database failed");
                return Err(Status::failed_precondition("database failed"));
            }
        };

        let mut irms: Vec<IdentifiedReportMessage> = Vec::new();

        info!(
            "\n\nrpc#QueryReportsByReporter :: ({:?}) \n\nGot {} reports to stream\n",
            &req,
            &queried.len()
        );

        for rep in queried.iter() {
            let irm = IdentifiedReportMessage {
                id: rep.id as i64,
                active: rep.active,
                timestamp: rep.timestamp,
                reporter: rep.reporter.clone(),
                reported: rep.reported.clone(),
                handler: rep.handler.clone().unwrap_or("".to_owned()),
                handle_ts: rep.handle_ts.clone().unwrap_or(-1),
                comment: rep.comment.clone().unwrap_or("".to_owned()),
                desc: rep.description.clone(),
                tags: rep.tags.clone().unwrap_or("".to_owned()),
            };

            irms.push(irm);
        }

        let (tx, rx) = mpsc::channel(4);
        let res = Arc::new(irms);

        tokio::spawn(async move {
            for result in &res[..] {
                tx.send(Ok(result.clone())).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    ///
    /// Query identified reports from the database / cache by
    /// their corresponding `reported` field as a UUID string.
    ///
    async fn query_reports_by_reported(
        &self,
        request: Request<ReportQuery>,
    ) -> Result<Response<Self::QueryReportsByReportedStream>, Status> {
        let req = request.into_inner();
        let query = req.clone().query;

        let queried = match self.db.query_report(service::QueryType::ByReported(query)).await {
            Ok(val) => val,
            Err(_) => {
                error!("database failed");
                return Err(Status::failed_precondition("database failed"));
            }
        };

        let mut irms: Vec<IdentifiedReportMessage> = Vec::new();

        info!(
            "\n\nrpc#QueryReportsByReported :: ({:?}) \n\nGot {} reports to stream\n",
            &req,
            &queried.len()
        );

        for rep in queried.iter() {
            let irm = IdentifiedReportMessage {
                id: rep.id as i64,
                active: rep.active,
                timestamp: rep.timestamp,
                reporter: rep.reporter.clone(),
                reported: rep.reported.clone(),
                handler: rep.handler.clone().unwrap_or("".to_owned()),
                handle_ts: rep.handle_ts.clone().unwrap_or(-1),
                comment: rep.comment.clone().unwrap_or("".to_owned()),
                desc: rep.description.clone(),
                tags: rep.tags.clone().unwrap_or("".to_owned()),
            };

            irms.push(irm);
        }

        let (tx, rx) = mpsc::channel(4);
        let res = Arc::new(irms);

        tokio::spawn(async move {
            for result in &res[..] {
                tx.send(Ok(result.clone())).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn query_reports_by_timestamp(
        &self,
        request: Request<ReportQuery>,
    ) -> Result<Response<Self::QueryReportsByTimestampStream>, tonic::Status> {
        let req = request.into_inner();
        let query = req.clone().query;

        let ts_val = match query.parse::<i64>() {
            Ok(val) => val,
            Err(_) => return Err(tonic::Status::invalid_argument("invalid timestamp")),
        };

        let queried = match self
            .db
            .query_report(service::QueryType::ByTimestamp(ts_val)).await
        {
            Ok(val) => val,
            Err(_) => {
                error!("database failed");
                return Err(tonic::Status::failed_precondition("database failed"));
            }
        };

        info!(
            "\n\nrpc#QueryReportsByTimestamp :: ({:?}) \n\nGot {} reports to stream\n",
            &req,
            &queried.len()
        );

        let mut irms: Vec<IdentifiedReportMessage> = Vec::new();

        for rep in queried.iter() {
            let irm = IdentifiedReportMessage {
                id: rep.id as i64,
                active: rep.active,
                timestamp: rep.timestamp,
                reporter: rep.reporter.clone(),
                reported: rep.reported.clone(),
                handler: rep.handler.clone().unwrap_or("".to_owned()),
                handle_ts: rep.handle_ts.clone().unwrap_or(-1),
                comment: rep.comment.clone().unwrap_or("".to_owned()),
                desc: rep.description.clone(),
                tags: rep.tags.clone().unwrap_or("".to_owned()),
            };

            irms.push(irm);
        }

        let (tx, rx) = mpsc::channel(4);
        let res = Arc::new(irms);

        tokio::spawn(async move {
            for result in &res[..] {
                tx.send(Ok(result.clone())).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    ///
    /// Query an identified report from the database by
    /// its corresponding `id` field as an integer.
    ///
    async fn query_report_by_id(
        &self,
        request: Request<ReportQuery>,
    ) -> Result<Response<IdentifiedReportMessage>, Status> {
        let req = request.into_inner();
        let query = req.clone().id;

        let queried = match self.db.query_report(service::QueryType::ById(query)).await {
            Ok(val) => val,
            Err(_) => {
                error!("database failed");
                return Err(Status::failed_precondition("database failed"));
            }
        };

        info!("\n\nrpc#QueryReportsById :: ({:?}) \n", &req);

        if queried.is_empty() {
            return Err(Status::not_found("not found"));
        }

        let res = IdentifiedReportMessage {
            id: queried[0].id as i64,
            active: queried[0].active,
            timestamp: queried[0].timestamp,
            reporter: queried[0].reporter.clone(),
            reported: queried[0].reported.clone(),
            handler: queried[0].handler.clone().unwrap_or("".to_owned()),
            handle_ts: queried[0].handle_ts.clone().unwrap_or(-1),
            comment: queried[0].comment.clone().unwrap_or("".to_owned()),
            desc: queried[0].reported.clone(),
            tags: queried[0].tags.clone().unwrap_or("".to_owned()),
        };

        Ok(Response::new(res))
    }

    async fn query_reports_by_handler(
        &self,
        request: Request<ReportQuery>,
    ) -> Result<Response<Self::QueryReportsByHandlerStream>, Status> {
        let req = request.into_inner();
        let query = req.clone().query;

        let queried = match self.db.query_report(service::QueryType::ByHandler(query)).await {
            Ok(val) => val,
            Err(_) => {
                error!("database failed");
                return Err(Status::failed_precondition("database failed"));
            }
        };

        let mut irms: Vec<IdentifiedReportMessage> = Vec::new();

        info!(
            "\n\nrpc#QueryReportsByHandler :: ({:?}) \n\nGot {} reports to stream\n",
            &req,
            &queried.len()
        );

        for rep in queried.iter() {
            let irm = IdentifiedReportMessage {
                id: rep.id as i64,
                active: rep.active,
                timestamp: rep.timestamp,
                reporter: rep.reporter.clone(),
                reported: rep.reported.clone(),
                handler: rep.handler.clone().unwrap_or("".to_owned()),
                handle_ts: rep.handle_ts.unwrap_or(-1),
                comment: rep.comment.clone().unwrap_or("".to_owned()),
                desc: rep.description.clone(),
                tags: rep.tags.clone().unwrap_or("".to_owned()),
            };

            irms.push(irm);
        }

        let (tx, rx) = mpsc::channel(4);
        let res = Arc::new(irms);

        tokio::spawn(async move {
            for result in &res[..] {
                tx.send(Ok(result.clone())).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn query_reports_by_handle_timestamp(
        &self,
        request: Request<ReportQuery>,
    ) -> Result<Response<Self::QueryReportsByHandleTimestampStream>, Status> {
        let req = request.into_inner();
        let query = req.clone().query;

        let ts_val = match query.parse::<i64>() {
            Ok(val) => val,
            Err(_) => return Err(tonic::Status::invalid_argument("invalid timestamp")),
        };

        let queried = match self
            .db
            .query_report(service::QueryType::ByHandleTimestamp(ts_val)).await
        {
            Ok(val) => val,
            Err(_) => {
                error!("database failed");
                return Err(tonic::Status::invalid_argument("invalid timestamp"));
            }
        };

        let mut irms: Vec<IdentifiedReportMessage> = Vec::new();

        info!(
            "\n\nrpc#QueryReportsByHandleTimestamp :: ({:?}) \n\nGot {} reports to stream\n",
            &req,
            &queried.len()
        );

        for rep in queried.iter() {
            let irm = IdentifiedReportMessage {
                id: rep.id as i64,
                active: rep.active,
                timestamp: rep.timestamp,
                reporter: rep.reporter.clone(),
                reported: rep.reported.clone(),
                handler: rep.handler.clone().unwrap_or("".to_owned()),
                handle_ts: rep.handle_ts.clone().unwrap_or(-1),
                comment: rep.comment.clone().unwrap_or("".to_owned()),
                desc: rep.description.clone(),
                tags: rep.tags.clone().unwrap_or("".to_owned()),
            };

            irms.push(irm);
        }

        let (tx, rx) = mpsc::channel(4);
        let res = Arc::new(irms);

        tokio::spawn(async move {
            for result in &res[..] {
                tx.send(Ok(result.clone())).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn query_reports_by_active(
        &self,
        request: Request<ReportQuery>,
    ) -> Result<Response<Self::QueryReportsByActiveStream>, tonic::Status> {
        let req = request.into_inner();
        let query = req.clone().query;
        let queried = match self.db.query_report(service::QueryType::ByActive).await {
            Ok(val) => val,
            Err(_) => {
                error!("database failed");
                return Err(Status::failed_precondition("database failed"));
            }
        };

        let mut irms: Vec<IdentifiedReportMessage> = Vec::new();

        info!(
            "\n\nrpc#QueryReportsByActive :: ({:?}) \n\nGot {} reports to stream\n",
            &req,
            &queried.len()
        );

        for rep in queried.iter() {
            let irm = IdentifiedReportMessage {
                id: rep.id as i64,
                active: rep.active,
                timestamp: rep.timestamp,
                reporter: rep.reporter.clone(),
                reported: rep.reported.clone(),
                handler: rep.handler.clone().unwrap_or("".to_owned()),
                handle_ts: rep.handle_ts.clone().unwrap_or(-1),
                comment: rep.comment.clone().unwrap_or("".to_owned()),
                desc: rep.description.clone(),
                tags: rep.tags.clone().unwrap_or("".to_owned()),
            };

            irms.push(irm);
        }

        let (tx, rx) = mpsc::channel(4);
        let res = Arc::new(irms);

        tokio::spawn(async move {
            for result in &res[..] {
                tx.send(Ok(result.clone())).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

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
    debug!("addr :: -> {}", &addr);

    let dburl = &dotenv::var("DATABASE_URL").unwrap();
    debug!("DATABASE_URL :: -> {}", &dburl);

    let report_handler = MainReportHandler::new(&dburl).await?;

    info!("ReportHandler initiated");
    info!("LISTENING TO CHANNEL BEGUN: {}", &addr);

    Server::builder()
        .add_service(ReportHandlerServer::new(report_handler))
        .serve(addr)
        .await?;

    Ok(())
}
