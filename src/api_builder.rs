use crate::{api::OpenShockAPI, error::Error};
use reqwest::header;

/// Builder for [`OpenShockAPI`]
pub struct OpenShockAPIBuilder {
    base_url: String,
    default_key: String,
    user_agent: String,
}

impl OpenShockAPIBuilder {
    /// Create a new builder
    pub(crate) fn new(api_token: String) -> Self {
        Self {
            base_url: "https://api.openshock.app".to_string(),
            default_key: api_token,
            user_agent: format!("rzap/{}", env!("CARGO_PKG_VERSION")),
        }
    }

    /// set the base URL to use
    ///
    /// this is optional and can be provided to use a self-hosted instance of the OpenShock API. if
    /// left unset, the default (`https://api.openshock.app`) will be used.
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    /// Defaults to rzap/CARGO_PKG_VERSION
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = user_agent;
        self
    }

    /// check parameters and build an instance of [`OpenShockAPI`]
    pub fn build(self) -> Result<OpenShockAPI, Error> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Content-type",
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "accept",
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_str(&self.user_agent).map_err(Error::InvalidHeaderValue)?,
        );
        headers.insert(
            "OpenShockToken",
            header::HeaderValue::from_str(&self.default_key).unwrap(),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        Ok(OpenShockAPI {
            client,
            base_url: self.base_url,
            user_agent: self.user_agent,
        })
    }
}
