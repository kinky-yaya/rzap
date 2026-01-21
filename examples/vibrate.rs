use std::error;
use std::time::Duration;

use rzap_ng::OpenShockAPI;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let api_key = std::env::args()
        .skip(1)
        .next()
        .expect("This example must be passed an OpenShock API key as only argument");

    let api = OpenShockAPI::new(api_key)?.with_user_agent("rzap example".to_string())?;

    let first_hub = api
        .list_owned()
        .await?
        .into_iter()
        .next()
        .expect("For this example you need one hub set up");

    let first_shocker = first_hub
        .shockers()
        .into_iter()
        .next()
        .expect("For this example the first hub needs to have one shocker set up");

    first_shocker.vibrate(20, Duration::from_secs(2)).await?;

    Ok(())
}
