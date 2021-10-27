extern crate clap;
extern crate dotenv;

mod grpc;

pub mod report_handler;
pub mod report_transporter;

use grpc::report_handler::GrpcReportHandler;

use clap::{App, Arg};
use service::*;

use report::report_handler_server::ReportHandlerServer;

use tonic::transport::Server;
use tracing::{debug, info, Level};

fn setup_log() {
    if cfg!(debug_assertions) {
        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .init();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    setup_log();

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

    let report_handler = GrpcReportHandler::new(&dburl).await?;

    info!("ReportHandler initiated");
    info!("LISTENING TO CHANNEL BEGUN: {}", &addr);

    Server::builder()
        .add_service(ReportHandlerServer::new(report_handler))
        .serve(addr)
        .await?;

    Ok(())
}
