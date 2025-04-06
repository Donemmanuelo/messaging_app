use hyper::{Body, Client, Request, Uri};
use hyper_tls::HttpsConnector;
use log::{info, error};

pub struct SecureTransmissionService {
    client: Client<HttpsConnector<hyper::client::HttpConnector>>,
}

impl SecureTransmissionService {
    pub fn new() -> Self {
        info!("Secure transmission service initialized");
        let https = HttpsConnector::new();
        let client = Client::builder().build(https);
        SecureTransmissionService { client }
    }

    pub async fn send_request(&self, uri: Uri, body: Body) -> Result<hyper::Response<Body>, hyper::Error> {
        info!("Sending secure request to: {}", uri);
        let request = Request::builder()
            .uri(uri)
            .method("POST")
            .header("Content-Type", "application/json")
            .body(body)
            .expect("Failed to build request");

        match self.client.request(request).await {
            Ok(response) => {
                info!("Request sent successfully: {:?}", response);
                Ok(response)
            }
            Err(e) => {
                error!("Failed to send request: {}", e);
                Err(e)
            }
        }
    }
}