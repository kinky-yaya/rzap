use reqwest::header::InvalidHeaderValue;
use reqwest::{Client, header};

use crate::data_type::*;
use std::fmt::Debug;
use std::rc::{self, Rc};
use std::time::Duration;

pub struct OpenShockAPI {
    inner: Rc<Inner>,
}

#[derive(Debug, Clone)]
struct Inner {
    client: reqwest::Client,
    base_url: String,
    user_agent: String,
    api_key: String,
}

impl OpenShockAPI {
    pub fn new(api_key: String) -> Result<Self, CreateApiError> {
        let user_agent = format!("rzap/{}", env!("CARGO_PKG_VERSION"));
        Ok(Self {
            inner: Rc::new(Inner {
                client: client(&user_agent, &api_key)?,
                base_url: "https://api.openshock.app".to_string(),
                api_key,
                user_agent: format!("rzap/{}", env!("CARGO_PKG_VERSION")),
            }),
        })
    }

    /// this is optional and can be provided to use a self-hosted instance of the OpenShock API. if
    /// left unset, the default is (`https://api.openshock.app`).
    pub fn with_base_url(mut self, base_url: String) -> Self {
        let inner = Rc::make_mut(&mut self.inner);
        inner.base_url = base_url;
        self
    }

    /// defaults is rzap/CARGO_PKG_VERSION
    pub fn with_user_agent(mut self, user_agent: String) -> Result<Self, CreateApiError> {
        let inner = Rc::make_mut(&mut self.inner);
        inner.client = client(&user_agent, &inner.api_key)?;
        inner.user_agent = user_agent;
        Ok(self)
    }

    pub async fn list_owned<'a>(&'a self) -> Result<Vec<Hub>, ListError> {
        self.list_inner("own").await
    }

    pub async fn list_shared<'a>(&'a self) -> Result<Vec<Hub>, ListError> {
        self.list_inner("shared").await
    }

    async fn list_inner<'a>(&'a self, source: &str) -> Result<Vec<Hub>, ListError> {
        let resp = self
            .inner
            .client
            .get(format!("{}/1/shockers/{}", self.inner.base_url, source))
            .send()
            .await?
            .error_for_status()?;
        let body = resp.text().await?;
        let list_shockers_response: BaseResponse<Vec<ListShockersResponse>> =
            serde_json::from_str(&body)?;

        let Some(hubs) = list_shockers_response.data else {
            return Ok(Vec::new());
        };

        let hubs = hubs
            .into_iter()
            .map(|hub| Hub {
                name: hub.name,
                shockers: hub
                    .shockers
                    .into_iter()
                    .map(|s| Shocker {
                        api: Rc::downgrade(&self.inner),
                        id: s.id,
                        name: s.name,
                    })
                    .collect(),
            })
            .collect();

        Ok(hubs)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateApiError {
    #[error("Error setting up http client")]
    ContactingApi(#[from] reqwest::Error),
    #[error("Provided user agent can not be a header value")]
    InvalidUserAgent(InvalidHeaderValue),
    #[error("Incorrect Api token")]
    IncorrectApiToken(InvalidHeaderValue),
}

fn client(user_agent: &str, api_key: &str) -> Result<Client, CreateApiError> {
    use CreateApiError as E;

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
        header::HeaderValue::from_str(user_agent).map_err(E::InvalidUserAgent)?,
    );
    headers.insert(
        "OpenShockToken",
        header::HeaderValue::from_str(api_key).map_err(E::IncorrectApiToken)?,
    );
    Ok(reqwest::Client::builder()
        .default_headers(headers)
        .build()?)
}

#[derive(thiserror::Error, Debug)]
pub enum ListError {
    #[error("Server replied with unknown acknowledge: {0:?}")]
    UnexpectedServerReply(Option<String>),
    #[error("Error talking to OpenShock api server")]
    ContactingApi(#[from] reqwest::Error),
    #[error("Could not deserialize server reply")]
    Decoding(#[from] serde_json::Error),
}

pub struct Hub {
    name: String,
    shockers: Vec<Shocker>,
}

impl Hub {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn shockers(&self) -> &[Shocker] {
        &self.shockers
    }
}

pub struct Shocker {
    api: rc::Weak<Inner>,
    id: String,
    name: Option<String>,
}

impl Shocker {
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub async fn shock(&self, intensity: u8, duration: Duration) -> Result<(), ControlError> {
        self.control("Shock", intensity, duration).await
    }

    pub async fn beep(&self, intensity: u8, duration: Duration) -> Result<(), ControlError> {
        self.control("Sound", intensity, duration).await
    }

    pub async fn vibrate(&self, intensity: u8, duration: Duration) -> Result<(), ControlError> {
        self.control("Vibrate", intensity, duration).await
    }

    async fn control(
        &self,
        control_type: &str,
        intensity: u8,
        duration: Duration,
    ) -> Result<(), ControlError> {
        let api = &self.api.upgrade().ok_or(ControlError::ApiDropped)?;

        let duration = if (300..=30000).contains(&duration.as_millis()) {
            duration.as_millis() as u16
        } else {
            return Err(ControlError::DurationOutOfRange);
        };

        if !(1..=100).contains(&intensity) {
            return Err(ControlError::IntensityOutOfRange);
        }

        let control_request = serde_json::json!({
            "shocks": [
                {
                    "id": self.id,
                    "type": control_type,
                    "intensity": intensity,
                    "duration": duration,
                    "exclusive": true,
                }
            ],
            "custom_name": api.user_agent,
        });

        let resp = api
            .client
            .post(format!("{}/2/shockers/control", api.base_url))
            .json(&control_request)
            .send()
            .await?
            .error_for_status()?;
        let base_response: BaseResponse<String> =
            serde_json::from_str(resp.text().await?.as_str())?;

        if let Some(ref reply) = base_response.message
            && reply == "Successfully sent control messages"
        {
            Ok(())
        } else {
            Err(ControlError::UnexpectedServerReply(
                base_response.message.map(|s| s.to_string()),
            ))
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ControlError {
    #[error("Duration has to be at least 0.3s up to and including 10m")]
    DurationOutOfRange,
    #[error("Intensity must be at least 1 up to and including 100")]
    IntensityOutOfRange,
    #[error("Server replied with unknown acknowledge: {0:?}")]
    UnexpectedServerReply(Option<String>),
    #[error("Error talking to OpenShock API server, is the API key correct?")]
    ContactingApi(#[from] reqwest::Error),
    #[error("Could not deserialize server reply")]
    Decoding(#[from] serde_json::Error),
    #[error("The OpenShockAPI was dropped or changed")]
    ApiDropped,
}
