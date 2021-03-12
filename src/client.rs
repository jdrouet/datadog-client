use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    Body(StatusCode, Vec<String>),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Reqwest(err)
    }
}

#[derive(Deserialize)]
struct BodyError {
    errors: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Client {
    host: String,
    api_key: String,
}

impl Client {
    pub fn new(host: String, api_key: String) -> Self {
        Self { host, api_key }
    }
}

impl Client {
    pub async fn post<T: Serialize>(&self, path: &str, payload: &T) -> Result<(), Error> {
        let client = reqwest::Client::new();
        let url = format!("{}{}", self.host, path);
        let response = client
            .post(url.as_str())
            .header("Content-Type", "application/json")
            .header("DD-API-KEY", self.api_key.as_str())
            .json(payload)
            .send()
            .await?;
        let status = response.status();
        if status.is_client_error() || status.is_server_error() {
            let body = response.json::<BodyError>().await?;
            Err(Error::Body(status, body.errors))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;

    #[tokio::test]
    async fn post_success() {
        let call = mock("POST", "/somewhere").with_status(202).create();
        let client = Client::new(mockito::server_url(), String::from("fake-api-key"));
        let result = client
            .post("/somewhere", &String::from("Hello World!"))
            .await;
        assert!(result.is_ok());
        call.expect(1);
    }

    #[tokio::test]
    async fn post_authentication_error() {
        let call = mock("POST", "/somewhere")
            .with_status(403)
            .with_body("{\"errors\":[\"Authentication error\"]}")
            .create();
        let client = Client::new(mockito::server_url(), String::from("fake-api-key"));
        let result = client
            .post("/somewhere", &String::from("Hello World!"))
            .await;
        assert!(result.is_err());
        call.expect(1);
    }
}
