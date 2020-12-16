use tonic::{transport::Server, Request, Response, Status};

use report::report_handler_server::{ReportHandler, ReportHandlerServer};
use report::{ReportRequest, ReportResponse, ReportMessage};

pub mod report {
    tonic::include_proto!("report");
}

#[derive(Debug, Default)]
pub struct MainReportHandler {}

#[tonic::async_trait]
impl ReportHandler for MainReportHandler {

    async fn submit_report(&self, request: Request<ReportRequest>) -> Result<Response<ReportResponse>, Status> {

        println!("Got a request: {:?}", request);

        let req_msg = request.into_inner().msg
            .expect("The request was a malformed request");

        let msg = ReportMessage {
            reporter_uuid: req_msg.reporter_uuid,
            reported_uuid: req_msg.reported_uuid,
            description: req_msg.description,
        };

        let resp = report::ReportResponse {
            msg: Some(msg),
        };

        Ok(Response::new(resp))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let addr = "[::1]:50051".parse()?;
    let report_handler = MainReportHandler::default();

    Server::builder()
        .add_service(ReportHandlerServer::new(report_handler))
        .serve(addr)
        .await?;

    Ok(())
}
