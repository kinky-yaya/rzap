use std::error;
use std::time::Duration;

use rzap_ng::api::OpenShockAPI;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let api_key = std::env::args()
        .skip(1)
        .next()
        .expect("This example must be passed an OpenShock API key as only argument");

    let mut api = OpenShockAPI::builder(api_key)
        .build()
        .expect("we provided the app and api key");

    let first_hub = api
        .list_owned()
        .await
        .unwrap()
        .into_iter()
        .next()
        .expect("For this example you need one hub set up");

    let first_shocker = first_hub
        .shockers()
        .into_iter()
        .next()
        .expect("For this example the first hub needs to have one shocker set up");

    first_shocker
        .vibrate(&mut api, 20, Duration::from_secs(2))
        .await?;

    Ok(())
}
