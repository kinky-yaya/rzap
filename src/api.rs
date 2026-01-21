use crate::{api_builder::OpenShockAPIBuilder, data_type::*};
use std::fmt::Debug;
use std::time::Duration;

pub struct OpenShockAPI {
    pub(crate) client: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) user_agent: String,
}

impl OpenShockAPI {
    /// Return a builder for the api interface
    ///
    /// this is the same as [`OpenShockAPIBuilder::new`]
    pub fn builder(api_key: String) -> OpenShockAPIBuilder {
        OpenShockAPIBuilder::new(api_key)
    }

    pub async fn list_owned(&self) -> Result<Vec<Hub>, ListError> {
        self.list_inner("own").await
    }

    pub async fn list_shared(&self) -> Result<Vec<Hub>, ListError> {
        self.list_inner("shared").await
    }

    async fn list_inner(&self, source: &str) -> Result<Vec<Hub>, ListError> {
        let resp = self
            .client
            .get(format!("{}/1/shockers/{}", self.base_url, source))
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
                        id: s.id,
                        name: s.name,
                    })
                    .collect(),
            })
            .collect();

        Ok(hubs)
    }
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

#[derive(Debug, Clone)]
pub struct Shocker {
    id: String,
    name: Option<String>,
}

impl Shocker {
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub async fn shock(
        &self,
        api: &mut OpenShockAPI,
        intensity: u8,
        duration: Duration,
    ) -> Result<(), ControlError> {
        self.control(api, "Shock", intensity, duration).await
    }

    pub async fn beep(
        &self,
        api: &mut OpenShockAPI,
        intensity: u8,
        duration: Duration,
    ) -> Result<(), ControlError> {
        self.control(api, "Sound", intensity, duration).await
    }

    pub async fn vibrate(
        &self,
        api: &mut OpenShockAPI,
        intensity: u8,
        duration: Duration,
    ) -> Result<(), ControlError> {
        self.control(api, "Vibrate", intensity, duration).await
    }

    async fn control(
        &self,
        api: &mut OpenShockAPI,
        control_type: &str,
        intensity: u8,
        duration: Duration,
    ) -> Result<(), ControlError> {
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
    #[error("Error talking to openshock api server")]
    ContactingApi(#[from] reqwest::Error),
    #[error("Could not deserialize server reply")]
    Decoding(#[from] serde_json::Error),
}
