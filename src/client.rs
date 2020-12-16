use report::report_handler_client::ReportHandlerClient;
use report::{ReportRequest, ReportMessage};

pub mod report {
    tonic::include_proto!("report");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut client = ReportHandlerClient::connect("http://[::1]:50051").await?;

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
