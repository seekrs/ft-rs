use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct AccessToken {
  access_token: String,
  token_type: String,
  expires_in: u32,
  scope: String,
  created_at: u32
}

impl AccessToken {
  pub fn is_expired(&self) -> bool {
    self.created_at + self.expires_in <= (chrono::Utc::now().timestamp() + 5) as u32
  }
}