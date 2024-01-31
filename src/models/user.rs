#![allow(non_snake_case)]

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserImageData {
  pub link: String,
  pub versions: UserImageVersions
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserImageVersions {
  pub large: String,
  pub medium: String,
  pub small: String,
  pub micro: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
  pub id: u32,
  pub email: String,
  pub login: String,
  pub first_name: String,
  pub last_name: String,
  pub usual_full_name: Option<String>,
  pub usual_first_name: Option<String>,
  pub url: String,
  pub phone: String,
  pub displayname: String,
  pub kind: String, //TODO: enum
  pub image: UserImageData,
  #[serde(rename = "staff?")] // what the actual fuck
  pub staff: bool,
  pub correction_point: i32,
  pub wallet: i32,
}