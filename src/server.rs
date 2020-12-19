extern crate clap;
use clap::{App, Arg};

use tonic::{transport::Server, Request, Response, Status};

use futures::{Stream, StreamExt};
use tokio::sync::mpsc;

use report::report_handler_server::{ReportHandler, ReportHandlerServer};
use report::{ReportMessage, IdentifiedReportMessage, ReportQuery, ReportRequest, ReportResponse};

pub mod report {
    tonic::include_proto!("report");
}

#[derive(Debug, Default)] // <- debug purposes
pub struct MainReportHandler {}

#[allow(unused_variables)]
#[tonic::async_trait]
impl ReportHandler for MainReportHandler {

    type QueryAllReportsStream = mpsc::Receiver<Result<IdentifiedReportMessage, Status>>;
    type QueryReportsByReporterStream = mpsc::Receiver<Result<IdentifiedReportMessage, Status>>;
    type QueryReportsByReportedStream = mpsc::Receiver<Result<IdentifiedReportMessage, Status>>;

    async fn submit_report(
        &self,
        request: Request<ReportRequest>,
    ) -> Result<Response<ReportResponse>, Status> {
        println!("Got a request: {:?}", request);

        let req_msg = request
            .into_inner()
            .msg
            .expect("The request was a malformed request");

        let msg = ReportMessage {
            reporter: req_msg.reporter,
            reported: req_msg.reported,
            desc: req_msg.desc,
        };

        let resp = report::ReportResponse { msg: Some(msg) };

        Ok(Response::new(resp))
    }

    async fn query_all_reports(
        &self,
        request: Request<ReportQuery>,
    ) -> Result<Response<Self::QueryAllReportsStream>, Status> {
        unimplemented!("todo");
    }

    async fn query_reports_by_reporter(
        &self,
        request: Request<ReportQuery>,
    ) -> Result<Response<Self::QueryReportsByReporterStream>, Status> {
        unimplemented!("todo");
    }

    async fn query_reports_by_reported(
        &self,
        request: Request<ReportQuery>,
    ) -> Result<Response<Self::QueryReportsByReportedStream>, Status> {
        unimplemented!("todo");
    }

    async fn query_report_by_id(&self, request: Request<ReportQuery>) ->Result<Response<IdentifiedReportMessage>, Status> {
        unimplemented!("todo");
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

    let report_handler = MainReportHandler::default();

    Server::builder()
        .add_service(ReportHandlerServer::new(report_handler))
        .serve(addr)
        .await?;

    Ok(())
}
