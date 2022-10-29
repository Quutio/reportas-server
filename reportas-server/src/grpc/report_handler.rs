use std::sync::Arc;

use crate::report_handler::Error;
use crate::report_handler::ReportHandler;

use service::report::report_handler_server;
use service::report::IdentifiedReportMessage;
use service::report::ReportDeactivateRequest;
use service::report::ReportQuery;
use service::report::ReportRequest;

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

        Ok(GrpcReportHandler { handler })
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
            Err(e) => match e {
                Error::DatabaseFailed => {
                    return Err(Status::failed_precondition(e.to_string()));
                }
                Error::TransportError => {
                    return Err(Status::aborted(e.to_string()));
                }
                Error::InvalidTimestamp => {
                    return Err(Status::invalid_argument(e.to_string()));
                }
            },
        };

        info!("\n\nrpc#SubmitReport :: ({:?}) \n\n{:?}\n", &req_msg, &rep);

        Ok(Response::new(rep.into()))
    }

    async fn deactivate_report(
        &self,
        request: Request<ReportDeactivateRequest>,
    ) -> Result<Response<IdentifiedReportMessage>, tonic::Status> {
        let rdr = request.into_inner();

        let rep = match self.handler.deactivate_report(rdr.clone().into()).await {
            Ok(val) => val,
            Err(e) => match e {
                Error::DatabaseFailed => {
                    return Err(Status::failed_precondition(e.to_string()));
                }
                Error::TransportError => {
                    return Err(Status::aborted(e.to_string()));
                }
                Error::InvalidTimestamp => {
                    return Err(Status::invalid_argument(e.to_string()));
                }
            },
        };

        info!("\n\nrpc#DeactivateReport :: ({:?}) \n\n{:?}\n", &rdr, &rep);

        Ok(Response::new(rep.into()))
    }

    ///
    /// Query *ALL* identified reports from the database.
    ///
    async fn query_all_reports(
        &self,
        request: Request<ReportQuery>,
    ) -> Result<Response<Self::QueryAllReportsStream>, Status> {
        let req = request.into_inner();

        let res = match self.handler.query_all_reports().await {
            Ok(val) => val,
            Err(e) => match e {
                Error::DatabaseFailed => {
                    return Err(Status::failed_precondition(e.to_string()));
                }
                Error::TransportError => {
                    return Err(Status::aborted(e.to_string()));
                }
                Error::InvalidTimestamp => {
                    return Err(Status::invalid_argument(e.to_string()));
                }
            },
        };

        info!(
            "\n\nrpc#QueryAllReports :: ({:?}) \n\nGot {} reports to stream\n",
            &req,
            &res.len()
        );

        let mut irms = Vec::<IdentifiedReportMessage>::new();

        for rep in res.into_iter() {
            let irm = rep.into();

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

        let res = match self
            .handler
            .query_reports_by_reporter(req.clone().into())
            .await
        {
            Ok(val) => val,
            Err(e) => match e {
                Error::DatabaseFailed => {
                    return Err(Status::failed_precondition(e.to_string()));
                }
                Error::TransportError => {
                    return Err(Status::aborted(e.to_string()));
                }
                Error::InvalidTimestamp => {
                    return Err(Status::invalid_argument(e.to_string()));
                }
            },
        };

        let mut irms: Vec<IdentifiedReportMessage> = Vec::new();

        info!(
            "\n\nrpc#QueryReportsByReporter :: ({:?}) \n\nGot {} reports to stream\n",
            &req,
            &res.len()
        );

        for rep in res.into_iter() {
            let irm = rep.into();
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

        let res = match self
            .handler
            .query_reports_by_reported(req.clone().into())
            .await
        {
            Ok(val) => val,
            Err(e) => match e {
                Error::DatabaseFailed => {
                    return Err(Status::failed_precondition(e.to_string()));
                }
                Error::TransportError => {
                    return Err(Status::aborted(e.to_string()));
                }
                Error::InvalidTimestamp => {
                    return Err(Status::invalid_argument(e.to_string()));
                }
            },
        };

        let mut irms: Vec<IdentifiedReportMessage> = Vec::new();

        info!(
            "\n\nrpc#QueryReportsByReported :: ({:?}) \n\nGot {} reports to stream\n",
            &req,
            &res.len()
        );

        for rep in res.into_iter() {
            let irm = rep.into();
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

        let res = match self
            .handler
            .query_reports_by_timestamp(req.clone().into())
            .await
        {
            Ok(val) => val,
            Err(e) => match e {
                Error::DatabaseFailed => {
                    return Err(Status::failed_precondition(e.to_string()));
                }
                Error::TransportError => {
                    return Err(Status::aborted(e.to_string()));
                }
                Error::InvalidTimestamp => {
                    return Err(Status::invalid_argument(e.to_string()));
                }
            },
        };

        info!(
            "\n\nrpc#QueryReportsByTimestamp :: ({:?}) \n\nGot {} reports to stream\n",
            &req,
            &res.len()
        );

        let mut irms: Vec<IdentifiedReportMessage> = Vec::new();

        for rep in res.into_iter() {
            let irm = rep.into();
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

        let res = match self.handler.query_reports_by_id(req.clone().into()).await {
            Ok(val) => val,
            Err(e) => match e {
                Error::DatabaseFailed => {
                    return Err(Status::failed_precondition(e.to_string()));
                }
                Error::TransportError => {
                    return Err(Status::aborted(e.to_string()));
                }
                Error::InvalidTimestamp => {
                    return Err(Status::invalid_argument(e.to_string()));
                }
            },
        };

        info!("\n\nrpc#QueryReportsById :: ({:?}) \n", &req);

        if res.is_empty() {
            return Err(Status::not_found("not found"));
        }

        let res0 = res[0].clone();

        Ok(Response::new(res0.into()))
    }

    async fn query_reports_by_handler(
        &self,
        request: Request<ReportQuery>,
    ) -> Result<Response<Self::QueryReportsByHandlerStream>, Status> {
        let req = request.into_inner();

        let res = match self
            .handler
            .query_reports_by_handler(req.clone().into())
            .await
        {
            Ok(val) => val,
            Err(e) => match e {
                Error::DatabaseFailed => {
                    return Err(Status::failed_precondition(e.to_string()));
                }
                Error::TransportError => {
                    return Err(Status::aborted(e.to_string()));
                }
                Error::InvalidTimestamp => {
                    return Err(Status::invalid_argument(e.to_string()));
                }
            },
        };

        let mut irms: Vec<IdentifiedReportMessage> = Vec::new();

        info!(
            "\n\nrpc#QueryReportsByHandler :: ({:?}) \n\nGot {} reports to stream\n",
            &req,
            &res.len()
        );

        for rep in res.into_iter() {
            let irm = rep.into();
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

        let res = match self
            .handler
            .query_reports_by_handle_timestamp(req.clone().into())
            .await
        {
            Ok(val) => val,
            Err(e) => match e {
                Error::DatabaseFailed => {
                    return Err(Status::failed_precondition(e.to_string()));
                }
                Error::TransportError => {
                    return Err(Status::aborted(e.to_string()));
                }
                Error::InvalidTimestamp => {
                    return Err(Status::invalid_argument(e.to_string()));
                }
            },
        };

        let mut irms: Vec<IdentifiedReportMessage> = Vec::new();

        info!(
            "\n\nrpc#QueryReportsByHandleTimestamp :: ({:?}) \n\nGot {} reports to stream\n",
            &req,
            &res.len()
        );

        for rep in res.into_iter() {
            let irm = rep.into();
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

        let res = match self.handler.query_reports_by_active().await {
            Ok(val) => val,
            Err(e) => match e {
                Error::DatabaseFailed => {
                    return Err(Status::failed_precondition(e.to_string()));
                }
                Error::TransportError => {
                    return Err(Status::aborted(e.to_string()));
                }
                Error::InvalidTimestamp => {
                    return Err(Status::invalid_argument(e.to_string()));
                }
            },
        };

        let mut irms: Vec<IdentifiedReportMessage> = Vec::new();

        info!(
            "\n\nrpc#QueryReportsByActive :: ({:?}) \n\nGot {} reports to stream\n",
            &req,
            &res.len()
        );

        for rep in res.into_iter() {
            let irm = rep.into();
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
