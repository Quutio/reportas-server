
use report::report_transporter_client::ReportTransporterClient;
use report::IdentifiedReportMessage;

pub mod report {
    tonic::include_proto!("report");
}

pub async fn transport(irm: IdentifiedReportMessage) -> Result<(), Box<dyn std::error::Error>> {

    let mut client = ReportTransporterClient::connect("http://[::1]:50024").await?;
    let request = tonic::Request::new(irm);

    let status = client.broadcast_report(request).await?;

    Ok(())
}
