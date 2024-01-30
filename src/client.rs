use reqwest::Client;
use std::time::Duration;

use crate::{models::{self, token::AccessToken}, FtError, Result};

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

const API_URL: &str = "https://api.intra.42.fr";

macro_rules! endpoint {
  ($path:expr) => { format!("{}{}", API_URL, $path) };
}

/// The client struct, used to make requests to the API.
///
/// You can create a client with the [`from_app`](#method.from_app) method.
pub struct FtClient {
  app_uid: String,
  app_secret: String,

  client: Client,
  last_valid_token: Option<AccessToken>
}

impl FtClient {
  /// Creates a new client for a v2 application, providing the application's UID and secret.
  /// 
  /// ```rust
  /// use ft_rs::FtClient;
  /// 
  /// let client = FtClient::from_app("my_uid", "my_super_secret_secret");
  /// ```
  /// 
  /// # Errors
  /// 
  /// This method will return an error if the reqwest client could not be built, or if the UID or secret are invalid.
  pub fn from_app<U: Into<String>, S: Into<String>>(
    app_uid: U, 
    app_secret: S,
  ) -> crate::Result<Self> {
    let app_uid = app_uid.into();
    let app_secret = app_secret.into();

    let client = reqwest::ClientBuilder::new()
      .user_agent(format!("{}/{}", PKG_NAME, PKG_VERSION))
      .connect_timeout(Duration::from_secs(30))
      .build();

    if let Err(err) = client {
      Err(FtError::ReqwestBuilderError(err))
    } else {
      Ok(Self { 
        app_uid,
        app_secret,
  
        client: client.unwrap(),
        last_valid_token: None
      })
    }
  }

  /// Fetches a new access token from the API.
  /// 
  /// This method will return the last valid token if it is still valid, as per the API's documentation.
  /// 
  /// Note that the API Client will automatically fetch a new token if the last one is expired, so there is no need to call this method manually.
  /// 
  /// # Example
  /// 
  /// ```no_run
  /// use ft_rs::FtClient;
  /// 
  /// #[tokio::main]
  /// async fn main() -> ft_rs::Result<()> {
  ///   let client = FtClient::from_app("my_uid", "my_super_secret_secret")?;
  ///   let token = client.fetch_token().await?;
  ///   println!("Token: {:?}", token);
  ///   Ok(())
  /// }
  /// ```
  pub async fn fetch_token(&self) -> Result<models::token::AccessToken> {
    let res = self.client
      .post(endpoint!("/oauth/token"))
      .form(&[
        ("grant_type", "client_credentials"),
        ("client_id", &self.app_uid),
        ("client_secret", &self.app_secret)
      ])
      .send()
      .await?;
    let text = res.text().await?;
    let json_object = serde_json::from_str::<serde_json::Value>(&text)?;

    if let Some(error) = json_object.get("error") {
      let error = error.as_str().unwrap();
      let error_description = json_object.get("error_description")
        .unwrap_or(&serde_json::Value::Null)
        .as_str()
        .unwrap_or("No description provided.");
      Err(FtError::from_api_error(
        error.parse()?,
        error_description.to_string()
      ))
    } else {
      let token = serde_json::from_str::<AccessToken>(&text)?;
      Ok(token)
    }
  }

  /// Ensures that the last valid token is still valid, and fetches a new one if it is not.
  /// 
  /// This method is called automatically by the API Client when making a request, so there is no need to call it manually.
  pub async fn ensure_valid_token(&mut self) -> Result<()> {
    if let Some(token) = &self.last_valid_token {
      if token.is_expired() {
        self.last_valid_token = Some(self.fetch_token().await?);
      }
    } else {
      self.last_valid_token = Some(self.fetch_token().await?);
    }
    Ok(())
  }
}