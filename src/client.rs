use reqwest::Client;
use std::time::Duration;

use crate::{models::{self, token::AccessToken}, FtError, Result};

const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

const API_URL: &str = "https://api.intra.42.fr";

macro_rules! endpoint {
  ($path:expr) => { format!("{}{}", API_URL, $path) };
}

enum AuthType {
  App {
    uid: String,
    secret: String,

    last_token: Option<AccessToken>
  },
  User {
    token: AccessToken
  }
}

/// The client struct, used to make requests to the API.
///
/// You can create a client with the [`from_app`](#method.from_app) method.
pub struct FtClient {
  auth_type: AuthType,
  client: Client,
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
  ) -> Result<Self> {
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
        auth_type: AuthType::App { 
          uid: app_uid,
          secret: app_secret,
          last_token: None
        },
        client: client.unwrap(),
      })
    }
  }

  pub fn from_user(token: AccessToken) -> Result<Self> {
    let client = reqwest::ClientBuilder::new()
      .user_agent(format!("{}/{}", PKG_NAME, PKG_VERSION))
      .connect_timeout(Duration::from_secs(30))
      .build();

    if let Err(err) = client {
      Err(FtError::ReqwestBuilderError(err))
    } else {
      Ok(Self { 
        auth_type: AuthType::User { 
          token
        },
        client: client.unwrap(),
      })
    }
  }

  async fn handle_error(&self, res: reqwest::Response) -> Result<()> {
    let status = res.status().as_u16();
    let text = res.text().await?;
    let json_object = serde_json::from_str::<serde_json::Value>(&text)?;
    
    let error = json_object.get("error")
      .unwrap_or(&serde_json::Value::Null)
      .as_str()
      .unwrap_or("unknown");
    let error_description = json_object.get("error_description")
      .unwrap_or(&serde_json::Value::Null)
      .as_str()
      .unwrap_or("No description provided.");
    let status = json_object.get("status")
      .unwrap_or(&serde_json::Value::Null)
      .as_u64()
      .unwrap_or(status as _);

    Err(FtError::from_api_error(
      error.parse()?,
      status,
      error_description.to_string()
    ))
  }

  /// Fetches a new app access token from the API.
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
  ///   let token = client.fetch_app_token().await?;
  ///   println!("Token: {:?}", token);
  ///   Ok(())
  /// }
  /// ```
  pub async fn fetch_app_token(&self, extra_data: bool) -> Result<models::token::AccessToken> {
    if let AuthType::App { uid, secret, .. } = &self.auth_type {
      let res = self.client
        .post(endpoint!("/oauth/token"))
        .form(&[
          ("grant_type", "client_credentials"),
          ("client_id", &uid),
          ("client_secret", &secret)
        ])
        .send()
        .await?;
      
      if res.status().is_success() {
        let token = res.json::<AccessToken>().await?;
        if extra_data {
          unimplemented!("Extra data is not implemented yet.");
        }
        Ok(token)
      } else {
        self.handle_error(res).await?;
        unreachable!()
      }
    } else {
      Err(FtError::InvalidAuthType)
    }
  }

  /// Ensures that the current token is still valid, and fetches a new one if it is not.
  /// 
  /// This method is called automatically by the API Client when making a request, so there is no need to call it manually.
  pub async fn ensure_app_token(&mut self) -> Result<()> {
    // self.auth_type.try_refresh_token(self).await?;
    Ok(())
  }

  pub fn get_authorization_url(&self, callback_url: &str, scopes: &[&str]) -> Result<String> {
    if let AuthType::App { uid, .. } = &self.auth_type {
      Ok(format!(
        "{}/oauth/authorize?client_id={}&redirect_uri={}&scope={}&response_type=code",
        API_URL,
        uid,
        urlencoding::encode(callback_url),
        scopes.join(" ")
      ))
    } else {
      Err(FtError::InvalidAuthType)
    }
  }

  pub async fn fetch_access_token(&self, code: &str, callback_url: &str) -> Result<AccessToken> {
    if let AuthType::App { uid, secret, .. } = &self.auth_type {
      let res = self.client
        .post(endpoint!("/oauth/token"))
        .form(&[
          ("grant_type", "authorization_code"),
          ("client_id", &uid),
          ("client_secret", &secret),
          ("code", code),
          ("redirect_uri", callback_url),
        ])
        .send()
        .await?;

      if res.status().is_success() {
        let text = res.text().await?;
        println!("User Token: {:?}", text);
        // let token = res.json::<AccessToken>().await?;
        let token = serde_json::from_str::<AccessToken>(&text)?;
        Ok(token)
      } else {
        self.handle_error(res).await?;
        unreachable!()
      }
    } else {
      Err(FtError::InvalidAuthType)
    }
  }

  pub async fn fetch_user_data(&self, token: &AccessToken) -> Result<models::user::User> {
    let res = self.client
      .get(endpoint!("/v2/me"))
      .bearer_auth(token.access_token.clone())
      .send()
      .await?;

    if res.status().is_success() {
      let text = res.text().await?;
      println!("{}", text);
      let user = serde_json::from_str::<models::user::User>(&text)?;
      Ok(user)
    } else {
      self.handle_error(res).await?;
      unreachable!()
    }
  }
}