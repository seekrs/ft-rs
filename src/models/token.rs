use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct AccessToken {
  access_token: String,
  token_type: String,
  expires_in: u64,
  scope: String,
  created_at: u64
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
    self.created_at + self.expires_in <= (chrono::Utc::now().timestamp() + 5).try_into().unwrap()
  }
}