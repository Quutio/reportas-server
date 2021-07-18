use crate::report_handler;
use crate::report_handler::ReportHandler;
use crate::report_handler::Error;

use service::PgReportDb;
use service::report;
use service::report::IdentifiedReportMessage;
use service::report::ReportDeactivateRequest;
use service::report::ReportRequest;
use service::report::report_handler_server;

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Request;
use tonic::Response;
use tonic::Status;
use tracing::info;


pub struct GrpcReportHandler {
    handler: ReportHandler,
}

impl GrpcReportHandler {
    pub async fn new(addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let handler = ReportHandler::new(addr).await?;

        Ok(GrpcReportHandler {
            handler: handler,
        })
    }
}

#[tonic::async_trait]
impl report_handler_server::ReportHandler for GrpcReportHandler {

    type QueryAllReportsStream = ReceiverStream<Result<IdentifiedReportMessage, Status>>;
    type QueryReportsByReporterStream = ReceiverStream<Result<IdentifiedReportMessage, Status>>;
    type QueryReportsByReportedStream = ReceiverStream<Result<IdentifiedReportMessage, Status>>;
    type QueryReportsByActiveStream = ReceiverStream<Result<IdentifiedReportMessage, Status>>;
    type QueryReportsByTimestampStream = ReceiverStream<Result<IdentifiedReportMessage, Status>>;
    type QueryReportsByHandlerStream = ReceiverStream<Result<IdentifiedReportMessage, Status>>;
    type QueryReportsByHandleTimestampStream =
        ReceiverStream<Result<IdentifiedReportMessage, Status>>;

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

        let rep = match self.handler.submit_report(req_msg.clone().into()).await {
            Ok(val) => val,
            Err(e) => {
                match e {
                    Error::DatabaseFailed => {
                        return Err(Status::failed_precondition(e.to_string()));
                    }
                    Error::TransportError => {
                        return Err(Status::aborted(e.to_string()));
                    }
                }
            }
        };

        info!(
            "\n\nrpc#SubmitReport :: ({:?}) \n\n{:?}\n",
            &req_msg, &rep
        );

        Ok(Response::new(rep.into()))
    }

    async fn deactivate_report(
        &self,
        request: Request<ReportDeactivateRequest>,
    ) -> Result<Response<IdentifiedReportMessage>, tonic::Status> {

        let rep = self.deactivate_report(request.into_inner().into()).await.unwrap();

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
