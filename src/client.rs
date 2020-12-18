extern crate clap;
use clap::{App, Arg};

use report::report_handler_client::ReportHandlerClient;
use report::{ReportRequest, ReportMessage};

pub mod report {
    tonic::include_proto!("report");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {


    let matches = App::new("reportas-client")
        .version("0.1.0")
        .author("7Gv")
        .arg(Arg::with_name("address")
             .short("a")
             .long("address")
             .required(true)
             .value_name("ADDRESS")
             .help("Given address for server to listen to")
             .takes_value(true))
        .arg(Arg::with_name("port")
             .short("p")
             .long("port")
             .required(true)
             .value_name("PORT")
             .help("Given TCP port for server to listen to")
             .takes_value(true))
        .get_matches();


    let domain = format!(
        "https://{}:{}",
        matches.value_of("address").unwrap(),
        matches.value_of("port").unwrap()
    );

    let mut client = ReportHandlerClient::connect(domain).await?;

    let msg = ReportMessage {
        reporter_uuid: uuid::Uuid::new_v4().to_string(),
        reported_uuid: uuid::Uuid::new_v4().to_string(),
        description: "joujou".into(),
    };

    let request = tonic::Request::new(

        ReportRequest {
            msg: Some(msg)}
    );

    let resp = client.submit_report(request).await?;

    println!("RESPONSE: {:?}", resp);

    Ok(())
}
