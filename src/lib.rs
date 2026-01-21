//! # rzap-ng
//!
//! This library provides an interface to control shocker devices via [OpenShock](http://openshock.org)'s API
//!
//! > **Note**
//! > This is an un-official API interface, and a (hopefully temporary) fork of
//! > [`LostQuasar/rzap`].
//!
//! [`LostQuasar/rzap`]: https://github.com/LostQuasar/rzap
//!
//! ## Example
//!
//! A simple request to retrieve the API key user's id
//!
//! ```no_run
//! use rzap_ng::api::OpenShockAPI;
//! use dotenv::dotenv;
//!
//! #[tokio::main]
//! async fn main() {
//! dotenv().ok();
//! let user_test_id = dotenv::var("USER_TEST_ID").expect("missing USER_TEST_ID");
//! let openshock_token = dotenv::var("OPENSHOCK_TOKEN").expect("missing OPENSHOCK_TOKEN");
//!
//! let openshock_api = OpenShockAPI::new(None, openshock_token).unwrap();
//! println!("{}", openshock_api.get_user_info(None).await.unwrap().unwrap().id);
//! }
//! ```
//!

mod api;
mod data_type;
pub use api::{OpenShockAPI, Shocker, Hub, ControlError, CreateApiError, ListError};
