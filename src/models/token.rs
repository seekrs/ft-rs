use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct AccessToken {
  pub(crate) access_token: String,
  token_type: String,
  expires_in: u64,
  scope: String,
  created_at: u64,
  pub(crate) refresh_token: Option<String>,
  pub(crate) secret_valid_until: Option<u64>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TokenInfo {
  resource_owner_id: u32,
  scopes: Vec<String>,
  expires_in_seconds: u64,
  application: ApplicationInfo,
  created_at: u64
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApplicationInfo {
  uid: String,
  name: String,
  redirect_uri: String
}

impl AccessToken {
  /// Checks if the token is expired.
  /// 
  /// This method will return true if the token is expired, and false otherwise.
  /// 
  /// # Panics
  /// 
  /// This method will panic if the system's time is not set correctly.
  pub fn is_expired(&self) -> bool {
    self.created_at + self.expires_in <= (chrono::Utc::now().timestamp() as u64 + 5)
  }

  /// Checks if the token can be renewed.
  /// 
  /// This method will return true if the token can be renewed, and false otherwise.
  /// 
  /// # Panics
  /// 
  /// This method will panic if the system's time is not set correctly.
  pub fn can_renew(&self) -> bool {
    if self.refresh_token.is_some() {
      if let Some(secret_valid_until) = self.secret_valid_until {
        secret_valid_until <= (chrono::Utc::now().timestamp() as u64 + 5)
      } else {
        true
      }
    } else {
      false
    }
  }
}