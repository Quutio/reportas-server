use tonic::transport::Endpoint;
use tracing::info;

use crate::report::report_transporter_client::ReportTransporterClient;
use crate::report::IdentifiedReportMessage;
use crate::report::ReportId;

pub struct Transporter {
    endpoints: Vec<Endpoint>,
}

impl Transporter {
    pub async fn new(addrs: Vec<&'static str>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut endpoints = Vec::<Endpoint>::new();

        for addr in addrs.iter() {
            let endpoint = Endpoint::from_static(addr);
            endpoints.push(endpoint);
        }

        Ok(Self { endpoints })
    }

    pub async fn transport(
        &self,
        irm: IdentifiedReportMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Attempting to transport to the following ENDPOINTS:");

        for endpoint in self.endpoints.iter() {
            info!("{}", endpoint.uri());

            if let Ok(e) = endpoint.connect().await {
                let mut client = ReportTransporterClient::new(e);
                let request = tonic::Request::new(irm.clone());

                let _status = client.broadcast_report(request).await?;
            }
        }

        Ok(())
    }

    pub async fn deactivate(
        &self,
        irm: IdentifiedReportMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Attempting to transport to the following ENDPOINTS:");

        for endpoint in self.endpoints.iter() {
            info!("{}", endpoint.uri());

            if let Ok(e) = endpoint.connect().await {
                let mut client = ReportTransporterClient::new(e);
                let request = tonic::Request::new(irm.clone());

                let _status = client.broadcast_deactivate(request).await?;
            }
        }

        Ok(())
    }
}
