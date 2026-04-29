use std::collections::HashMap;

/// Trait defining the core HTTP functionality required by the application.
pub trait HttpClient: Send + Sync {
    /// Performs an HTTP GET request to the specified URL and returns the raw bytes.
    fn get_bytes(&self, url: &str, headers: Option<HashMap<String, String>>) -> Result<Vec<u8>, String>;

    /// Performs an HTTP GET request to the specified URL and returns the response body as a string.
    fn get_string(&self, url: &str, headers: Option<HashMap<String, String>>) -> Result<(String, u16), String>;
}

/// Implementation of the `HttpClient` trait using the `reqwest::blocking` client.
pub struct ReqwestClient {
    client: reqwest::blocking::Client,
}

impl ReqwestClient {
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        Self { client }
    }
}

impl HttpClient for ReqwestClient {
    fn get_bytes(&self, url: &str, headers: Option<HashMap<String, String>>) -> Result<Vec<u8>, String> {
        let mut request = self.client.get(url);

        if let Some(h) = headers {
            for (k, v) in h {
                request = request.header(k, v);
            }
        }

        request
            .send()
            .map_err(|e| e.to_string())?
            .bytes()
            .map_err(|e| e.to_string())
            .map(|b| b.to_vec())
    }

    fn get_string(&self, url: &str, headers: Option<HashMap<String, String>>) -> Result<(String, u16), String> {
        let mut request = self.client.get(url);

        if let Some(h) = headers {
            for (k, v) in h {
                request = request.header(k, v);
            }
        }

        let response = request.send().map_err(|e| e.to_string())?;
        let status = response.status().as_u16();
        let body = response.text().map_err(|e| e.to_string())?;
        Ok((body, status))
    }
}
