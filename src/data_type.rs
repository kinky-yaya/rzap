/// Variable names and data format is converted from api documentation hosted at <https://api.openshock.app/swagger/index.html>
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

/// Response provided when a request is sent to retrieve the list of available shockers
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListShockersResponse {
    pub shockers: Vec<ShockerResponse>,
    pub id: String,
    pub name: String,
    pub created_on: String,
}

/// Contains data about a shocker such as it's ID and model
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShockerResponse {
    pub name: Option<String>,
    pub is_paused: bool,
    pub created_on: String,
    pub id: String,
    pub rf_id: u32,
    pub model: ShockerModel,
}

#[derive(EnumString, Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum ShockerModel {
    CaiXianlin,
    PetTrainer,
    Petrainer998DR,
}

/// The base response used for most of the OpenShock API endpoints
#[derive(Serialize, Deserialize, Debug)]
pub struct BaseResponse<T> {
    pub message: Option<String>,
    pub data: Option<T>,
}
